use std::io::prelude::*;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Weak};

use crate::data::{Symbol, Variant};
use crate::environment::{Environment, Package};
use crate::error::{Error, ErrorKind, Fallible};
use crate::eval::Evaluate;
use crate::program::PackagePath;
use crate::runtime::RuntimeContextRef;

pub struct Capsule {
    pub(crate) ctx: RuntimeContextRef,
    pub(crate) environments: Vec<Environment>,
}

impl Capsule {
    pub(crate) fn root(ctx: RuntimeContextRef) -> Capsule {
        Capsule {
            ctx,
            environments: vec![Default::default()],
        }
    }

    pub fn eval<T>(&mut self, code: &T) -> Fallible<T::Value>
    where
        T: Evaluate,
    {
        code.eval(self)
    }

    pub(crate) fn load(&mut self, path: PackagePath) -> Fallible<Arc<Package>> {
        let mut res = Err(ErrorKind::Import(path.clone()).into());
        let ctx = Arc::clone(&self.ctx);
        ctx.packages.alter(path.clone(), |mut entry| {
            if let Some(pkg) = entry.as_ref().and_then(Weak::upgrade) {
                res = Ok(pkg);
                return entry;
            }
            res = (|| {
                let prog = self::internal::load(&self.ctx.paths, &path)?;
                let mut pkg_capsule = Capsule::root(self.ctx.clone());
                prog.eval(&mut pkg_capsule)?;
                let pkg = Package {
                    environment: pkg_capsule
                        .environments
                        .into_iter()
                        .next()
                        .expect("no environment"),
                };
                let pkg = Arc::new(pkg);
                self.environment_mut().packages.push(Arc::clone(&pkg));
                entry = Some(Arc::downgrade(&pkg));
                Ok(pkg)
            })();
            entry
        });
        res
    }

    pub(crate) fn push(&mut self) -> ContextGuard<'_> {
        ContextGuard::new(self)
    }

    fn environment_mut(&mut self) -> &mut Environment {
        self.environments
            .last_mut()
            .expect("no environment")
    }

    pub(crate) fn bind(&mut self, name: Symbol, value: Variant) {
        self.environment_mut().bind(name, value);
    }

    pub(crate) fn lookup(&self, depth: usize, index: usize) -> Fallible<&Variant> {
        self._lookup(depth, index).ok_or_else(Error::name)
    }

    fn _lookup(&self, depth: usize, index: usize) -> Option<&Variant> {
        let i = self.environments.len().checked_sub(depth + 1)?;
        self.environments.get(i)?.values.get(index)
    }

    pub(crate) fn write(&mut self, bytes: &[u8]) -> Fallible<()> {
        std::io::stdout().write_all(bytes).expect("write error");
        Ok(())
    }
}

pub(crate) struct ContextGuard<'a>(&'a mut Capsule);

impl<'a> ContextGuard<'a> {
    fn new(ctx: &'a mut Capsule) -> Self {
        ctx.environments.push(Environment::default());
        ContextGuard(ctx)
    }

    pub(crate) fn load(&mut self, env: &Environment) {
        self.0
            .environments
            .last_mut()
            .expect("no environment")
            .clone_from(env);
    }
}

impl Drop for ContextGuard<'_> {
    fn drop(&mut self) {
        self.0.environments.pop();
    }
}

impl<'a> Deref for ContextGuard<'a> {
    type Target = Capsule;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<'a> DerefMut for ContextGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

mod internal {
    use std::fs::File;
    use std::path::{Path, PathBuf};

    use crate::error::{ErrorKind, Fallible};
    use crate::program::{PackagePath, PackageProgram};

    pub(super) fn load(paths: &[PathBuf], pkg_path: &PackagePath) -> Fallible<PackageProgram> {
        for base_path in paths {
            let mut path = base_path.clone();
            path.extend(pkg_path.into_iter().map(|i| i.as_ref()));
            path.set_extension("yaml");
            log::info!("{}", path.display());
            if path.is_file() {
                return Ok(from_path(&path).unwrap());
            }
        }
        Err(ErrorKind::Import(pkg_path.clone()).into())
    }

    fn from_path(path: impl AsRef<Path>) -> failure::Fallible<PackageProgram> {
        let f = File::open(path.as_ref())?;
        let pkg = serde_yaml::from_reader(f)?;
        Ok(pkg)
    }
}
