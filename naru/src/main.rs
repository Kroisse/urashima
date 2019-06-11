use std::env;

use urashima::{Fallible, Runtime};

fn main() -> Fallible<()> {
    env_logger::init();
    let path = env::args().nth(1).unwrap();
    let rt = Runtime::new();
    rt.execute(path)?;
    Ok(())
}
