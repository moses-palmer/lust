//! # The abstract syntax tree
//!
//! This module contains functions to tokenise and parse a script.
//!
//! ## Examples
//!
//! ```
//! # use lust_lib::ast;
//!
//! let script = "(test-a-script 5 4 (
//!     \"hello\"))";
//! let root = ast::parse(&mut ast::token::tokenize(script))
//!     .unwrap();
//! assert_eq!(
//!     root.to_string(),
//!     "(test-a-script 5 4 (\"hello\"))"
//! );
//! ```
use thiserror::Error;

use token::tokenizer::{StringTokenizer, Tokenizer};

pub mod parser;
pub mod token;

pub use parser::parse;
pub use token::tokenize;

/// The value of a single [`Position`] value.
pub type PositionValue = u16;

/// A position in the input string.
///
/// This is used for error reporting when parsing, compiling and evaluating.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Position {
    /// The row.
    ///
    /// This is 1 based.
    row: PositionValue,

    /// The column.
    ///
    /// This is 1 based.
    column: PositionValue,
}

impl Position {
    /// Creates a new value.
    ///
    /// # Arguments
    /// *  `row` - The row.
    /// *  `column`- The column.
    pub const fn new(row: PositionValue, column: PositionValue) -> Self {
        Self { row, column }
    }

    /// The starting position.
    pub fn start() -> Self {
        Self { row: 1, column: 1 }
    }

    /// The row of this position.
    #[inline]
    pub fn row(&self) -> PositionValue {
        self.row
    }

    /// The column of this position.
    #[inline]
    pub fn column(&self) -> PositionValue {
        self.column
    }

    /// The beginning of the next row.
    #[inline]
    pub fn next_row(self) -> Self {
        Self {
            row: self.row + 1,
            column: 0,
        }
    }

    /// The next column.
    #[inline]
    pub fn next_column(self) -> Self {
        Self {
            column: self.column + 1,
            ..self
        }
    }
}

impl ::std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.row, self.column)
    }
}

/// An error originating at a specific location.
#[derive(Debug, PartialEq, Error)]
#[error("at {position}: {cause}")]
pub struct PositionedError<E> {
    /// The position of the error.
    position: Position,

    /// The cause of the error.
    cause: E,
}

impl<E> PositionedError<E> {
    #[inline]
    pub fn position(&self) -> Position {
        self.position
    }

    #[inline]
    pub fn cause(&self) -> &E {
        &self.cause
    }
}

/// A cause for a [positioned error](PositionedError).
pub trait PositionedErrorCause: Sized + ::std::error::Error {
    /// Constructs a [`PositionedError`] from this cause.
    ///
    /// # Arguments
    /// *  `position` - The position.
    fn for_position(self, position: Position) -> PositionedError<Self> {
        PositionedError {
            position,
            cause: self,
        }
    }
}

/// A single value.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Value {
    /// An atom.
    Atom { value: String },

    /// A number.
    Number { value: f32 },

    /// A string.
    String { value: String },
}

impl ::std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Value::*;
        match self {
            Atom { value, .. } => write!(f, "{value}"),
            Number { value, .. } => write!(f, "{value}"),
            String { value, .. } => {
                write!(f, "\"")?;
                for character in value.chars() {
                    if StringTokenizer::ESCAPABLES.contains(&character) {
                        write!(f, "{}{character}", StringTokenizer::ESCAPE)?;
                    } else {
                        write!(f, "{character}")?;
                    }
                }
                write!(f, "\"")
            }
        }
    }
}

/// An AST node value.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum NodeValue {
    /// A leaf node
    Leaf(Value),

    /// A tree node.
    Tree(Vec<Node>),
}

impl ::std::fmt::Display for NodeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use NodeValue::*;
        match &self {
            Leaf(value) => write!(f, "{}", value),
            Tree(value) => crate::common::write_list(value.iter(), f),
        }
    }
}

/// An AST node.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Node {
    /// The original token position.
    position: Position,

    /// Whether this node is quoted.
    quoted: bool,

    /// The node value.
    pub value: NodeValue,
}

impl Node {
    /// Creates a new AST node.
    pub const fn new(position: Position, quoted: bool, value: NodeValue) -> Self {
        Self {
            position,
            quoted,
            value,
        }
    }

    /// The length of this node.
    ///
    /// This does not include child nodes
    #[inline]
    pub fn len(&self) -> usize {
        match &self.value {
            NodeValue::Leaf(_) => 1,
            NodeValue::Tree(nodes) => nodes.len(),
        }
    }

    /// Whether this node is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The position.
    #[inline]
    pub fn position(&self) -> &Position {
        &self.position
    }

    /// Whether this node is quoted.
    #[inline]
    pub fn quoted(&self) -> bool {
        self.quoted
    }

    /// The value.
    #[inline]
    pub fn value(&self) -> &NodeValue {
        &self.value
    }

    /// This node unquoted.
    pub fn unquoted(self) -> Self {
        Self {
            quoted: false,
            ..self
        }
    }
}

impl ::std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.quoted {
            write!(f, "{}", Tokenizer::QUOTE)?;
        }
        self.value.fmt(f)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::ast::Position;

    use super::*;

    pub fn atom(value: &str) -> Value {
        Value::Atom {
            value: value.into(),
        }
    }

    pub fn number(value: f32) -> Value {
        Value::Number { value }
    }

    pub fn string(value: &str) -> Value {
        Value::String {
            value: value.into(),
        }
    }

    pub fn leaf(position: Position, value: Value) -> Node {
        Node {
            position,
            quoted: false,
            value: NodeValue::Leaf(value),
        }
    }

    pub fn leaf_quoted(position: Position, value: Value) -> Node {
        Node {
            position,
            quoted: true,
            value: NodeValue::Leaf(value),
        }
    }

    pub fn tree(position: Position, nodes: &[Node]) -> Node {
        Node {
            position,
            quoted: false,
            value: NodeValue::Tree(nodes.iter().cloned().collect()),
        }
    }

    pub fn tree_quoted(position: Position, nodes: &[Node]) -> Node {
        Node {
            position,
            quoted: true,
            value: NodeValue::Tree(nodes.iter().cloned().collect()),
        }
    }
}
