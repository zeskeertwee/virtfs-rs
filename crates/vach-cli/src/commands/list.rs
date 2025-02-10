use std::fs::File;

use tabled::{
	Table, Tabled,
	settings::{*, object::Columns},
};
use vach::prelude::{ArchiveConfig, Archive, Flags};
use indicatif::HumanBytes;

use super::CommandTrait;
use crate::keys::key_names;

pub const VERSION: &str = "0.2";

/// This command lists the entries in an archive in tabulated form
pub struct Evaluator;

impl CommandTrait for Evaluator {
	fn evaluate(&self, args: &clap::ArgMatches) -> anyhow::Result<()> {
		let archive_path = match args.value_of(key_names::INPUT) {
			Some(path) => path,
			None => {
				anyhow::bail!("Please provide an input archive file using the -i or --input keys!")
			},
		};

		let magic: [u8; vach::MAGIC_LENGTH] = match args.value_of(key_names::MAGIC) {
			Some(magic) => magic.as_bytes().try_into()?,
			None => *vach::DEFAULT_MAGIC,
		};

		let file = File::open(archive_path)?;
		let archive = Archive::with_config(file, &ArchiveConfig::new(magic, None))?;

		// log basic metadata
		println!("{}", archive);

		let mut entries: Vec<_> = archive.entries().iter().map(|(_, entry)| entry).collect();

		// Sort the entries accordingly
		match args.value_of(key_names::SORT) {
			Some("alphabetical") => entries.sort_by(|a, b| a.id.cmp(&b.id)),
			Some("alphabetical-reversed") => entries.sort_by(|a, b| b.id.cmp(&a.id)),
			Some("size-ascending") => entries.sort_by(|a, b| a.offset.cmp(&b.offset)),
			Some("size-descending") => entries.sort_by(|a, b| b.offset.cmp(&a.offset)),
			Some(sort) => anyhow::bail!("Unknown sort option provided: {}. Valid sort types are: 'alphabetical' 'alphabetical-descending' 'size-ascending' 'size-descending'", sort),
			_ => (),
		};

		let table_entries: Vec<FileTableEntry> = entries
			.into_iter()
			.map(|entry| {
				let c_algo = if entry.flags.contains(Flags::LZ4_COMPRESSED) {
					"LZ4"
				} else if entry.flags.contains(Flags::BROTLI_COMPRESSED) {
					"Brotli"
				} else if entry.flags.contains(Flags::SNAPPY_COMPRESSED) {
					"Snappy"
				} else {
					"None"
				};

				FileTableEntry {
					id: &entry.id,
					size: HumanBytes(entry.offset).to_string(),
					flags: entry.flags,
					compression: c_algo,
				}
			})
			.collect();

		let mut table = Table::new(table_entries);
		table
			.with(Style::rounded())
			.with(Modify::list(Columns::new(..1), Alignment::left()));

		println!("{}", table);

		Ok(())
	}
}

#[derive(Tabled)]
struct FileTableEntry<'a> {
	id: &'a str,
	size: String,
	flags: Flags,
	compression: &'static str,
}
