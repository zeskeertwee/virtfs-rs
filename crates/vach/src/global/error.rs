use std::{error, io};
use thiserror::Error;

/// Internal `Result` type alias used by `vach`. Basically equal to: `Result<T, InternalError>`
pub type InternalResult<T = ()> = Result<T, InternalError>;

/// All errors manifestable within `vach` collected into a neat enum
#[derive(Debug, Error)]
pub enum InternalError {
	/// Generic Error
	#[error("[VachError::GenericError] {0}")]
	OtherError(Box<dyn error::Error + Send + Sync>),
	/// a necessary cargo feature wasn't enabled for a certain action: eg trying to decompress without the `compression` feature
	#[error("[VachError::MissingFeatureError] Unable to continue with operation, the cargo feature ({0}) is missing")]
	MissingFeatureError(&'static str),
	/// a [`Signature`](crate::crypto::Signature), [`VerifyingKey`](crate::crypto::VerifyingKey) or [`SigningKey`](crate::crypto::SigningKey) failed to deserialize.
	#[error("[VachError::ParseError] {0}")]
	ParseError(String),
	/// thin wrapper over [io::Error](std::io::Error), captures all IO errors
	#[error("[VachError::IOError] {0}")]
	IOError(#[from] io::Error),
	/// invalid MAGIC sequence in the given source, hinting at corruption or possible incompatibility with the given source
	#[error("[VachError::ValidationError] Invalid magic found in Header, possible incompatibility with given source. Magic found {0:?}")]
	MalformedArchiveSource([u8; crate::MAGIC_LENGTH]),
	/// the resource was not found
	#[error("[VachError::MissingResourceError] Resource not found: {0}")]
	MissingResourceError(String),
	/// two leaves found with the same ID, each leaf should have a unique ID
	#[error("[VachError::LeafAppendError] A leaf with the ID: {0} already exists. Consider changing the ID to prevent collisions")]
	DuplicateLeafID(String),
	/// no `Keypair` is provided and an encrypted [Leaf](crate::builder::Leaf) is encountered
	#[error("[VachError::NoKeypairError] Unable to continue with cryptographic operation, as no keypair was supplied")]
	NoKeypairError,
	/// decryption or encryption failed
	#[cfg(feature = "crypto")]
	#[error("[VachError::CryptoError] {0}")]
	CryptoError(aes_gcm::Error),
	/// attempted to set a bit in the reserved bit range, [`Flags::RESERVED_MASK`](crate::global::flags::Flags::RESERVED_MASK)
	#[error("[VachError::RestrictedFlagAccessError] Tried to set reserved bit(s)!")]
	RestrictedFlagAccessError,
	/// a [`Leaf`](crate::builder::Leaf) has an ID that is longer than [`crate::MAX_ID_LENGTH`], contains the overflowing `ID`
	#[error("[VachError::IDSizeOverflowError] The maximum size of any ID is: {}. The leaf with ID: {} has an overflowing ID of length: {}", crate::MAX_ID_LENGTH, .0, .0.len())]
	IDSizeOverflowError(String),
	/// current loader attempted to load an incompatible version, contains the incompatible source's version
	#[error("The provided archive source has version: {}. While the current implementation has a spec-version: {}. The provided source is incompatible!", .0, crate::VERSION)]
	IncompatibleArchiveVersionError(u16),
	/// errors thrown  during compression or decompression
	#[error("[VachError::CompressorDecompressorError]: {0}")]
	#[cfg(feature = "compression")]
	DeCompressionError(#[from] lz4_flex::frame::Error),
}
