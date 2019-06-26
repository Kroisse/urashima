use std::env;
use std::path::Path;

const ATOMS: &[&str] = &[
    // labels
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15",
    // keywords
    "assert", "break", "continue", "else", "false", "fn", "for", "if", "new", "package", "pub",
    "return", "static", "switch", "syntax", "test", "true", "type", "use", "var", "yield",
    // stdlib
    "naru", "core", "bool", "int", "nat", "rat", "str", "ref", "result", "println",
];

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    string_cache_codegen::AtomType::new("Symbol", "symbol!")
        .atoms(ATOMS)
        .write_to_file(&Path::new(&out_dir).join("symbol.rs"))
        .unwrap();
}
