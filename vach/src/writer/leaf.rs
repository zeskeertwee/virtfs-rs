use crate::{
	global::{reg_entry::RegistryEntry, types::Flags},
};
use std::{io::Read};

#[derive(Clone, Copy)]
pub enum CompressMode {
	Always,
	Detect,
	Never,
}

pub struct Leaf<'a> {
	// This lifetime simply reflects to the `Builder`'s lifetime, meaning the handle must live longer than or the same as the Builder
	pub handle: Box<dyn Read + 'a>,
	pub id: String,
	pub content_version: u8,
	pub compress: CompressMode,
	pub flags: Flags,
}

impl<'a> Default for Leaf<'a> {
	#[inline(always)]
	fn default() -> Leaf<'a> {
		Leaf {
			handle: Box::<&[u8]>::new(&[]),
			id: String::new(),
			content_version: 0,
			compress: CompressMode::Detect,
			flags: Flags::default(),
		}
	}
}

impl<'a> Leaf<'a> {
	#[inline(always)]
	pub fn from_handle(handle: impl Read + 'a) -> anyhow::Result<Leaf<'a>> {
		Ok(Leaf {
			handle: Box::new(handle),
			..Default::default()
		})
	}
	pub(crate) fn to_registry_entry(&self) -> RegistryEntry {
		let mut entry = RegistryEntry::empty();
		entry.content_version = self.content_version;
		entry.flags = self.flags;
		entry
	}

	pub fn template(mut self, other: &Leaf) -> Self {
		self.compress = other.compress;
		self.content_version = other.content_version;
		self.flags = other.flags;
		self
	}
	pub fn compress(mut self, compress: CompressMode) -> Self {
		self.compress = compress;
		self
	}
	pub fn version(mut self, version: u8) -> Self {
		self.content_version = version;
		self
	}
	pub fn id(mut self, id: &str) -> Self {
		self.id = id.to_string();
		self
	}
	pub fn flags(mut self, flags: Flags) -> Self {
		self.flags = flags;
		self
	}
}
