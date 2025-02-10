use std::collections::HashMap;
use clap::{Command, Arg};

use crate::keys::key_names;
use crate::commands;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

pub fn build_app<'a>(key_map: HashMap<&'static str, Arg<'a>>) -> Command<'a> {
	Command::new("vach-cli")
		.author(self::AUTHORS)
		.about("A command-line interface for unpacking and packing files")
		.version(self::VERSION)
		.subcommand(
			Command::new("keypair")
				.author(AUTHORS)
				.version(commands::keypair::VERSION)
				.about("Generate a keypair (public & secret key)")
				.arg(key_map.get(key_names::OUTPUT).unwrap())
				.arg(key_map.get(key_names::SPLIT_KEY).unwrap()),
		)
		.subcommand(
			Command::new("split")
				.author(AUTHORS)
				.version(commands::split::VERSION)
				.about("Splits a keypair into it's respective secret and public keys")
				.arg(key_map.get(key_names::INPUT).unwrap()),
		)
		.subcommand(
			Command::new("verify")
				.author(AUTHORS)
				.version(commands::verify::VERSION)
				.about("Verifies the validity of an archive")
				.arg(key_map.get(key_names::MAGIC).unwrap())
				.arg(key_map.get(key_names::INPUT).unwrap()),
		)
		.subcommand(
			Command::new("list")
				.author(AUTHORS)
				.version(commands::list::VERSION)
				.about("Lists all the entries in a archive and their metadata")
				.arg(key_map.get(key_names::INPUT).unwrap())
				.arg(key_map.get(key_names::MAGIC).unwrap())
				.arg(key_map.get(key_names::SORT).unwrap()),
		)
		.subcommand(
			Command::new("unpack")
				.author(AUTHORS)
				.version(commands::unpack::VERSION)
				.about("Unpacks a archive")
				// Files
				.arg(key_map.get(key_names::OUTPUT).unwrap())
				.arg(key_map.get(key_names::INPUT).unwrap())
				// encryption
				.arg(key_map.get(key_names::KEYPAIR).unwrap())
				.arg(key_map.get(key_names::MAGIC).unwrap())
				.arg(key_map.get(key_names::PUBLIC_KEY).unwrap())
				// modifiers
				.arg(key_map.get(key_names::JOBS).unwrap())
				.arg(key_map.get(key_names::TRUNCATE).unwrap()),
		)
		.subcommand(
			Command::new("pipe")
				.author(AUTHORS)
				.version(commands::pipe::VERSION)
				.about("Pipes a Resource from an archive to stdout")
				.arg(key_map.get(key_names::INPUT).unwrap())
				.arg(key_map.get(key_names::MAGIC).unwrap())
				.arg(key_map.get(key_names::PUBLIC_KEY).unwrap())
				.arg(key_map.get(key_names::RESOURCE).unwrap())
				.arg(key_map.get(key_names::KEYPAIR).unwrap()),
		)
		.subcommand(
			Command::new("pack")
				.author(AUTHORS)
				.version(commands::pack::VERSION)
				.about("Packages all input files into a archive")
				// Output file
				.arg(key_map.get(key_names::OUTPUT).unwrap())
				// Data sources
				.arg(key_map.get(key_names::INPUT).unwrap())
				.arg(key_map.get(key_names::DIR_INPUT).unwrap())
				.arg(key_map.get(key_names::DIR_INPUT_REC).unwrap())
				.arg(key_map.get(key_names::EXCLUDE).unwrap())
				// Crypto shit
				.arg(key_map.get(key_names::KEYPAIR).unwrap())
				.arg(key_map.get(key_names::SECRET_KEY).unwrap())
				// Modifiers
				.arg(key_map.get(key_names::JOBS).unwrap())
				.arg(key_map.get(key_names::FLAGS).unwrap())
				.arg(key_map.get(key_names::COMPRESS_MODE).unwrap())
				.arg(key_map.get(key_names::COMPRESS_ALGO).unwrap())
				.arg(key_map.get(key_names::MAGIC).unwrap())
				.arg(key_map.get(key_names::ENCRYPT).unwrap())
				.arg(key_map.get(key_names::HASH).unwrap())
				.arg(key_map.get(key_names::VERSION).unwrap())
				.arg(key_map.get(key_names::TRUNCATE).unwrap()),
		)
}
