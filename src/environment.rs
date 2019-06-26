use std::sync::Arc;

use urashima_util::arena::{Arena, Index};

use crate::{
    data::{Function, Symbol, Variant},
    error::{Error, Fallible},
};

/// Execution context
#[derive(Clone, Default)]
pub struct Environment {
    pub(crate) values: Vec<Variant>,
    pub(crate) names: Vec<Symbol>,
    heads: Vec<usize>, // TODO: call stack metadata
    packages: Vec<Arc<Package>>,
    fn_arena: Arena<Function>,
    arena: Arena<Variant>,
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
            .rposition(|n| n == name)
            .ok_or_else(|| Error::name(name))?;
        Ok(&self.values[i])
    }

    pub(crate) fn add_package(&mut self, pkg: Arc<Package>) {
        self.packages.push(pkg);
    }

    pub(crate) fn push(&mut self) {
        self.heads.push(self.values.len());
    }

    pub(crate) fn pop(&mut self) {
        if let Some(head) = self.heads.pop() {
            self.values.truncate(head);
        }
    }

    pub(crate) fn boxed(&mut self, value: Variant) -> Index<Variant> {
        self.arena.insert(value)
    }

    pub(crate) fn get(&self, idx: Index<Variant>) -> Option<&Variant> {
        self.arena.get(idx)
    }

    pub(crate) fn add_function(&mut self, f: Function) -> Index<Function> {
        self.fn_arena.insert(f)
    }

    pub(crate) fn get_function(&self, idx: Index<Function>) -> Option<&Function> {
        self.fn_arena.get(idx)
    }
}

pub struct Package {
    pub(crate) environment: Environment,
}
