use crate::{ast, exp};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
pub enum Tag {
    A,
    B,
    C,
}
impl crate::val::Tag for Tag {}

pub type Value<'a> = crate::Value<'a, Tag>;

pub struct Context;

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

    fn parse<'a>(
        _node: &'a ast::Node,
    ) -> ::std::result::Result<exp::Expression<Self>, exp::Error<'a>> {
        todo!()
    }

    fn evaluate<'a, 'b>(
        &self,
        _script: &crate::Script<Self>,
        _ctx: &Self::Context,
        _env: &crate::exp::env::Environment<'a, 'b, Self>,
    ) -> exp::Result<'a, Self> {
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
