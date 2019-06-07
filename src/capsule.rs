use std::ops::{Deref, DerefMut};

use crate::data::Symbol;
use crate::environment::{Environment, Value};
use crate::error::{Error, Fallible};
use crate::eval::Evaluate;
use crate::program::ScriptProgram;

pub struct Capsule {
    pub(crate) environments: Vec<Environment>,
}

impl Capsule {
    pub fn interactive() -> Capsule {
        Capsule {
            environments: vec![Environment::default()],
        }
    }

    pub fn eval<T>(&mut self, code: &T) -> Fallible<T::Value>
    where
        T: Evaluate,
    {
        code.eval(&mut self.context())
    }

    pub(crate) fn context(&mut self) -> Context<'_> {
        Context { capsule: self }
    }

    pub fn execute(&mut self, program: ScriptProgram) -> Fallible<()> {
        program.eval(&mut self.context())
    }
}

pub struct Context<'a> {
    capsule: &'a mut Capsule,
}

impl<'a> Context<'a> {
    pub(crate) fn push(&mut self) -> ContextGuard<'a, '_> {
        ContextGuard::new(self)
    }

    fn environment_mut(&mut self) -> &mut Environment {
        self.capsule
            .environments
            .last_mut()
            .expect("no environment")
    }

    pub(crate) fn bind(&mut self, name: Symbol, value: Value) {
        self.environment_mut().bind(name, value);
    }

    pub(crate) fn lookup(&self, depth: usize, index: usize) -> Fallible<&Value> {
        self._lookup(depth, index).ok_or_else(Error::name)
    }

    fn _lookup(&self, depth: usize, index: usize) -> Option<&Value> {
        let i = self.capsule.environments.len().checked_sub(depth + 1)?;
        self.capsule.environments.get(i)?.values.get(index)
    }
}

pub(crate) struct ContextGuard<'a, 'b>(&'b mut Context<'a>);

impl<'a, 'b> ContextGuard<'a, 'b> {
    fn new(ctx: &'b mut Context<'a>) -> Self {
        ctx.capsule.environments.push(Environment::default());
        ContextGuard(ctx)
    }

    pub(crate) fn load(&mut self, env: &Environment) {
        self.0
            .capsule
            .environments
            .last_mut()
            .expect("no environment")
            .clone_from(env);
    }
}

impl Drop for ContextGuard<'_, '_> {
    fn drop(&mut self) {
        self.0.capsule.environments.pop();
    }
}

impl<'a> Deref for ContextGuard<'a, '_> {
    type Target = Context<'a>;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<'a> DerefMut for ContextGuard<'a, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}
