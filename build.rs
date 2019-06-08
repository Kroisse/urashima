use std::env;
use std::path::Path;

const ATOMS: &[&str] = &[
    // keywords
    "assert", "break", "continue", "else", "false", "fn", "for", "if", "new", "package", "pub",
    "return", "static", "switch", "syntax", "test", "true", "type", "use", "var", "yield",
    // stdlib
    "naru", "core", "println",
];

fn main() {
    string_cache_codegen::AtomType::new("data::Symbol", "symbol!")
        .atoms(ATOMS)
        .write_to_file(&Path::new(&env::var("OUT_DIR").unwrap()).join("symbol.rs"))
        .unwrap()
}
