use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use std::str::FromStr;
use anyhow::{Result, bail};

pub fn create_and_write_to_file(path: &str, data: &[u8]) -> Result<()> {
	let path = PathBuf::from_str(path)?;

	// Check if the file exists
	if path.exists() {
		bail!("The file {} already exists!", path.to_string_lossy());
	}

	let mut file = File::create(path)?;
	file.write_all(data)?;

	Ok(())
}
