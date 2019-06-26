use std::env;

fn main() {
    let crate_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(cbindgen::Language::C)
        .with_include_guard("_naru__bindings_h_")
        .with_namespace("naru")
        .generate()
        .map(|b| b.write_to_file("naru.h"))
        .expect("Unable to generate bindings");
}
