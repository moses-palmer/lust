use crate::{alloc, ast, exp};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
pub enum Tag {
    A,
    B,
    C,
}
impl crate::val::Tag for Tag {}

pub type Value<'a> = crate::Value<'a, Tag>;
pub type Cons<'a> = crate::Cons<'a, Value<'a>>;

pub struct Context;
pub type Expression = crate::Expression<Command>;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
pub struct Command;

impl<'a> TryFrom<&'a ast::Node> for Command {
    type Error = exp::Error<'a>;

    fn try_from(_value: &'a ast::Node) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl ::std::fmt::Display for Command {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl exp::cmd::Command for Command {
    type Tag = Tag;
    type Context = Context;

    fn name(&self) -> &'static str {
        todo!()
    }

    fn arguments(&self) -> &[exp::Expression<Self>] {
        todo!()
    }

    fn arguments_mut(&mut self) -> &mut [exp::Expression<Self>] {
        todo!()
    }

    fn parse<'a>(
        _head: &'a ast::Node,
        _tail: &'a [ast::Node],
    ) -> ::std::result::Result<exp::Expression<Self>, exp::Error<'a>> {
        todo!()
    }

    fn evaluate<'a, 'b, A>(
        &self,
        _script: &crate::Script<Self>,
        _alloc: &A,
        _ctx: &Self::Context,
        _env: &crate::exp::env::Environment<'a, 'b, Self>,
    ) -> exp::Result<'a, Self>
    where
        A: alloc::Allocator<'a, Item = Cons<'a>> + 'a,
        Self::Tag: 'a,
    {
        todo!()
    }
}

/// Parses a string into an AST node.
///
/// # Panics
/// This function will panic if the string is invalid.
pub fn parse(script: &str) -> ast::Node {
    ast::parse(&mut ast::tokenize(script)).unwrap()
}
