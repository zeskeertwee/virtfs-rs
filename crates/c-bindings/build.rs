extern crate cbindgen;

use std::env;
use cbindgen::*;

fn main() {
	let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

	let mut bindings = Builder::new()
		.with_crate(crate_dir)
		.with_no_includes()
		.with_sys_include("stdbool.h")
		.with_sys_include("stdint.h")
		.with_language(Language::C)
		.with_line_length(300)
		.with_pragma_once(true)
		.generate()
		.expect("Unable to generate bindings");

	bindings.config.documentation_style = DocumentationStyle::C99;
	bindings.config.documentation_length = DocumentationLength::Full;

	bindings.write_to_file("bindings.h");
}
