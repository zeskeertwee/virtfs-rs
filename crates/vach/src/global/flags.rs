use std::fmt;
use super::error::*;

/// Abstracted flag access and manipulation `struct`.
/// Basically just a tiny, [`bitflags`](https://github.com/bitflags/bitflags)
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Flags {
	pub(crate) bits: u32,
}

impl Flags {
	/// Bits reserved for internal use.
	pub const RESERVED_MASK: u32 = 0b1111_1111_1111_1111_0000_0000_0000_0000;
	/// The size in bytes of any flags entry
	pub const BYTES: usize = 4;

	/// This flag shows that the adjacent entry is compressed
	pub const COMPRESSED_FLAG: u32 = 0b_1000_0000_0000_0000_0000_0000_0000_0000;
	/// This entry was compressed using the [LZ4](https://crates.io/crates/lz4_flex) scheme for very fast decompression with average compression ratios
	pub const LZ4_COMPRESSED: u32 = 0b_0100_0000_0000_0000_0000_0000_0000_0000;
	/// This entry was compressed using the [snappy](https://crates.io/crates/snap) scheme for balanced compression properties
	pub const SNAPPY_COMPRESSED: u32 = 0b_0010_0000_0000_0000_0000_0000_0000_0000;
	/// This entry was compressed using the [brotli](https://crates.io/crates/brotli) scheme for higher compression ratios but slower compression speed
	pub const BROTLI_COMPRESSED: u32 = 0b_0001_0000_0000_0000_0000_0000_0000_0000;

	/// The flag that denotes that the archive source has signatures
	pub const SIGNED_FLAG: u32 = 0b_0000_1000_0000_0000_0000_0000_0000_0000;
	/// The flag that shows data in the leaf in encrypted
	pub const ENCRYPTED_FLAG: u32 = 0b_0000_0010_0000_0000_0000_0000_0000_0000;

	/// Construct a `Flags` struct from a `u32` number
	#[inline(always)]
	pub fn from_bits(bits: u32) -> Self {
		Flags { bits }
	}

	/// Returns a copy of the underlying number.
	#[inline(always)]
	pub fn bits(&self) -> u32 {
		self.bits
	}

	/// Create a new empty instance
	#[inline(always)]
	pub fn new() -> Self {
		Flags { bits: 0 }
	}

	/// Set a bit into the underlying [`u32`], will fail if set into the reserved mask.
	/// The `toggle` parameter specifies whether to insert the flags (when true), or to pop the flag, (when false).
	pub fn set(&mut self, bit: u32, toggle: bool) -> InternalResult<u32> {
		if (Flags::RESERVED_MASK & bit) != 0 {
			return Err(InternalError::RestrictedFlagAccessError);
		} else {
			self.force_set(bit, toggle)
		}

		Ok(self.bits)
	}

	pub(crate) fn force_set(&mut self, mask: u32, toggle: bool) {
		if toggle {
			self.bits |= mask;
		} else {
			self.bits &= !mask;
		}
	}

	#[inline(always)]
	/// Checks whether the given flag is set.
	pub fn contains(&self, bit: u32) -> bool {
		(self.bits & bit) != 0
	}
}

#[rustfmt::skip]
impl fmt::Display for Flags {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let compressed = if self.contains(Flags::COMPRESSED_FLAG) { 'C' } else { '-' };
		let signed = if self.contains(Flags::SIGNED_FLAG) { 'S' } else { '-' };
		let encrypted = if self.contains(Flags::ENCRYPTED_FLAG) { 'E' } else { '-' };

		write!(f, "Flags[{}{}{}]", compressed, encrypted, signed)
	}
}

#[rustfmt::skip]
impl fmt::Debug for Flags {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let compressed = if self.contains(Flags::COMPRESSED_FLAG) { 'C' } else { '-' };
		let signed = if self.contains(Flags::SIGNED_FLAG) { 'S' } else { '-' };
		let encrypted = if self.contains(Flags::ENCRYPTED_FLAG) { 'E' } else { '-' };

		write!(
			f,
			"Flags[{}{}{}]: <{}u32 : {:#032b}>",
			compressed, encrypted, signed, self.bits, self.bits
		)
	}
}
