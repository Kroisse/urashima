use std::fmt;
use std::io::prelude::*;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Weak};

use urashima_ast::{
    expr::{Alloc, ExprArena},
    program::{PackagePath, PackageProgram},
    Parse,
};

use crate::{
    data::Variant,
    environment::{Environment, Package},
    error::{Error, Fallible},
    eval::Evaluate,
    runtime::RuntimeContextRef,
};

pub struct Capsule {
    pub(crate) ctx: RuntimeContextRef,
    pub(crate) environment: Environment,
    pub(crate) expr_arena: ExprArena,
}

impl Capsule {
    pub(crate) fn root(ctx: RuntimeContextRef) -> Capsule {
        Capsule {
            ctx,
            environment: Default::default(),
            expr_arena: ExprArena::new(),
        }
    }

    pub(crate) fn parse_sourcecode<'a, T>(&'a mut self, input: &str) -> Fallible<T>
    where
        T: Parse,
    {
        Ok(urashima_ast::parse(&mut self.expr_arena, input)?)
    }

    pub fn eval<T>(&mut self, code: &T) -> Fallible<T::Value>
    where
        T: Evaluate,
    {
        code.eval(self)
    }

    pub(crate) fn load(&mut self, path: PackagePath) -> Fallible<Arc<Package>> {
        let mut res = Err(Error::import(&path));
        let ctx = Arc::clone(&self.ctx);
        ctx.packages.alter(path.clone(), |mut entry| {
            if let Some(pkg) = entry.as_ref().and_then(Weak::upgrade) {
                res = Ok(pkg);
                return entry;
            }
            res = (|| {
                let input = self::internal::load(&self.ctx.paths, &path)?;
                let mut pkg_capsule = Capsule::root(self.ctx.clone());
                let prog: PackageProgram = pkg_capsule.parse_sourcecode(&input)?;
                prog.eval(&mut pkg_capsule)?;
                let pkg = Package {
                    environment: pkg_capsule.environment,
                };
                let pkg = Arc::new(pkg);
                self.environment.add_package(Arc::clone(&pkg));
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

    pub(crate) fn bind(&mut self, name: &str, value: Variant) {
        self.environment.bind(name, value);
    }

    pub(crate) fn print(&mut self, args: fmt::Arguments<'_>) -> Fallible<()> {
        write!(std::io::stdout(), "{}", args).expect("write error");
        Ok(())
    }
}

impl Capsule {
    pub(crate) fn alloc<T>(&mut self) -> Alloc<'_, T> {
        Alloc::from(&mut self.expr_arena)
    }
}

pub(crate) struct ContextGuard<'a>(&'a mut Capsule);

impl<'a> ContextGuard<'a> {
    fn new(ctx: &'a mut Capsule) -> Self {
        ctx.environment.push();
        ContextGuard(ctx)
    }
}

impl Drop for ContextGuard<'_> {
    fn drop(&mut self) {
        self.0.environment.pop();
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

#[cfg(deserialize)]
mod de {
    use serde::de::{DeserializeSeed, Deserializer};

    pub(crate) trait Parse<'a>: Sized {
        fn parse<D>(capsule: &'a mut Capsule, deserializer: D) -> Fallible<Self>
        where
            D: for<'de> Deserializer<'de>;
    }

    impl<'a, T: 'static> Parse<'a> for T
    where
        T: Sized,
        Alloc<'a, T>: for<'de> DeserializeSeed<'de, Value = T>,
    {
        fn parse<D>(capsule: &'a mut Capsule, deserializer: D) -> Fallible<Self>
        where
            D: for<'de> Deserializer<'de>,
        {
            DeserializeSeed::deserialize(capsule.alloc::<T>(), deserializer).map_err(Error::from_de)
        }
    }

    impl Capsule {
        pub(crate) fn parse<'a, T, D>(&'a mut self, deserializer: D) -> Fallible<T>
        where
            T: Parse<'a>,
            D: for<'de> Deserializer<'de>,
        {
            Parse::parse(self, deserializer)
        }
    }
}

mod internal {
    use std::path::{Path, PathBuf};

    use urashima_ast::program::PackagePath;

    use crate::error::{Error, Fallible};

    pub(super) fn load(paths: &[PathBuf], pkg_path: &PackagePath) -> Fallible<String> {
        for base_path in paths {
            let mut path = base_path.clone();
            path.extend(pkg_path.into_iter().map(|i| i.as_ref()));
            path.set_extension("n");
            log::info!("{}", path.display());
            if path.is_file() {
                return Ok(from_path(&path)?);
            }
        }
        Err(Error::import(pkg_path))
    }

    fn from_path(path: impl AsRef<Path>) -> Fallible<String> {
        let path = path.as_ref();
        let input = std::fs::read_to_string(path).map_err(|_| Error::load(path))?;
        Ok(input)
    }
}
