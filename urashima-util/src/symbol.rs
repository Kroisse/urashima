include!(concat!(env!("OUT_DIR"), "/symbol.rs"));

#[cfg(test)]
mod test {
    use std::mem;

    use super::Symbol;

    #[test]
    fn symbol_size() {
        assert!(mem::size_of::<Symbol>() <= 8);
    }
}
