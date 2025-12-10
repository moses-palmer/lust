use crate::ast;

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
pub enum Tag {
    A,
    B,
    C,
}
impl crate::val::Tag for Tag {}

pub type Value<'a> = crate::Value<'a, Tag>;

/// Parses a string into an AST node.
///
/// # Panics
/// This function will panic if the string is invalid.
pub fn parse(script: &str) -> ast::Node {
    ast::parse(&mut ast::tokenize(script)).unwrap()
}
