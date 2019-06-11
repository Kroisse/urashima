use std::env;
use std::path::Path;

const ATOMS: &[&str] = &[
    // keywords
    "assert", "break", "continue", "else", "false", "fn", "for", "if", "new", "package", "pub",
    "return", "static", "switch", "syntax", "test", "true", "type", "use", "var", "yield",
    // stdlib
    "naru", "core", "bool", "int", "nat", "rat", "ref", "result", "println",
];

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    string_cache_codegen::AtomType::new("data::Symbol", "symbol!")
        .atoms(ATOMS)
        .write_to_file(&Path::new(&out_dir).join("symbol.rs"))
        .unwrap();
}
