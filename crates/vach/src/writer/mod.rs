use std::io::{Seek, SeekFrom, Write};

#[cfg(feature = "multithreaded")]
use std::{thread, sync::mpsc};

mod config;
mod leaf;

pub use config::BuilderConfig;
pub use leaf::Leaf;

#[cfg(feature = "compression")]
pub use {leaf::CompressMode, crate::global::compressor::Compressor};

use crate::global::error::*;
use crate::global::{header::Header, reg_entry::RegistryEntry, flags::Flags};

#[cfg(feature = "crypto")]
use {crate::crypto::Encryptor, ed25519_dalek::Signer};

#[cfg(not(feature = "crypto"))]
type Encryptor = ();

/// Counts bytes written to the target
struct WriteCounter<W: Send> {
	bytes: u64,
	inner: W,
}

impl<W: Write + Send> Write for WriteCounter<W> {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		let len = self.inner.write(buf)?;
		self.bytes += len as u64;
		Ok(len)
	}

	fn flush(&mut self) -> std::io::Result<()> {
		self.inner.flush()
	}
}

impl<W: Seek + Send> Seek for WriteCounter<W> {
	fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
		self.inner.seek(pos)
	}
}

/// iterates over all [`Leaf`], processes them and writes the output into the target.
pub fn dump<'a, W: Write + Seek + Send>(
	target: W, leaves: &mut [Leaf<'a>], config: &BuilderConfig,
	mut callback: Option<&mut dyn FnMut(&RegistryEntry, &[u8])>,
) -> InternalResult<u64> {
	// TODO: Move to vectored io
	let mut config = config.clone();
	let mut target = WriteCounter {
		bytes: 0,
		inner: target,
	};

	// find duplicates
	let set = leaves
		.iter()
		.map(|l| l.id.as_ref())
		.collect::<std::collections::HashSet<_>>();

	if set.len() < leaves.len() {
		for (idx, leaf) in leaves.iter().enumerate() {
			let slice = &leaves[idx + 1..];

			// find duplicate
			if slice.iter().any(|l| l.id == leaf.id) {
				return Err(InternalError::DuplicateLeafID(leaf.id.to_string()));
			}
		}
	}

	// Determines the offset at which to start writing leafs
	let mut leaf_offset = {
		leaves
			.iter()
			.map(|leaf| {
				// The size of it's ID, the minimum size of an entry without a signature, and the size of a signature if present
				leaf.id.len() + RegistryEntry::MIN_SIZE + {
					#[cfg(feature = "crypto")]
					if config.signing_key.is_some() && leaf.sign {
						crate::SIGNATURE_LENGTH
					} else {
						0
					}
					#[cfg(not(feature = "crypto"))]
					{
						0
					}
				}
			})
			.sum::<usize>()
			+ Header::BASE_SIZE
	} as u64;

	#[cfg(feature = "crypto")]
	if config.signing_key.is_some() {
		config.flags.force_set(Flags::SIGNED_FLAG, true);
	};

	// write HEADER
	let header = crate::global::header::Header {
		magic: crate::MAGIC,
		flags: config.flags,
		version: crate::VERSION,
		capacity: leaves.len() as u16,
	};

	target.seek(SeekFrom::Start(0))?;
	target.write_all(&header.to_bytes())?;

	// Build encryptor
	#[cfg(feature = "crypto")]
	let encryptor = {
		let use_encryption = leaves.iter().any(|leaf| leaf.encrypt);
		if use_encryption {
			if let Some(keypair) = config.signing_key.as_ref() {
				Some(Encryptor::new(&keypair.verifying_key()))
			} else {
				return Err(InternalError::NoKeypairError);
			}
		} else {
			None
		}
	};

	#[cfg(not(feature = "crypto"))]
	let encryptor = None;

	// buffer registry data
	let mut registry = Vec::with_capacity(leaf_offset as usize - Header::BASE_SIZE);
	target.seek(SeekFrom::Start(leaf_offset))?;

	#[allow(unused_mut)]
	// Callback for processing IO
	let mut write = |result: InternalResult<leaf::ProcessedLeaf>| -> InternalResult<()> {
		let mut result = result?;
		let bytes = result.data.len() as u64;

		// write LEAF
		target.write_all(&result.data)?;

		// update registry entry
		result.entry.location = leaf_offset;
		result.entry.offset = bytes;

		// increment leaf offset
		leaf_offset += result.data.len() as u64;

		// write out registry entry
		#[cfg(feature = "crypto")]
		if result.sign {
			if let Some(keypair) = &config.signing_key {
				result.entry.flags.force_set(Flags::SIGNED_FLAG, true);

				let entry_bytes = result.entry.to_bytes(true)?;
				result.data.extend_from_slice(&entry_bytes);

				// Include registry data in the signature
				result.entry.signature = Some(keypair.sign(&result.data));
			};
		}

		// write to registry buffer, this one might include the Signature
		let entry_bytes = result.entry.to_bytes(false)?;
		registry.write_all(&entry_bytes)?;

		// Call the progress callback bound within the [`BuilderConfig`]
		if let Some(callback) = callback.as_mut() {
			callback(&result.entry, &result.data);
		}

		Ok(())
	};

	#[cfg(feature = "multithreaded")]
	let (tx, rx) = mpsc::sync_channel(leaves.len());

	#[cfg(feature = "multithreaded")]
	if !leaves.is_empty() {
		thread::scope(|s| -> InternalResult<()> {
			let count = leaves.len();

			#[rustfmt::skip]
			// if we have an insane number of threads send leafs in chunks of 8
			let chunk_size = if config.num_threads.get() > count { 8 } else { count / config.num_threads };

			let chunks = leaves.chunks_mut(chunk_size);
			let encryptor = encryptor.as_ref();

			// Spawn CPU threads
			for chunk in chunks {
				let queue = tx.clone();

				s.spawn(move || {
					for leaf in chunk {
						let res = leaf::process_leaf(leaf, encryptor);
						queue.send(res).unwrap();
					}
				});
			}

			// Process IO, read results from
			let mut results = 0;
			loop {
				match rx.try_recv() {
					Ok(r) => {
						results += 1;
						write(r)?
					},
					Err(e) => match e {
						mpsc::TryRecvError::Empty => {
							if results >= count {
								break Ok(());
							}
						},
						mpsc::TryRecvError::Disconnected => break Ok(()),
					},
				}
			}
		})?;
	};

	#[cfg(not(feature = "multithreaded"))]
	leaves
		.iter_mut()
		.map(|l| leaf::process_leaf(l, encryptor.as_ref()))
		.try_for_each(write)?;

	// write UPDATED REGISTRY
	target.seek(SeekFrom::Start(Header::BASE_SIZE as _))?;
	target.write_all(&registry)?;

	Ok(target.bytes)
}
