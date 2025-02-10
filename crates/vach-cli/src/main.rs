// Fundamental modules
mod app;
mod commands;
mod keys;
mod utils;

// NOTE: Unwrapping in a CLI is a no-no. Since throwing Rust developer errors at average users is mental overload
fn main() {
	// Build CLI
	let keys = keys::build_keys();
	let app = app::build_app(keys);
	let commands = commands::build_commands();

	// Start CLI
	let matches = app.get_matches();

	match matches.subcommand() {
		Some((key, mtx)) => commands.get(key).unwrap().evaluate(mtx),
		None => {
			println!("vach-cli: Run `vach --help` and refer to crates.io/vach-cli for the manual");
			Ok(())
		},
	}
	.unwrap();
}
