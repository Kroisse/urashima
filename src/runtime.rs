use std::path::{Path, PathBuf};
use std::sync::{Arc, Weak};

use chashmap::CHashMap;

use crate::{
    capsule::Capsule,
    environment::Package,
    error::{ErrorKind, Fallible},
    eval::Evaluate,
    program::{PackagePath, ScriptProgram},
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

    pub fn root_capsule(&self) -> Capsule {
        Capsule::root(Arc::clone(&self.inner))
    }

    pub fn execute(&self, path: impl AsRef<Path>) -> Fallible<()> {
        let path = path.as_ref();
        let f = std::fs::File::open(path).map_err(|_| ErrorKind::Load(path.to_owned()))?;
        let prog: serde_yaml::Value =
            serde_yaml::from_reader(f).map_err(|_| ErrorKind::Load(path.to_owned()))?;
        let mut capsule = self.root_capsule();
        let prog: ScriptProgram = capsule
            .parse(prog)
            .map_err(|_| ErrorKind::Load(path.to_owned()))?;
        prog.eval(&mut capsule)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use failure::Fallible;

    use super::*;

    #[test]
    fn helloworld() -> Fallible<()> {
        let s = include_bytes!("../tests/helloworld.yaml");
        let prog: serde_yaml::Value = serde_yaml::from_slice(&*s)?;
        let rt = Runtime::new();
        let mut capsule = rt.root_capsule();
        let prog: ScriptProgram = capsule.parse(prog)?;
        capsule.eval(&prog)?;
        Ok(())
    }
}
