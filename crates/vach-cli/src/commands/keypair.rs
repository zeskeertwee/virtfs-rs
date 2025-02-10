use vach::crypto_utils::gen_keypair;

use crate::utils;
use crate::keys::key_names;

use super::CommandTrait;

// Default keypair write destination
const DEFAULT_KEYPAIR_FILE_NAME: &str = "keypair.kp";
pub const VERSION: &str = "0.0.1";

/// This command is used to generate keypair
pub struct Evaluator;

impl CommandTrait for Evaluator {
	fn evaluate(&self, args: &clap::ArgMatches) -> anyhow::Result<()> {
		let mut output_path = match args.value_of(key_names::OUTPUT) {
			Some(path) => path.to_string(),
			None => DEFAULT_KEYPAIR_FILE_NAME.to_string(),
		};

		let kp = gen_keypair();
		if args.is_present(key_names::SPLIT_KEY) {
			output_path = output_path.trim_end_matches(".kp").to_string();

			let mut sk_path = output_path.clone();
			sk_path.push_str(".sk");

			let mut pk_path = output_path;
			pk_path.push_str(".pk");

			utils::create_and_write_to_file(&sk_path, &kp.to_bytes())?;
			println!("Secret Key successfully generated and saved in: {}", sk_path);

			utils::create_and_write_to_file(&pk_path, &kp.verifying_key().to_bytes())?;
			println!("Public Key successfully generated and saved in: {}", pk_path);
		} else {
			utils::create_and_write_to_file(&output_path, &kp.to_keypair_bytes())?;
			println!("KeyPair successfully generated and saved in: {}", output_path);
		}

		Ok(())
	}
}
