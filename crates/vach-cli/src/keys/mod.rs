use clap::Arg;
use std::collections::HashMap;

pub mod key_names {
	pub(crate) const JOBS: &str = "JOBS";

	pub(crate) const OUTPUT: &str = "OUTPUT";
	pub(crate) const INPUT: &str = "INPUT";
	pub(crate) const RESOURCE: &str = "RESOURCE";

	pub(crate) const DIR_INPUT: &str = "DIR_INPUT";
	pub(crate) const DIR_INPUT_REC: &str = "DIR_INPUT_REC";

	pub(crate) const EXCLUDE: &str = "EXCLUDE";
	pub(crate) const TRUNCATE: &str = "TRUNCATE";

	pub(crate) const FLAGS: &str = "FLAGS";
	pub(crate) const VERSION: &str = "VERSION";
	pub(crate) const MAGIC: &str = "MAGIC";
	pub(crate) const COMPRESS_MODE: &str = "COMPRESS_MODE";
	pub(crate) const COMPRESS_ALGO: &str = "COMPRESS_ALGO";
	pub(crate) const HASH: &str = "HASH";
	pub(crate) const ENCRYPT: &str = "ENCRYPT";
	pub(crate) const SPLIT_KEY: &str = "SPLIT_KEY";

	pub(crate) const SECRET_KEY: &str = "SECRET_KEY";
	pub(crate) const PUBLIC_KEY: &str = "PUBLIC_KEY";
	pub(crate) const KEYPAIR: &str = "KEYPAIR";

	pub(crate) const SORT: &str = "SORT";
}

pub fn build_keys<'a>() -> HashMap<&'static str, Arg<'a>> {
	/* please only use this function once during the lifecycle of the program */
	let mut map = HashMap::with_capacity(20);

	/* The various keys usable in the CLI */
	// Number of threads to spawn during processing
	map.insert(
		key_names::JOBS,
		Arg::new(key_names::JOBS)
			.short('j')
			.long("jobs")
			.value_name(key_names::JOBS)
			.help("How many threads to spawn during archive processing, defaults to number of threads on system")
			.required(false)
			.takes_value(true)
			.number_of_values(1),
	);

	// A general output target
	map.insert(
		key_names::OUTPUT,
		Arg::new(key_names::OUTPUT)
			.short('o')
			.long("output")
			.value_name(key_names::OUTPUT)
			.help("A general output target, for example a file to write to")
			.required(false)
			.takes_value(true)
			.number_of_values(1),
	);

	// A resource to focus on and extract
	map.insert(
		key_names::RESOURCE,
		Arg::new(key_names::RESOURCE)
			.short('r')
			.long("resource")
			.value_name(key_names::RESOURCE)
			.help("An exact resource to extract from the archive")
			.required(false)
			.takes_value(true)
			.number_of_values(1),
	);

	// A general input source
	map.insert(
		key_names::INPUT,
		Arg::new(key_names::INPUT)
			.long("input")
			.short('i')
			.value_name(key_names::INPUT)
			.help("A general list of input sources, like paths to files")
			.required(false)
			.takes_value(true)
			.multiple_values(true),
	);

	// add all files in a directory into the input queue
	map.insert(
		key_names::DIR_INPUT,
		Arg::new(key_names::DIR_INPUT)
			.long("directory")
			.short('d')
			.value_name(key_names::DIR_INPUT)
			.help("Add all files in a directory into the input queue")
			.required(false)
			.takes_value(true)
			.multiple_values(true),
	);

	// same as above, only that it adds files from the directory recursively
	map.insert(
		key_names::DIR_INPUT_REC,
		Arg::new(key_names::DIR_INPUT_REC)
			.long("directory-r")
			.short('r')
			.value_name(key_names::DIR_INPUT_REC)
			.help("Recursively add all files in a directory into the input queue")
			.required(false)
			.takes_value(true)
			.multiple_values(true),
	);

	// exclude the given files from the write queue
	map.insert(
		key_names::EXCLUDE,
		Arg::new(key_names::EXCLUDE)
			.long("exclude")
			.short('x')
			.value_name(key_names::EXCLUDE)
			.help("Exclude the given paths from the input queue")
			.required(false)
			.takes_value(true)
			.multiple_values(true),
	);

	// Deletes the original files after they have been processed successfully
	map.insert(
		key_names::TRUNCATE,
		Arg::new(key_names::TRUNCATE)
			.short('t')
			.long("truncate")
			.value_name(key_names::TRUNCATE)
			.help("Exclude the given paths from the input queue")
			.required(false)
			.takes_value(false),
	);

	// treats the entries in a .vach file like regular files, but with metadata from the archive
	map.insert(
		key_names::MAGIC,
		Arg::new(key_names::MAGIC)
			.long("magic")
			.short('m')
			.value_name(key_names::MAGIC)
			.help("The magic used to generate the archive")
			.required(false)
			.takes_value(true)
			.number_of_values(1)
			.validator(|magic| {
				if magic.len() != vach::MAGIC_LENGTH {
					return Err(format!(
						"Please provide a magic of the right length: {}. Magic: {} has length: {}",
						vach::MAGIC_LENGTH,
						&magic,
						magic.len()
					));
				};

				Ok(())
			}),
	);

	// The compress mode of the adjacent leafs
	map.insert(
		key_names::COMPRESS_MODE,
		Arg::new(key_names::COMPRESS_MODE)
			.long("compress-mode")
			.short('c')
			.value_name(key_names::COMPRESS_MODE)
			.help("The compress mode of the adjacent leafs, Can be 'Always', 'Detect' or 'Never' (case insensitive). Defaults to 'Detect'")
			.required(false)
			.takes_value(true)
			.number_of_values(1)
			.validator(|c_mode| {
				let c_mode = c_mode.to_ascii_lowercase();
				if c_mode != "always" && c_mode != "never" && c_mode != "detect" {
					return Err(format!("Please provide a valid Compress Mode, either 'Always', 'Detect' or 'Never' (case insensitive). Not: {}", c_mode));
				};

				Ok(())
			}),
	);

	// The compression algorithm to use for the adjacent leafs
	map.insert(
		key_names::COMPRESS_ALGO,
		Arg::new(key_names::COMPRESS_ALGO)
			.long("compress-algo")
			.short('g')
			.value_name(key_names::COMPRESS_ALGO)
			.help("The compression algorithm to use in compression, can be; 'lz4', 'brotli' or 'snappy'")
			.required(false)
			.takes_value(true)
			.number_of_values(1)
			.validator(|c_mode| {
				let c_mode = c_mode.to_ascii_lowercase();
				if c_mode != "lz4" && c_mode != "brotli" && c_mode != "snappy" {
					return Err(format!("Please provide a valid Compression Algorithm to use, either 'lz4', 'brotli' or 'snappy' (case insensitive). Not: {}", c_mode));
				};

				Ok(())
			}),
	);

	// To sign the entries and include the signatures in the target, an sk or kp must be provided
	map.insert(
		key_names::HASH,
		Arg::new(key_names::HASH)
			.short('a')
			.long("hash")
			.value_name(key_names::HASH)
			.help("To sign the entries and include the signatures in the target, an sk or kp must be provided")
			.required(false)
			.takes_value(false),
	);

	// Encrypt the data, an sk or kp must be provided
	map.insert(
		key_names::ENCRYPT,
		Arg::new(key_names::ENCRYPT)
			.short('e')
			.long("encrypt")
			.value_name(key_names::ENCRYPT)
			.help("Encrypt the data, a secret key or keypair must be provided with either -s or -k")
			.required(false)
			.takes_value(false),
	);

	// Used in conjunction with the keypair subcommand to split the keypair upon generation into it's two parts
	map.insert(
		key_names::SPLIT_KEY,
		Arg::new(key_names::SPLIT_KEY)
			.long("split-key")
			.value_name(key_names::SPLIT_KEY)
			.help("Used in conjunction with the keypair subcommand to split the keypair upon generation into it's two parts")
			.required(false)
			.takes_value(false),
	);

	// The secret key to be used in signing of signatures
	map.insert(
		key_names::SECRET_KEY,
		Arg::new(key_names::SECRET_KEY)
			.short('s')
			.long("secret-key")
			.value_name(key_names::SECRET_KEY)
			.help("The secret key used to signing data, do not distribute your secret key")
			.required(false)
			.takes_value(true)
			.number_of_values(1),
	);

	// The public key to be used in decryption and validation of signatures
	map.insert(
		key_names::PUBLIC_KEY,
		Arg::new(key_names::PUBLIC_KEY)
			.short('p')
			.long("public-key")
			.value_name(key_names::PUBLIC_KEY)
			.help("The public key used in decryption and authentication of signatures")
			.required(false)
			.takes_value(true)
			.number_of_values(1),
	);

	// A keypair is just a pair of the above two, but when set will be chosen over the above
	map.insert(
		key_names::KEYPAIR,
		Arg::new(key_names::KEYPAIR)
			.long("keypair")
			.short('k')
			.value_name(key_names::KEYPAIR)
			.help("A pair of cryptographic keys, note only ever distribute the public key. Use the keypair during development")
			.required(false)
			.takes_value(true)
			.number_of_values(1),
	);

	// The flags that go into the .vach file header section
	map.insert(
		key_names::FLAGS,
		Arg::new(key_names::FLAGS)
			.long("flags")
			.short('f')
			.value_name(key_names::FLAGS)
			.help("The flags that go into the .vach file header section")
			.required(false)
			.takes_value(true)
			.number_of_values(1),
	);

	// the version of the leafs being read or to be written
	map.insert(
		key_names::VERSION,
		Arg::new(key_names::VERSION)
			.long("version")
			.short('v')
			.value_name(key_names::VERSION)
			.help("the version of the leafs being read or to be written")
			.required(false)
			.takes_value(true)
			.number_of_values(1),
	);

	// the version of the leafs being read or to be written
	map.insert(
		key_names::SORT,
		Arg::new(key_names::SORT)
			.long("sort")
			.value_name(key_names::SORT)
			.help("How to sort entries within the table, either based on size or alphabetically")
			.required(false)
			.takes_value(true)
			.number_of_values(1),
	);

	map
}
