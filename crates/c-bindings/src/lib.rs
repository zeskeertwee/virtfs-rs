#![deny(clippy::from_raw_with_void_ptr)]
#![allow(non_camel_case_types)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

mod errors;
pub mod reader;
pub mod writer;

// TODO: Document usage

/// The version of the library
#[no_mangle]
pub extern "C" fn version() -> u16 {
	vach::VERSION
}

/// The length of a public key
pub const V_VERIFYING_KEY_LENGTH: usize = 32;
/// The length of a secret
pub const V_SECRET_KEY_LENGTH: usize = 32;
