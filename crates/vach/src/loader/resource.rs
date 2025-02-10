use std::fmt;
use crate::global::flags::Flags;

/// Basically processed data obtained from an archive returned by [`archive.fetch(...)`](crate::archive::Archive::fetch) and [`archive.fetch_mut(...)`](crate::archive::Archive::fetch_mut)
#[derive(Debug)]
pub struct Resource {
	/// The parsed data
	pub data: Box<[u8]>,
	/// The flags extracted from the archive's registry entry
	pub flags: Flags,
	/// The content version of stored in the registry entry
	pub content_version: u8,
	/// If a [`Resource's`](Resource) bytes were signed and the signature check passed
	pub verified: bool,
}

impl fmt::Display for Resource {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"[Resource] size: {size} bytes, content version: {version}, flags: {flags}",
			size = self.data.len(),
			flags = &self.flags,
			version = &self.content_version
		)
	}
}
