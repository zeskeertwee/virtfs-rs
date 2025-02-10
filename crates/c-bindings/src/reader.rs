use std::{fs, io, slice, ffi};
use vach::{crypto_utils::read_verifying_key, prelude::*};
use super::errors;

/// Verifying and Decryption Key
pub type v_verifying_key = ffi::c_void;

/// Create new loader configuration
#[no_mangle]
pub extern "C" fn new_verifying_key(
	vk_bytes: *const [u8; super::V_VERIFYING_KEY_LENGTH], error_p: *mut i32,
) -> *mut v_verifying_key {
	if let Some(bytes) = unsafe { vk_bytes.as_ref() } {
		match read_verifying_key(bytes.as_slice()) {
			Ok(vk) => Box::into_raw(Box::new(vk)) as _,
			Err(e) => errors::v_error_to_id(error_p, e),
		}
	} else {
		errors::report(error_p, errors::E_PARAMETER_IS_NULL)
	}
}

/// Free archive loader configuration
#[no_mangle]
pub extern "C" fn free_verifying_key(config: *mut v_verifying_key) {
	if let Some(c) = unsafe { (config as *mut VerifyingKey).as_mut() } {
		drop(unsafe { Box::from_raw(c) });
	}
}

/// A wrapper combining `fs::File` and `io::Cursor`, over the C boundary.
/// Allowing both inner buffers and files to be used for data.
pub(crate) enum ArchiveInner {
	File(fs::File),
	Buffer(io::Cursor<&'static [u8]>),
}

impl io::Read for ArchiveInner {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		match self {
			ArchiveInner::File(f) => f.read(buf),
			ArchiveInner::Buffer(b) => b.read(buf),
		}
	}
}

impl io::Seek for ArchiveInner {
	fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
		match self {
			ArchiveInner::File(f) => f.seek(pos),
			ArchiveInner::Buffer(b) => b.seek(pos),
		}
	}
}

/// An Archive instance, bound to either a file or a buffer
pub type v_archive = ffi::c_void;

/// Create a new archive from a file
#[no_mangle]
pub extern "C" fn new_archive_from_file(
	path: *const ffi::c_char, config: *const v_verifying_key, error_p: *mut i32,
) -> *mut v_archive {
	let path = match unsafe { std::ffi::CStr::from_ptr(path).to_str() } {
		Ok(p) => p,
		Err(_) => return errors::report(error_p, errors::E_INVALID_UTF8),
	};

	let file = match fs::File::open(path) {
		Ok(file) => file,
		Err(e) => return errors::v_error_to_id(error_p, InternalError::IOError(e)),
	};

	let vk = unsafe { (config as *const VerifyingKey).as_ref() };
	let archive = match vk {
		Some(vk) => Archive::with_key(ArchiveInner::File(file), vk),
		None => Archive::new(ArchiveInner::File(file)),
	};

	let archive = match archive {
		Ok(archive) => archive,
		Err(err) => return errors::v_error_to_id(error_p, err),
	};

	Box::into_raw(Box::new(archive)) as _
}

/// Create a new archive from a buffer
#[no_mangle]
pub extern "C" fn new_archive_from_buffer(
	config: *const v_verifying_key, data: *const u8, len: usize, error_p: *mut i32,
) -> *mut v_archive {
	if data.is_null() {
		return errors::report(error_p, errors::E_PARAMETER_IS_NULL);
	}

	let source = unsafe { slice::from_raw_parts(data, len) };
	let buffer = io::Cursor::new(source);

	let vk = unsafe { (config as *const VerifyingKey).as_ref() };

	let archive = match vk {
		Some(vk) => Archive::with_key(ArchiveInner::Buffer(buffer), vk),
		None => Archive::new(ArchiveInner::Buffer(buffer)),
	};

	let archive = match archive {
		Ok(archive) => archive,
		Err(err) => return errors::v_error_to_id(error_p, err),
	};

	Box::into_raw(Box::new(archive)) as _
}

#[no_mangle]
pub extern "C" fn free_archive(archive: *mut v_archive) {
	if let Some(a) = unsafe { (archive as *mut Archive<ArchiveInner>).as_mut() } {
		drop(unsafe { Box::from_raw(a) });
	}
}

/// A list archive entry IDs
#[repr(C)]
pub struct v_entries {
	count: usize,
	paths: *mut *mut ffi::c_char,
}

/// Get a list of archive entry IDs
#[no_mangle]
pub extern "C" fn archive_get_entries(archive: *const v_archive, error_p: *mut i32) -> *mut v_entries {
	let archive = match unsafe { (archive as *const Archive<ArchiveInner>).as_ref() } {
		Some(a) => a,
		None => return errors::report(error_p, errors::E_PARAMETER_IS_NULL),
	};

	let paths = archive
		.entries()
		.keys()
		.map(|k| ffi::CString::new(k.as_bytes()).unwrap().into_raw())
		.collect::<Vec<_>>();

	let list = paths.into_boxed_slice();

	let entries = v_entries {
		count: list.len(),
		paths: Box::leak(list).as_mut_ptr(),
	};

	Box::into_raw(Box::new(entries))
}

#[no_mangle]
pub extern "C" fn free_entries(entries: *mut v_entries) {
	if !entries.is_null() {
		let entries = unsafe { Box::from_raw(entries) };

		// reallocate box
		let slice = unsafe { slice::from_raw_parts_mut(entries.paths, entries.count) };
		let list = unsafe { Box::from_raw(slice) };

		for entry in list {
			drop(unsafe { ffi::CString::from_raw(entry) });
		}
	}
}

/// An archive resource
#[repr(C)]
pub struct v_resource {
	data: *mut u8,
	len: usize,
	flags: ffi::c_uint,
	content_version: u8,
	verified: bool,
}

/// Fetch a resource, WITHOUT locking the internal Mutex
#[no_mangle]
pub extern "C" fn archive_fetch_resource(
	archive: *mut v_archive, id: *const ffi::c_char, error_p: *mut i32,
) -> *mut v_resource {
	let path = match unsafe { std::ffi::CStr::from_ptr(id).to_str() } {
		Ok(p) => p,
		Err(_) => return errors::report(error_p, errors::E_INVALID_UTF8),
	};

	let archive = match unsafe { (archive as *mut Archive<ArchiveInner>).as_mut() } {
		Some(a) => a,
		None => return errors::report(error_p, errors::E_PARAMETER_IS_NULL),
	};

	let resource = match archive.fetch_mut(path) {
		Ok(resource) => resource,
		Err(err) => return errors::v_error_to_id(error_p, err),
	};

	let resource = v_resource {
		len: resource.data.len(),
		data: Box::leak(resource.data).as_mut_ptr(),
		flags: resource.flags.bits(),
		content_version: resource.content_version,
		verified: resource.verified,
	};

	Box::into_raw(Box::new(resource))
}

/// Fetch a resource, LOCKS the internal Mutex. For use in multithreaded environments
#[no_mangle]
pub extern "C" fn archive_fetch_resource_lock(
	archive: *const v_archive, id: *const i8, error_p: *mut i32,
) -> *mut v_resource {
	let path = match unsafe { std::ffi::CStr::from_ptr(id).to_str() } {
		Ok(p) => p,
		Err(_) => return errors::report(error_p, errors::E_INVALID_UTF8),
	};

	let archive = match unsafe { (archive as *const Archive<ArchiveInner>).as_ref() } {
		Some(a) => a,
		None => return errors::report(error_p, errors::E_PARAMETER_IS_NULL),
	};

	let resource = match archive.fetch(path) {
		Ok(resource) => resource,
		Err(err) => return errors::v_error_to_id(error_p, err),
	};

	let resource = v_resource {
		len: resource.data.len(),
		data: Box::leak(resource.data).as_mut_ptr(),
		flags: resource.flags.bits(),
		content_version: resource.content_version,
		verified: resource.verified,
	};

	Box::into_raw(Box::new(resource))
}

#[no_mangle]
pub extern "C" fn free_resource(resource: *mut v_resource) {
	if let Some(resource) = unsafe { resource.as_mut() } {
		let resource = unsafe { Box::from_raw(resource) };

		let data = unsafe { slice::from_raw_parts_mut(resource.data, resource.len) };
		drop(unsafe { Box::from_raw(data) });
	}
}
