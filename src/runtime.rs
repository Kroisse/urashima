use std::path::{Path, PathBuf};
use std::sync::{Arc, Weak};

use chashmap::CHashMap;
use urashima_ast::{parse, program::ScriptProgram};
use urashima_util::PackagePath;

use crate::{
    capsule::Capsule,
    environment::Package,
    error::{Error, Fallible},
    eval::Evaluate,
};

pub struct Runtime {
    inner: RuntimeContextRef,
}

pub(crate) struct RuntimeContext {
    pub(crate) packages: CHashMap<PackagePath, Weak<Package>>,
    pub(crate) paths: Vec<PathBuf>,
}

pub(crate) type RuntimeContextRef = Arc<RuntimeContext>;

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Self {
        let cur_dir = std::env::current_dir().unwrap();
        let ctx = RuntimeContext {
            packages: CHashMap::new(),
            paths: vec![cur_dir],
        };
        Runtime {
            inner: Arc::new(ctx),
        }
    }

    pub(crate) fn context(&self) -> RuntimeContextRef {
        Arc::clone(&self.inner)
    }

    pub fn root_capsule(&self) -> Capsule<'static> {
        Capsule::root(self.context())
    }

    pub fn execute(&self, path: impl AsRef<Path>) -> Fallible<()> {
        let path = path.as_ref();
        let input = std::fs::read_to_string(path).map_err(|_| Error::load(path))?;
        let mut capsule = self.root_capsule();
        let prog: ScriptProgram = capsule.parse_sourcecode(&input)?;
        prog.eval(&mut capsule)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::io;

    use super::*;

    #[test]
    fn helloworld() {
        let s = include_str!("../tests/helloworld.n");
        let rt = Runtime::new();
        let mut out = Vec::new();
        {
            let w = Box::new(io::Cursor::new(&mut out));
            let mut capsule = Capsule::new(rt.context(), w);
            let prog: ScriptProgram = capsule.parse_sourcecode(&s).unwrap();
            capsule.eval(&prog).unwrap();
        }
        assert_eq!(std::str::from_utf8(&out).unwrap(), "Hello, world!\n");
    }

    #[cfg(feature = "deserialize")]
    #[test]
    fn helloworld_yaml() {
        let s = include_bytes!("../tests/helloworld.yaml");
        let prog: serde_yaml::Value = serde_yaml::from_slice(&*s).unwrap();
        let rt = Runtime::new();
        let mut out = Vec::new();
        {
            let w = Box::new(io::Cursor::new(&mut out));
            let mut capsule = Capsule::new(rt.context(), w);
            let prog: ScriptProgram = capsule.parse(prog).unwrap();
            capsule.eval(&prog).unwrap();
        }
        assert_eq!(std::str::from_utf8(&out).unwrap(), "Hello, world!\n");
    }
}
