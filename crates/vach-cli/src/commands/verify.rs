use std::fs::File;

use vach::archive::{Archive, ArchiveConfig};

use super::CommandTrait;
use crate::keys::key_names;

pub const VERSION: &str = "0.0.1";

/// This command verifies the validity and integrity of an archive
pub struct Evaluator;

impl CommandTrait for Evaluator {
	fn evaluate(&self, args: &clap::ArgMatches) -> anyhow::Result<()> {
		let input_path = match args.value_of(key_names::INPUT) {
			Some(path) => path,
			None => anyhow::bail!("Please provide an input path using the -i or --input key"),
		};

		let magic: [u8; vach::MAGIC_LENGTH] = match args.value_of(key_names::MAGIC) {
			Some(magic) => magic.as_bytes().try_into()?,
			None => *vach::DEFAULT_MAGIC,
		};

		let input_file = File::open(input_path)?;

		if let Err(err) = Archive::with_config(input_file, &ArchiveConfig::new(magic, None)) {
			anyhow::bail!("Unable to verify the archive source, error: {}", err.to_string())
		};

		Ok(())
	}
}
