use crate::global::{reg_entry::RegistryEntry, flags::Flags};
use crate::global::error::InternalResult;

#[cfg(feature = "compression")]
use crate::global::compressor::{CompressionAlgorithm, Compressor};

#[cfg(feature = "crypto")]
use crate::crypto::Encryptor;

#[cfg(not(feature = "crypto"))]
type Encryptor = ();

use std::{fmt, io::Read, sync::Arc};

/// Configures how a [`Leaf`] should be compressed.
#[derive(Debug, Clone, Copy, Default)]
#[cfg(feature = "compression")]
#[cfg_attr(docsrs, doc(cfg(feature = "compression")))]
pub enum CompressMode {
	/// The data is never compressed and is embedded as is.
	#[default]
	Never,
	/// The data will always be compressed
	Always,
	/// The compressed data is used, only if it is smaller than the original data.
	Detect,
}

/// A named ([`ID`](Leaf::id)) wrapper around an [`io::Read`](Read) handle, tagged with extra metadata.
pub struct Leaf<'a> {
	/// Boxed read handle
	pub handle: Box<dyn Read + Send + Sync + 'a>,

	/// The `ID` under which the embedded data will be referenced
	pub id: Arc<str>,
	/// The version of the content, allowing you to track obsolete data.
	pub content_version: u8,
	/// The flags that will go into the archive write target.
	pub flags: Flags,

	/// How a [`Leaf`] should be compressed
	#[cfg(feature = "compression")]
	#[cfg_attr(docsrs, doc(cfg(feature = "compression")))]
	pub compress: CompressMode,
	/// The specific compression algorithm to use
	#[cfg_attr(docsrs, doc(cfg(feature = "compression")))]
	#[cfg(feature = "compression")]
	pub compression_algo: CompressionAlgorithm,

	/// Use encryption when writing into the target.
	#[cfg(feature = "crypto")]
	#[cfg_attr(docsrs, doc(cfg(feature = "crypto")))]
	pub encrypt: bool,
	/// Whether to include a signature with this [`Leaf`]
	#[cfg(feature = "crypto")]
	#[cfg_attr(docsrs, doc(cfg(feature = "crypto")))]
	pub sign: bool,
}

impl<'a> Leaf<'a> {
	#[inline(always)]
	/// Creates a new [`Leaf`] wrapping around the given [`Read`] handle, with an ID
	pub fn new<R: Read + Send + Sync + 'a, S: AsRef<str>>(handle: R, id: S) -> Leaf<'a> {
		Leaf {
			handle: Box::new(handle),
			id: Arc::from(id.as_ref()),
			..Default::default()
		}
	}

	/// Consume the [Leaf] and return the underlying Boxed handle
	pub fn into_inner(self) -> Box<dyn Read + Send + 'a> {
		self.handle
	}

	/// Copy all fields from another [`Leaf`], except for `handle` and `id`.
	pub fn template(self, other: &Leaf<'a>) -> Self {
		Leaf {
			handle: self.handle,
			id: self.id,
			..*other
		}
	}

	/// Setter for the [`compress`](Leaf::compress) field
	#[cfg(feature = "compression")]
	#[cfg_attr(docsrs, doc(cfg(feature = "compression")))]
	pub fn compress(mut self, compress: CompressMode) -> Self {
		self.compress = compress;
		self
	}

	/// Setter for the [`content_version`](Leaf::content_version) field
	pub fn version(mut self, content_version: u8) -> Self {
		self.content_version = content_version;
		self
	}

	/// Setter for the [`flags`](crate::builder::Flags) field
	pub fn flags(mut self, flags: Flags) -> Self {
		self.flags = flags;
		self
	}

	/// Setter for the [`encrypt`](Leaf::encrypt) field
	#[cfg(feature = "crypto")]
	pub fn encrypt(mut self, encrypt: bool) -> Self {
		self.encrypt = encrypt;
		self
	}

	/// Setter for the [`sign`](Leaf::sign) field
	#[cfg(feature = "crypto")]
	pub fn sign(mut self, sign: bool) -> Self {
		self.sign = sign;
		self
	}

	/// Setter for the [`compression_algo`](Leaf::compression_algo) field
	#[cfg(feature = "compression")]
	pub fn compression_algo(mut self, compression_algo: CompressionAlgorithm) -> Self {
		self.compression_algo = compression_algo;
		self
	}
}

impl<'a> Default for Leaf<'a> {
	/// The default leaf holds no bytes at all, this is expected to be used as a stencil|template.
	#[inline(always)]
	fn default() -> Leaf<'a> {
		Leaf {
			handle: Box::<&[u8]>::new(&[]),

			id: Arc::from(""),
			flags: Default::default(),
			content_version: Default::default(),

			#[cfg(feature = "crypto")]
			encrypt: Default::default(),
			#[cfg(feature = "crypto")]
			sign: Default::default(),

			#[cfg(feature = "compression")]
			compress: Default::default(),
			#[cfg(feature = "compression")]
			compression_algo: Default::default(),
		}
	}
}

impl fmt::Debug for Leaf<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut d = f.debug_struct("Leaf");
		d.field("id", &self.id)
			.field("content_version", &self.content_version)
			.field("flags", &self.flags);

		#[cfg(feature = "crypto")]
		{
			d.field("encrypt", &self.encrypt);
			d.field("sign", &self.sign);
		}

		#[cfg(feature = "compression")]
		{
			d.field("compress", &self.compress);
			d.field("compression_algo", &self.compression_algo);
		}

		d.finish()
	}
}

impl From<&mut Leaf<'_>> for RegistryEntry {
	fn from(leaf: &mut Leaf<'_>) -> Self {
		RegistryEntry {
			id: leaf.id.clone(),
			flags: leaf.flags,
			content_version: leaf.content_version,
			..RegistryEntry::empty()
		}
	}
}

// Processed data ready to be inserted into a `Write + Clone` target during Building
pub(crate) struct ProcessedLeaf {
	pub(crate) data: Vec<u8>,
	pub(crate) entry: RegistryEntry,
	#[cfg(feature = "crypto")]
	pub(crate) sign: bool,
}

// Process Leaf into Prepared Data, externalised for multithreading purposes
#[inline(never)]
pub(crate) fn process_leaf(leaf: &mut Leaf<'_>, _encryptor: Option<&Encryptor>) -> InternalResult<ProcessedLeaf> {
	let mut entry: RegistryEntry = leaf.into();
	let mut raw = Vec::new();

	// Compression comes first
	#[cfg(feature = "compression")]
	match leaf.compress {
		CompressMode::Never => {
			leaf.handle.read_to_end(&mut raw)?;
		},
		CompressMode::Always => {
			Compressor::new(&mut leaf.handle).compress(leaf.compression_algo, &mut raw)?;

			entry.flags.force_set(Flags::COMPRESSED_FLAG, true);
			entry.flags.force_set(leaf.compression_algo.into(), true);
		},
		CompressMode::Detect => {
			let mut buffer = Vec::new();
			leaf.handle.read_to_end(&mut buffer)?;

			let mut compressed_data = Vec::new();
			Compressor::new(buffer.as_slice()).compress(leaf.compression_algo, &mut compressed_data)?;

			if compressed_data.len() <= buffer.len() {
				entry.flags.force_set(Flags::COMPRESSED_FLAG, true);
				entry.flags.force_set(leaf.compression_algo.into(), true);

				raw = compressed_data;
			} else {
				buffer.as_slice().read_to_end(&mut raw)?;
			};
		},
	}

	// If the compression feature is turned off, simply reads into buffer
	#[cfg(not(feature = "compression"))]
	{
		use crate::global::error::InternalError;

		if entry.flags.contains(Flags::COMPRESSED_FLAG) {
			return Err(InternalError::MissingFeatureError("compression"));
		};

		leaf.handle.read_to_end(&mut raw)?;
	}

	// Encryption comes second
	#[cfg(feature = "crypto")]
	if leaf.encrypt {
		if let Some(ex) = _encryptor {
			raw = ex.encrypt(&raw)?;
			entry.flags.force_set(Flags::ENCRYPTED_FLAG, true);
		}
	}

	Ok(ProcessedLeaf {
		data: raw,
		entry,
		#[cfg(feature = "crypto")]
		sign: leaf.sign,
	})
}
