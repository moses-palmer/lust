use crate::{Command, Environment, Value, val};

/// A linked expression.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Script<C> {
    /// The root expression.
    root: super::Expression<C>,
}

impl<C> Script<C>
where
    C: Command,
{
    /// Evaluates the root expression given a context and environment.
    ///
    /// # Arguments
    /// *  `ctx` - The context of the evaluation.
    /// *  `env` - The current variable scope.
    pub fn evaluate<'a, 'b>(
        &'a self,
        ctx: &C::Context,
        env: &Environment<'a, 'b, C>,
    ) -> Result<val::owned::Value<C::Tag>, super::Error<'a>> {
        self.value(&self.root, ctx, env)
            .and_then(|v| Ok(v.try_into()?))
    }

    /// Evaluates an expression given a context and environment.
    ///
    /// # Arguments
    /// *  `ctx` - The context of the evaluation.
    /// *  `env` - The current variable scope.
    pub fn value<'a, 'b>(
        &'a self,
        e: &'a super::Expression<C>,
        ctx: &C::Context,
        env: &Environment<'a, 'b, C>,
    ) -> super::Result<'a, C> {
        use super::Expression::*;
        match e {
            List(_) => Err(val::Error::Operation("cannot evaluate list").into()),
            AST(v) => Ok(Value::AST(v)),
            Reference(v) => env
                .resolve(&v)
                .ok_or_else(|| super::Error::UnknownReference { value: v.clone() }),
            Boolean(v) => Ok((*v).into()),
            Number(v) => Ok((*v).into()),
            String(v) => Ok(v.as_str().into()),
            Command(v) => v.evaluate(self, ctx, env),
        }
    }
}

impl<C> Default for Script<C> {
    fn default() -> Self {
        Self {
            root: Default::default(),
        }
    }
}

impl<C> ::std::fmt::Display for Script<C>
where
    C: ::std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.root.fmt(f)
    }
}

impl<C> From<super::Expression<C>> for Script<C>
where
    C: Command,
{
    fn from(value: super::Expression<C>) -> Self {
        Self { root: value }
    }
}
