//! # Script commands
//!
//! The [`Command`] trait is the means to have a script execute anything; its
//! [`evalue`](Command::evaluate) method is the entry point into native code.

use std::sync::atomic::{AtomicIsize, Ordering};

use crate::{Script, Serializable, alloc, ast};

use super::{Error, Expression, Result, env::Environment};

/// The context for a command evaluation.
pub trait Context {
    /// Evaluation of a single expression is about to begin.
    ///
    /// If an error is returned, evaluation is stopped and the error bubbles up.
    ///
    /// This will be called every time a subexpression is evaluated or a command is executed.
    fn on_evaluate<'a>(&self) -> ::std::result::Result<(), Error<'a>> {
        Ok(())
    }
}

impl Context for () {}

/// A context that constrains resources for evaluation.
///
/// Each evaluation decrements a counter, and once it reaches 0, an error is returned.
pub struct ResourceConstrainer(AtomicIsize);

impl From<isize> for ResourceConstrainer {
    fn from(value: isize) -> Self {
        Self(AtomicIsize::from(value))
    }
}

impl Context for ResourceConstrainer {
    fn on_evaluate<'a>(&self) -> ::std::result::Result<(), Error<'a>> {
        // We use an atomic isize just to enable mutability for a shared reference, not for
        // concurrency, so relaxed ordering is enough
        let previous = self.0.fetch_sub(1, Ordering::Relaxed);

        if previous <= 0 {
            Err(Error::ExecutionLimited)
        } else {
            Ok(())
        }
    }
}

/// A in command for a script.
pub trait Command:
    Clone + ::std::fmt::Debug + ::std::fmt::Display + PartialEq + Serializable
{
    /// An opaque tag.
    type Tag: crate::val::Tag;

    /// The context passed when evaluating a command.
    type Context: Context;

    /// The name of this command.
    fn name(&self) -> &'static str;

    /// The arguments provided to this command.
    fn arguments(&self) -> &[Expression<Self>];

    /// The arguments provided to this command.
    fn arguments_mut(&mut self) -> &mut [Expression<Self>];

    /// Parses an AST node into an expression.
    ///
    /// # Arguments
    /// *  `head` - The head of the AST node, which is the command name.
    /// *  `tail` - The tail of the AST node, which are the arguments.
    fn parse<'a>(
        head: &'a ast::Node,
        tail: &'a [ast::Node],
    ) -> ::std::result::Result<Expression<Self>, Error<'a>>;

    /// Evaluates this expression with a runner in an environment.
    ///
    /// # Arguments
    /// *  `script` - The linked script.
    /// *  `ctx` - The evaluation context.
    /// *  `env` - The environment.
    fn evaluate<'a, 'b, A>(
        &'a self,
        script: &'a Script<Self>,
        alloc: &A,
        ctx: &Self::Context,
        env: &Environment<'a, 'b, Self>,
    ) -> Result<'a, Self>
    where
        A: alloc::Allocator<'a, Item = crate::Cons<'a, crate::Value<'a, Self::Tag>>> + 'a;
}
