use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let crate_version = env::var("CARGO_PKG_VERSION").unwrap();
    let version_string = format!("\n\n#define VERSION \"{}\"", crate_version);

    let mut bindings =
        cbindgen::generate(crate_dir).expect("cbindgen failed to generate C bindings");
    let after_includes = bindings
        .config
        .after_includes
        .get_or_insert_with(String::new);
    after_includes.push_str(&version_string);

    bindings.write_to_file("bindings.h");
}
