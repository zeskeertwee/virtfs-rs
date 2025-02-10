#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::or_fun_call)]
#![allow(clippy::manual_map)]
#![deny(missing_docs)]

/*!
![GitHub last commit](https://img.shields.io/github/last-commit/zeskeertwee/vach?logo=rust&logoColor=orange&style=flat-square)

A simple archive format, in Pure Rust.

### 🔫 Cargo Features
- `archive`: Enables the Archive loader.
- `builder`: Enables the Archive builder.
- `multithreaded`: [`dump`](builder::dump) processes leaves in parallel, number of threads can be set manually using [`num_threads`](crate::builder::BuilderConfig::num_threads).
- `compression`: Pulls `snap`, `lz4_flex` and `brotli` as dependencies and enables compression.
- `crypto`: Enables encryption and authentication by pulling the `ed25519_dalek` and `aes_gcm` crates
- `default`: Enables the `archive` and `builder` features.
- `all`: Enables all the above features.

### 🀄 Show me some code _dang it!_

```
use std::{io::Cursor, fs::File};
use vach::prelude::*;

// collect leaves in a vector, or static buffer
let mut leaves = [
	// Leaf::new(File::open("background.wav").unwrap(), "ambient"),
	Leaf::new([12, 23, 34, 45, 56, 67, 78, 90, 69].as_slice(), "ftstep").compress(CompressMode::Always),
	Leaf::new(b"Hello, Cassandra!".as_slice(), "hello")
];

// let mut target = File::create("sounds.vach")?;
let mut target = Cursor::new(Vec::new());

let config = BuilderConfig::default();
let bytes_written = dump(&mut target, &mut leaves, &config, None).unwrap();

// roundtrip
let mut archive = Archive::new(target).unwrap();
let resource = archive.fetch_mut("ftstep").unwrap();

assert_eq!(resource.data.as_ref(), [12, 23, 34, 45, 56, 67, 78, 90, 69].as_slice());
```
*/

/// All tests are included in this module.
mod tests;

pub(crate) mod global;

#[cfg(feature = "archive")]
#[cfg_attr(docsrs, doc(cfg(feature = "archive")))]
pub(crate) mod loader;

#[cfg(feature = "builder")]
#[cfg_attr(docsrs, doc(cfg(feature = "builder")))]
pub(crate) mod writer;

/// Current [`vach`](crate) spec version. increments by ten with every spec change
pub const VERSION: u16 = 30;

/// Size of a secret key
pub const SECRET_KEY_LENGTH: usize = 32;

/// Size of a public key
pub const PUBLIC_KEY_LENGTH: usize = 32;

/// Size of a signature
pub const SIGNATURE_LENGTH: usize = 64;

/// Maximum size for any ID, ie u16::MAX
pub const MAX_ID_LENGTH: usize = u16::MAX as usize;

/// Magic Sequence used by `vach`: "VfACH"
pub const MAGIC: [u8; crate::MAGIC_LENGTH] = *b"VfACH";
pub(crate) const MAGIC_LENGTH: usize = 5;

/// Consolidated crate imports.
pub mod prelude {
	pub use crate::global::{error::*, flags::Flags, reg_entry::RegistryEntry};

	#[cfg(feature = "crypto")]
	pub use crate::crypto::*;

	#[cfg(feature = "archive")]
	pub use crate::archive::*;

	#[cfg(feature = "builder")]
	pub use crate::builder::*;

	#[cfg(feature = "compression")]
	pub use crate::global::compressor::*;
}

/// Import keypairs and signatures from here, mirrors from `ed25519_dalek`
pub mod crypto;

/// Archive Creation logic and data structures, [`dump`](crate::builder::dump), [`Leaf`](crate::builder::Leaf) and [`BuilderConfig`](crate::builder::BuilderConfig)
#[cfg(feature = "builder")]
#[cfg_attr(docsrs, doc(cfg(feature = "builder")))]
pub mod builder {
	pub use crate::writer::*;
	pub use crate::global::{error::*, flags::Flags};

	#[cfg(feature = "compression")]
	pub use crate::global::compressor::CompressionAlgorithm;
}

/// Archive Reading logic and data-structures, [`Archive`](crate::archive::Archive), [`Resource`](crate::archive::Resource)
#[cfg(feature = "archive")]
#[cfg_attr(docsrs, doc(cfg(feature = "archive")))]
pub mod archive {
	pub use crate::loader::{archive::Archive, resource::Resource};
	pub use crate::global::{reg_entry::RegistryEntry, error::*, flags::Flags};
	#[cfg(feature = "compression")]
	pub use crate::global::compressor::CompressionAlgorithm;
}

/// Some utility functions to keep you happy
pub mod crypto_utils;
