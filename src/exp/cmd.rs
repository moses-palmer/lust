//! # Script commands
//!
//! The [`Command`] trait is the means to have a script execute anything; its
//! [`evalue`](Command::evaluate) method is the entry point into native code.

use crate::{Script, Serializable, ast};

use super::{Error, Expression, Result, env::Environment};

/// A in command for a script.
pub trait Command:
    Clone + ::std::fmt::Debug + ::std::fmt::Display + PartialEq + Serializable
{
    /// An opaque tag.
    type Tag: crate::val::Tag;

    /// The context passed when evaluating a command.
    type Context;

    /// The name of this command.
    fn name(&self) -> &'static str;

    /// Parses an AST node into an expression.
    fn parse<'a>(node: &'a ast::Node) -> ::std::result::Result<Expression<Self>, Error<'a>>;

    /// Evaluates this expression with a runner in an environment.
    ///
    /// # Arguments
    /// *  `script` - The linked script.
    /// *  `ctx` - The evaluation context.
    /// *  `env` - The environment.
    fn evaluate<'a, 'b>(
        &'a self,
        script: &'a Script<Self>,
        ctx: &Self::Context,
        env: &Environment<'a, 'b, Self>,
    ) -> Result<'a, Self>;
}
