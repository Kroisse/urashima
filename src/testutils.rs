#[macro_export]
macro_rules! assert_pat {
    ($value:expr, $($expected:pat => $extra:block)*) => {
        match $value {
            $( $expected => $extra )*
            otherwise => panic!("Unexpected: {:?}", otherwise),
        }
    }
}
