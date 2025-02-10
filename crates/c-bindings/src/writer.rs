use std::{fs, io, slice, ffi};
use vach::prelude::*;
use super::errors;

/// Archive Builder Context
pub type v_builder_ctx = ffi::c_void;
type _builder_ctx_inner = (BuilderConfig, Vec<Leaf<'static>>);

/// Create new Builder Context
#[no_mangle]
pub extern "C" fn new_builder_ctx(sk_bytes: *const [u8; super::V_SECRET_KEY_LENGTH], flags: u32) -> *mut v_builder_ctx {
	let signing_key = unsafe { sk_bytes.as_ref() }.map(SigningKey::from_bytes);
	let flags = Flags::from_bits(flags);

	let config = BuilderConfig {
		flags,
		signing_key,
		..Default::default()
	};

	Box::into_raw(Box::<_builder_ctx_inner>::new((config, Vec::new()))) as _
}

/// free memory bound by `new_builder_ctx`
#[no_mangle]
pub extern "C" fn free_builder_ctx(ctx: *mut v_builder_ctx) {
	if let Some(config) = unsafe { (ctx as *mut _builder_ctx_inner).as_mut() } {
		unsafe {
			drop(Box::from_raw(config));
		}
	}
}

fn new_leaf<'a, T: io::Read + Sync + Send + 'a>(id: *const ffi::c_char, data: T, flags: u32) -> Option<Leaf<'a>> {
	// get ID
	let id = unsafe { std::ffi::CStr::from_ptr(id).to_str() }.ok()?;

	// create leaf
	let mut leaf = Leaf::new(data, id);

	// set Rust booleans from flags data
	leaf.encrypt = leaf.flags.contains(Flags::ENCRYPTED_FLAG);
	leaf.sign = leaf.flags.contains(Flags::SIGNED_FLAG);

	// check for compression flags
	if leaf.flags.contains(Flags::BROTLI_COMPRESSED) {
		leaf.compress = CompressMode::Detect;
		leaf.compression_algo = CompressionAlgorithm::Brotli(11);
	} else if leaf.flags.contains(Flags::SNAPPY_COMPRESSED) {
		leaf.compress = CompressMode::Detect;
		leaf.compression_algo = CompressionAlgorithm::Snappy;
	} else if leaf.flags.contains(Flags::LZ4_COMPRESSED) {
		leaf.compress = CompressMode::Detect;
		leaf.compression_algo = CompressionAlgorithm::LZ4;
	}

	// add extra flags
	leaf.flags = Flags::from_bits(flags);
	Some(leaf)
}

/// Appends a new `v_builder_leaf` from a buffer
#[no_mangle]
pub extern "C" fn add_leaf_from_buffer(
	ctx: *mut v_builder_ctx, id: *const ffi::c_char, data: *const u8, len: usize, flags: u32, error_p: *mut i32,
) {
	if data.is_null() {
		errors::report::<()>(error_p, errors::E_PARAMETER_IS_NULL);
	} else {
		let Some(leaf) = new_leaf(id, unsafe { slice::from_raw_parts(data, len) }, flags) else {
			errors::report::<()>(error_p, errors::E_INVALID_UTF8);
			return;
		};

		// acquire ctx
		let ctx = unsafe { (ctx as *mut _builder_ctx_inner).as_mut() };
		let Some((_, leaves)) = ctx else {
			errors::report::<()>(error_p, errors::E_PARAMETER_IS_NULL);
			return;
		};

		leaves.push(leaf);
	};
}

/// Creates a new `v_builder_leaf` from a file
#[no_mangle]
pub extern "C" fn add_leaf_from_file(
	ctx: *mut v_builder_ctx, id: *const ffi::c_char, path: *const ffi::c_char, flags: u32, error_p: *mut i32,
) {
	if path.is_null() {
		errors::report::<()>(error_p, errors::E_PARAMETER_IS_NULL);
	} else {
		let leaf = match unsafe { std::ffi::CStr::from_ptr(path).to_str() } {
			Ok(path) => {
				let file = fs::File::open(path).unwrap();
				new_leaf(id, file, flags)
			},
			Err(_) => {
				errors::report::<()>(error_p, errors::E_INVALID_UTF8);
				return;
			},
		};

		// append leaf
		let Some(leaf) = leaf else {
			errors::report::<()>(error_p, errors::E_INVALID_UTF8);
			return;
		};

		let ctx = unsafe { (ctx as *mut _builder_ctx_inner).as_mut() };
		let Some((_, leaves)) = ctx else {
			errors::report::<()>(error_p, errors::E_PARAMETER_IS_NULL);
			return;
		};

		leaves.push(leaf);
	}
}

/// Archive Builder Configuration, use `libffcall` to construct closures in C
pub type v_builder_callback =
	extern "C" fn(id: *const ffi::c_char, id_len: usize, data: *const ffi::c_char, len: usize, location: u64);

/// process context and dump to a preallocated buffer, buffer must at least be big enough to fit data
#[no_mangle]
pub extern "C" fn dump_archive_to_buffer(
	ctx: *mut v_builder_ctx, buffer: *mut u8, buf_size: usize, callback: v_builder_callback, error_p: *mut i32,
) -> u64 {
	let slice = unsafe { slice::from_raw_parts_mut(buffer, buf_size) };
	let target = io::Cursor::new(slice);

	let config = unsafe { (ctx as *mut _builder_ctx_inner).as_mut() };
	let Some((config, leaves)) = config else {
		return errors::report::<()>(error_p, errors::E_PARAMETER_IS_NULL) as _;
	};

	// check if callback is NULL
	let mut cb = move |entry: &RegistryEntry, data: &[u8]| {
		let id = entry.id.as_ref();

		callback(
			id.as_ptr() as _,
			id.len(),
			data.as_ptr() as _,
			data.len(),
			entry.location,
		)
	};

	let wrapper = ((callback as usize) == 0).then_some(&mut cb as _);

	// write
	match dump(target, leaves, config, wrapper) {
		Ok(written) => written,
		Err(e) => errors::v_error_to_id::<()>(error_p, e) as _,
	}
}

/// processed context and write to a file on disk
#[no_mangle]
pub extern "C" fn dump_leaves_to_file(
	ctx: *mut v_builder_ctx, path: *const ffi::c_char, callback: v_builder_callback, error_p: *mut i32,
) -> u64 {
	let path = unsafe { ffi::CStr::from_ptr(path) }.to_str();
	let Ok(path) = path else {
		return errors::report::<()>(error_p, errors::E_INVALID_UTF8) as _;
	};

	let config = unsafe { (ctx as *mut _builder_ctx_inner).as_mut() };
	let Some((config, leaves)) = config else {
		return errors::report::<()>(error_p, errors::E_PARAMETER_IS_NULL) as _;
	};

	// check if callback is NULL
	let mut cb = move |entry: &RegistryEntry, data: &[u8]| {
		let id = entry.id.as_ref();

		callback(
			id.as_ptr() as _,
			id.len(),
			data.as_ptr() as _,
			data.len(),
			entry.location,
		)
	};

	let wrapper = ((callback as usize) == 0).then_some(&mut cb as _);

	// write
	let target = fs::File::create(path).unwrap();
	match dump(target, leaves, config, wrapper) {
		Ok(bytes_written) => bytes_written,
		Err(e) => errors::v_error_to_id::<()>(error_p, e) as _,
	}
}
