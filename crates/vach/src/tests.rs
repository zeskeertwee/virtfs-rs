#![cfg(test)]
// This is meant to mirror as closely as possible, how users should use the crate

// Boring, average every day contemporary imports
use std::{fs::File, str};
use crate::prelude::*;

// Contains both the public key and secret key in the same file:
// secret -> [u8; crate::SECRET_KEY_LENGTH], public -> [u8; crate::PUBLIC_KEY_LENGTH]
#[cfg(feature = "crypto")]
const KEYPAIR: &[u8; crate::SECRET_KEY_LENGTH + crate::PUBLIC_KEY_LENGTH] = include_bytes!("../test_data/pair.pub");

// The paths to the Archives, to be written|loaded
const SIGNED_TARGET: &str = "test_data/signed.vach";
const SIMPLE_TARGET: &str = "test_data/simple.vach";
const ENCRYPTED_TARGET: &str = "test_data/encrypted.vach";

// Custom bitflag tests
const CUSTOM_FLAG_1: u32 = 0b0000_0000_0000_0000_0000_1000_0000_0000;
const CUSTOM_FLAG_2: u32 = 0b0000_0000_0000_0000_0000_0100_0000_0000;
const CUSTOM_FLAG_3: u32 = 0b0000_0000_0000_0000_0000_0000_1000_0000;
const CUSTOM_FLAG_4: u32 = 0b0000_0000_0000_0000_0000_0000_0001_0000;

fn leaves_from_dir<'a>(
	path: impl AsRef<std::path::Path>, template: Option<&Leaf<'a>>,
) -> InternalResult<Vec<Leaf<'a>>> {
	use std::fs;

	let mut leaves = vec![];
	let directory = fs::read_dir(path)?;

	for file in directory {
		let path = file?.path();

		let v = path.iter().map(|u| u.to_string_lossy()).collect::<Vec<_>>();

		if path.is_file() && path.extension().map(|s| s.to_str().unwrap()) != Some("vach") {
			let file = fs::File::open(&path)?;
			let id = v.last().unwrap();

			let leaf = match template {
				Some(t) => Leaf::new(file, id).template(t),
				None => Leaf::new(file, id),
			};

			leaves.push(leaf);
		}
	}

	Ok(leaves)
}

#[test]
#[cfg(feature = "archive")]
fn custom_bitflags() -> InternalResult {
	let target = File::open(SIMPLE_TARGET)?;
	let archive = Archive::new(target)?;

	let entry = archive.fetch_entry("poem").unwrap();
	let flags = entry.flags;

	assert_eq!(flags.bits(), entry.flags.bits());
	assert!(flags.contains(CUSTOM_FLAG_1 | CUSTOM_FLAG_2 | CUSTOM_FLAG_3 | CUSTOM_FLAG_4));

	Ok(())
}

#[test]
fn flag_restricted_access() {
	let mut flag = Flags::from_bits(0b1111_1000_0000_0000);

	// This should return an error
	if let Err(error) = flag.set(Flags::COMPRESSED_FLAG, true) {
		assert!(matches!(error, InternalError::RestrictedFlagAccessError));
	} else {
		panic!("Access to restricted flags has been allowed, this should not be feasible")
	};
}

#[test]
fn flags_set_intersects() {
	let mut flag = Flags::new();

	flag.force_set(Flags::COMPRESSED_FLAG, true);
	assert_eq!(flag.bits(), Flags::COMPRESSED_FLAG);

	flag.force_set(Flags::COMPRESSED_FLAG, true);
	assert_eq!(flag.bits(), Flags::COMPRESSED_FLAG);

	flag.force_set(Flags::SIGNED_FLAG, true);
	assert_eq!(flag.bits(), Flags::COMPRESSED_FLAG | Flags::SIGNED_FLAG);

	flag.force_set(Flags::COMPRESSED_FLAG, false);
	assert_eq!(flag.bits(), Flags::SIGNED_FLAG);

	flag.force_set(Flags::COMPRESSED_FLAG, false);
	assert_eq!(flag.bits(), Flags::SIGNED_FLAG);

	flag.force_set(Flags::COMPRESSED_FLAG | Flags::SIGNED_FLAG, true);
	assert_eq!(flag.bits(), Flags::COMPRESSED_FLAG | Flags::SIGNED_FLAG);
}

#[test]
#[cfg(all(feature = "compression", feature = "builder"))]
fn builder_no_signature() {
	let build_config = BuilderConfig::default();

	let mut poem_flags = Flags::default();
	poem_flags
		.set(CUSTOM_FLAG_1 | CUSTOM_FLAG_2 | CUSTOM_FLAG_3 | CUSTOM_FLAG_4, true)
		.unwrap();

	let mut leaves = [
		Leaf::new(File::open("test_data/song.txt").unwrap(), "song"),
		Leaf::new(File::open("test_data/lorem.txt").unwrap(), "lorem"),
		Leaf::new(File::open("test_data/bee.script").unwrap(), "script"),
		Leaf::new(File::open("test_data/quicksort.wasm").unwrap(), "wasm"),
		Leaf::new(File::open("test_data/poem.txt").unwrap(), "poem")
			.compress(CompressMode::Always)
			.version(10)
			.flags(poem_flags),
		Leaf::new(b"Hello, Cassandra!" as &[u8], "greeting").compress(CompressMode::Never),
	];

	let mut target = File::create(SIMPLE_TARGET).unwrap();
	let written = dump(&mut target, &mut leaves, &build_config, None).unwrap();

	assert_eq!(target.metadata().unwrap().len(), written);
}

#[test]
#[cfg(all(feature = "compression", feature = "archive"))]
fn fetch_no_signature() -> InternalResult {
	let target = File::open(SIMPLE_TARGET)?;

	let mut archive = Archive::new(target)?;
	let resource = archive.fetch_mut("wasm")?;

	assert_eq!(resource.data.len(), 106537);
	assert!(!resource.verified);
	assert!(!resource.flags.contains(Flags::COMPRESSED_FLAG));

	let hello = archive.fetch_mut("greeting")?;
	assert_eq!("Hello, Cassandra!", str::from_utf8(&hello.data).unwrap());
	assert!(!hello.flags.contains(Flags::COMPRESSED_FLAG));

	Ok(())
}

#[test]
#[cfg(all(feature = "builder", feature = "crypto"))]
fn builder_with_signature() -> InternalResult {
	let mut build_config = BuilderConfig::default();
	build_config.load_keypair(KEYPAIR.as_slice())?;

	let mut leaves = leaves_from_dir("test_data", None)?;

	leaves.push(Leaf::new(b"".as_slice(), "not_signed"));
	leaves.push(Leaf::new(b"Don't forget to recite your beatitudes!".as_slice(), "signed").sign(true));

	let mut target = File::create(SIGNED_TARGET)?;
	let written = dump(&mut target, leaves.as_mut_slice(), &build_config, None)?;

	assert_eq!(target.metadata().unwrap().len(), written);
	Ok(())
}

#[test]
#[cfg(all(feature = "archive", feature = "crypto", feature = "compression"))]
fn fetch_with_signature() -> InternalResult {
	use crate::crypto_utils::read_verifying_key;

	// Load keypair
	let keypair = &KEYPAIR[crate::SECRET_KEY_LENGTH..];
	let vk = read_verifying_key(keypair)?;

	// open archive
	let target = File::open(SIGNED_TARGET)?;
	let mut archive = Archive::with_key(target, &vk)?;

	let resource = archive.fetch_mut("quicksort.wasm")?;
	assert_eq!(resource.data.len(), 106537);

	// The adjacent resource was flagged to not be signed
	let not_signed_resource = archive.fetch_mut("not_signed")?;
	assert!(!not_signed_resource.flags.contains(Flags::SIGNED_FLAG));
	assert!(!not_signed_resource.verified);

	let resource = archive.fetch_mut("signed")?;
	assert!(resource.verified);
	assert!(resource.flags.contains(Flags::SIGNED_FLAG));

	Ok(())
}

#[test]
#[cfg(feature = "crypto")]
fn decryptor_test() -> InternalResult {
	use crate::crypto_utils::gen_keypair;

	let vk = gen_keypair().verifying_key();

	let crypt = Encryptor::new(&vk);
	let data = vec![12, 12, 12, 12];

	let ciphertext = crypt.encrypt(&data)?;
	let plaintext = crypt.decrypt(&ciphertext)?;

	assert_ne!(&plaintext, &ciphertext);
	assert_eq!(&plaintext, &data);

	Ok(())
}

#[test]
#[cfg(all(feature = "compression", feature = "builder", feature = "crypto"))]
fn builder_with_encryption() -> InternalResult {
	let mut build_config = BuilderConfig::default();
	build_config.load_keypair(KEYPAIR.as_slice())?;

	let template = Leaf::default().encrypt(true).compress(CompressMode::Never).sign(true);
	let mut leaves = leaves_from_dir("test_data", Some(&template))?;

	leaves.push(
		Leaf::new(b"Snitches get stitches, iOS sucks" as &[u8], "stitches.snitches")
			.sign(false)
			.compression_algo(CompressionAlgorithm::Brotli(11))
			.compress(CompressMode::Always),
	);

	let mut target = File::create(ENCRYPTED_TARGET)?;
	let written = dump(&mut target, leaves.as_mut_slice(), &build_config, None)?;

	assert_eq!(target.metadata().unwrap().len(), written);
	Ok(())
}

#[test]
#[cfg(all(feature = "archive", feature = "crypto", feature = "compression"))]
fn fetch_from_encrypted() -> InternalResult {
	use crate::crypto_utils::read_verifying_key;

	let target = File::open(ENCRYPTED_TARGET)?;

	// Load keypair
	let public_key = &KEYPAIR[crate::SECRET_KEY_LENGTH..];
	let vk = read_verifying_key(public_key)?;

	let mut archive = Archive::with_key(target, &vk)?;

	// read data
	let not_signed = archive.fetch_mut("stitches.snitches")?;
	let data = std::str::from_utf8(&not_signed.data).unwrap();
	assert_eq!(data, "Snitches get stitches, iOS sucks");

	let signed = archive.fetch_mut("quicksort.wasm")?;

	assert_eq!(signed.data.len(), 106537);
	assert!(signed.verified);
	assert!(!signed.flags.contains(Flags::COMPRESSED_FLAG));
	assert!(signed.flags.contains(Flags::ENCRYPTED_FLAG));

	Ok(())
}

#[test]
#[cfg(all(feature = "builder", feature = "archive", feature = "crypto"))]
fn consolidated_test() -> InternalResult {
	use crate::crypto_utils::{gen_keypair, read_keypair};
	use std::{io::Cursor, time::Instant};

	let mut target = Cursor::new(vec![]);

	// Data to be written
	let data_1 = b"Around The World, Fatter wetter stronker" as &[u8];
	let data_2 = b"Imago" as &[u8];
	let data_3 = b"Fast-Acting Long-Lasting, *Bathroom Reader*" as &[u8];

	// Builder definition
	let keypair = gen_keypair();
	let keypair_bytes = keypair.to_keypair_bytes();

	let mut config = BuilderConfig::default();
	config.load_keypair(keypair_bytes.as_slice()).unwrap();

	// Add data
	let template = Leaf::default().encrypt(true).version(59).sign(true);
	let mut leaves = [
		Leaf::new(data_1, "d1").template(&template),
		Leaf::new(data_2, "d2").template(&template),
		Leaf::new(data_3, "d3").template(&template),
	];

	// Dump data
	let then = Instant::now();
	dump(&mut target, &mut leaves, &config, None)?;

	// Just because
	println!("Building took: {:?}", then.elapsed());

	// parse verifying key
	let sk = read_keypair(keypair_bytes.as_slice())?;
	let vk = sk.verifying_key();

	// open archive
	let then = Instant::now();
	let mut archive = Archive::with_key(target, &vk)?;

	println!("Archive initialization took: {:?}", then.elapsed());

	// Quick assertions
	let then = Instant::now();
	assert_eq!(archive.fetch_mut("d1")?.data.as_ref(), data_1);
	assert_eq!(archive.fetch_mut("d2")?.data.as_ref(), data_2);
	assert_eq!(archive.fetch_mut("d3")?.data.as_ref(), data_3);

	println!("Fetching took: {:?} on average", then.elapsed() / 3);

	// All seems ok
	Ok(())
}

#[test]
#[cfg(all(feature = "compression", feature = "builder"))]
fn test_compressors() -> InternalResult {
	use std::io::Cursor;
	const INPUT_LEN: usize = 4096;

	let input = [12u8; INPUT_LEN];
	let mut target = Cursor::new(vec![]);

	let mut leaves = [
		Leaf::new(input.as_slice(), "LZ4")
			.compression_algo(CompressionAlgorithm::LZ4)
			.compress(CompressMode::Always),
		Leaf::new(input.as_slice(), "BROTLI")
			.compression_algo(CompressionAlgorithm::Brotli(9))
			.compress(CompressMode::Always),
		Leaf::new(input.as_slice(), "SNAPPY")
			.compression_algo(CompressionAlgorithm::Snappy)
			.compress(CompressMode::Always),
	];

	let builder_config = BuilderConfig::default();
	dump(&mut target, &mut leaves, &builder_config, None)?;

	let mut archive = Archive::new(&mut target)?;

	let d1 = archive.fetch_mut("LZ4")?;
	let d2 = archive.fetch_mut("BROTLI")?;
	let d3 = archive.fetch_mut("SNAPPY")?;

	// Identity tests
	assert_eq!(d1.data.len(), INPUT_LEN);
	assert_eq!(d2.data.len(), INPUT_LEN);
	assert_eq!(d3.data.len(), INPUT_LEN);

	assert!(&d1.data[..] == &input);
	assert!(&d2.data[..] == &input);
	assert!(&d3.data[..] == &input);

	// Compression tests
	assert!(archive.fetch_entry("LZ4").unwrap().offset < INPUT_LEN as u64);
	assert!(archive.fetch_entry("BROTLI").unwrap().offset < INPUT_LEN as u64);
	assert!(archive.fetch_entry("SNAPPY").unwrap().offset < INPUT_LEN as u64);

	// A simple test to show that these are somehow not the same data
	assert!(archive.fetch_entry("SNAPPY").unwrap().offset != archive.fetch_entry("LZ4").unwrap().offset);
	assert!(archive.fetch_entry("BROTLI").unwrap().offset != archive.fetch_entry("LZ4").unwrap().offset);
	assert!(archive.fetch_entry("SNAPPY").unwrap().offset != archive.fetch_entry("BROTLI").unwrap().offset);

	Ok(())
}

#[test]
#[cfg(all(feature = "multithreaded", feature = "archive"))]
fn test_batch_fetching() -> InternalResult {
	use std::{io::Cursor, collections::HashMap};
	use rayon::prelude::*;

	// Define input constants
	const INPUT_LEN: usize = 8;
	const INPUT: [u8; INPUT_LEN] = [69u8; INPUT_LEN];

	let mut target = Cursor::new(vec![]);

	// Define and queue data
	let mut ids = (0..120).map(|i| format!("ID {}", i)).collect::<Vec<_>>();
	let mut leaves = ids.iter().map(|i| Leaf::new(&INPUT[..], i)).collect::<Vec<_>>();

	ids.push("ERRORS".to_string());

	// Process data
	let config = BuilderConfig::default().threads(2);
	dump(&mut target, leaves.as_mut_slice(), &config, None)?;

	let archive = Archive::new(target)?;
	let mut resources = ids
		.as_slice()
		.par_iter()
		.map(|id| (id.as_str(), archive.fetch(&id)))
		.collect::<HashMap<_, _>>();

	// Tests and checks
	assert!(resources.get("NON_EXISTENT").is_none());
	assert!(resources.get("ERRORS").is_some());

	match resources.remove("ERRORS").unwrap() {
		Ok(_) => return Err(InternalError::OtherError("This should be an error".into())),
		Err(err) => match err {
			InternalError::MissingResourceError(_) => {
				resources.remove("ERRORS");
			},

			specific => return Err(specific),
		},
	};

	for (_, res) in resources {
		assert_eq!(res?.data.as_ref(), &INPUT[..]);
	}

	Ok(())
}
