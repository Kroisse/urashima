use std::sync::Arc;

use crate::{
    data::{Symbol, Variant},
    error::{Error, Fallible},
};

/// Execution context
#[derive(Clone, Default)]
#[cfg_attr(test, derive(Debug))]
pub struct Environment {
    pub(crate) values: Vec<Variant>,
    pub(crate) names: Vec<Symbol>,
    pub(crate) packages: Vec<Arc<Package>>,
}

impl Environment {
    pub(crate) fn bind(&mut self, name: &str, value: Variant) {
        self.names.push(name.into());
        self.values.push(value);
    }

    pub(crate) fn lookup_name(&self, name: &str) -> Fallible<&Variant> {
        let i = self
            .names
            .iter()
            .position(|n| n == name)
            .ok_or_else(Error::name)?;
        Ok(&self.values[i])
    }
}

#[cfg(not(test))]
impl ::std::fmt::Debug for Environment {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.write_str("Environment")
    }
}

#[cfg_attr(test, derive(Debug))]
pub struct Package {
    pub(crate) environment: Environment,
}
