use std::io::Read;
use super::{error::*, flags::Flags};

#[derive(Debug)]
pub(crate) struct Header {
	pub(crate) magic: [u8; crate::MAGIC_LENGTH],
	pub flags: Flags,
	pub version: u16,
	pub capacity: u16,
}

impl Default for Header {
	#[inline(always)]
	fn default() -> Header {
		Header {
			magic: crate::MAGIC,
			flags: Flags::default(),
			version: crate::VERSION,
			capacity: 0,
		}
	}
}

impl Header {
	pub const BASE_SIZE: usize = crate::MAGIC_LENGTH + Flags::BYTES + Self::VERSION_SIZE + Self::CAPACITY_SIZE;

	// Data appears in this order
	pub const VERSION_SIZE: usize = 2;
	pub const CAPACITY_SIZE: usize = 2;

	/// Validates this Header's MAGIC and ARCHIVE_VERSION
	pub(crate) fn validate(&self) -> InternalResult {
		// Validate magic
		if self.magic != crate::MAGIC {
			return Err(InternalError::MalformedArchiveSource(self.magic));
		};

		// Validate version
		if crate::VERSION != self.version {
			return Err(InternalError::IncompatibleArchiveVersionError(self.version));
		};

		Ok(())
	}

	pub(crate) fn from_handle<T: Read>(mut handle: T) -> InternalResult<Header> {
		let mut buffer: [u8; Header::BASE_SIZE] = [0u8; Header::BASE_SIZE];
		handle.read_exact(&mut buffer)?;

		// Construct header
		Ok(Header {
			// Read magic, [u8;5]
			magic: buffer[0..crate::MAGIC_LENGTH].try_into().unwrap(),
			// Read flags, u32 from [u8;4]
			flags: Flags::from_bits(u32::from_le_bytes(buffer[crate::MAGIC_LENGTH..9].try_into().unwrap())),
			// Read version, u16 from [u8;2]
			version: u16::from_le_bytes(buffer[9..11].try_into().unwrap()),
			// Read the capacity of the archive, u16 from [u8;2]
			capacity: u16::from_le_bytes(buffer[11..13].try_into().unwrap()),
		})
	}

	pub(crate) fn to_bytes(&self) -> [u8; Header::BASE_SIZE] {
		let mut buffer: [u8; Header::BASE_SIZE] = [0u8; Header::BASE_SIZE];
		buffer[0..crate::MAGIC_LENGTH].copy_from_slice(&self.magic);
		buffer[crate::MAGIC_LENGTH..9].copy_from_slice(&self.flags.bits().to_le_bytes());
		buffer[9..11].copy_from_slice(&self.version.to_le_bytes());
		buffer[11..13].copy_from_slice(&self.capacity.to_le_bytes());
		buffer
	}
}
