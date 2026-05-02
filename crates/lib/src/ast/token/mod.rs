//! # The string tokeniser
//!
//! This module contains the function [`tokenize`] to convert a string slice to a sequence of
//! tokens.
//!
//! A token is one of left and right parentheses, a single quote, a string or number literal or an
//! atom.
//!
//! The value of a token is its string representation, and references the original string.
//!
//! ## String values
//!
//! A string value is delimited by the quote (`"`) character, and may contain a limited number of
//! escaped characters. These are:
//!
//! * `\"` - In order to include a quote character in a string, it must be escaped.
//! * `\\` - In orer to include an escape character in a string, it must be escaped.
//!
//! Any other character is kept as-is.
//!
//! ## Number values
//!
//! A number value is any token starting with a digit, a decimal dot (`.`) or a negative sign
//! (`-`). It may include any number of leading zeroes, but at most one decimal dot.
//!
//! ## Atoms
//!
//! Atoms may contain any ASCII alphanumeric characters as well as `'+'`, `'*'`, `'/'`, `'-'`,
//! `'_'`, `'?'`. They may not start with a digit.
//!
//! ## Examples
//!
//! ```
//! # use lust_lib::ast;
//!
//! let script = "(test \"string\" -123.4 (
//!     hello-world))";
//! let tokens = ast::token::tokenize(script)
//!     .collect::<Result<Vec<_>, _>>()
//!     .map(|tokens| tokens.into_iter()
//!         .map(|token| token.to_string())
//!         .collect::<Vec<_>>())
//!     .unwrap();
//! assert_eq!(
//!     tokens,
//!     vec![
//!         "at 1:1: (",
//!         "at 1:2: test",
//!         "at 1:7: \"string\"",
//!         "at 1:16: -123.4",
//!         "at 1:23: (",
//!         "at 2:5: hello-world",
//!         "at 2:16: )",
//!         "at 2:17: )",
//!     ],
//! );
//! ```
use thiserror::Error;

use crate::ast::PositionedError;

use super::{Position, PositionedErrorCause};

pub(crate) mod tokenizer;

/// An error occurring during tokenisation.
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    /// The input terminated unexpectedly.
    #[error("unexpected end of input")]
    UnexpectedEnd,

    /// An unexpected character was encountered.
    #[error("encountered an unexpected character: '{character}'")]
    UnexpectedCharacter {
        /// The character.
        character: char,
    },
}

impl PositionedErrorCause for Error {}

/// An individual token value.
#[derive(Debug, PartialEq)]
pub enum Value<'a> {
    /// A left parenthesis.
    ///
    /// This is the start of a list.
    LeftParenthesis,

    /// A right parenthesis.
    ///
    /// This is the end of a list.
    RightParenthesis,

    /// A single quote.
    ///
    /// This is shorthand to prevent evaluation of lists.
    Quote,

    /// A string literal.
    ///
    /// A string literal is delimited by quotes (`"`) and permits a limited number of escapes. A
    /// string token will contain the entire string literal, including quotes and escapes.
    String { value: &'a str },

    /// A number.
    ///
    /// A number literal is represented by at most two strings of numbers, separated by a dot (`.`).
    /// There is no limitation on the number of initial zeroes.
    Number { value: &'a str },

    /// An atom
    ///
    /// An atom is any word that starts with a valid atom initial character and contains only valid
    /// characters.
    Atom { value: &'a str },
}

impl<'a> ::std::fmt::Display for Value<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Value::*;
        match self {
            LeftParenthesis => write!(f, "{}", tokenizer::Tokenizer::LEFT_PARENTHESIS),
            RightParenthesis => write!(f, "{}", tokenizer::Tokenizer::RIGHT_PARENTHESIS),
            Quote => write!(f, "{}", tokenizer::Tokenizer::QUOTE),
            String { value } | Number { value } | Atom { value } => write!(f, "{}", value),
        }
    }
}

/// An input token.
#[derive(Debug, PartialEq)]
pub struct Token<'a> {
    /// The token value.
    value: Value<'a>,

    /// The position in the input string.
    position: Position,
}

impl<'a> Token<'a> {
    /// The value.
    #[inline]
    pub fn value(&self) -> &Value<'a> {
        &self.value
    }

    /// The position.
    #[inline]
    pub fn position(&self) -> &Position {
        &self.position
    }
}

impl<'a> ::std::fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "at {}: {}", self.position, self.value)
    }
}

/// The result of tokeniser operations.
pub type Result<'a> = ::std::result::Result<Token<'a>, PositionedError<Error>>;

/// Tokenises a string.
///
/// The iterator will yield tokens until an error is encountered, after which it will terminate.
///
/// # Arguments
/// *  `input` - The input string to tokenise.
pub fn tokenize<'a>(input: &'a str) -> impl Iterator<Item = Result<'a>> + 'a {
    tokenizer::Tokenizer::from(input)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    use crate::ast::PositionValue;

    #[test]
    fn empty() {
        // Arrange
        let input = "";
        let expected = Vec::<Token>::new();

        // Act
        let actual = tokenize(input)
            .collect::<::std::result::Result<Vec<_>, _>>()
            .expect("successful tokenisation");

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn single_parenthesis() {
        // Arrange
        let input = "(";
        let expected = vec![lparen(pos(1, 1))];

        // Act
        let actual = tokenize(input)
            .collect::<::std::result::Result<Vec<_>, _>>()
            .expect("successful tokenisation");

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn single_parenthesis_newline() {
        // Arrange
        let input = "(
)
)";
        let expected = vec![lparen(pos(1, 1)), rparen(pos(2, 1)), rparen(pos(3, 1))];

        // Act
        let actual = tokenize(input)
            .collect::<::std::result::Result<Vec<_>, _>>()
            .expect("successful tokenisation");

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn single_parenthesis_with_whitespace() {
        // Arrange
        let input = " ( ";
        let expected = vec![lparen(pos(1, 2))];

        // Act
        let actual = tokenize(input)
            .collect::<::std::result::Result<Vec<_>, _>>()
            .expect("successful tokenisation");

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn multiple_parentheses() {
        // Arrange
        let input = "(
            ( )";
        let expected = vec![lparen(pos(1, 1)), lparen(pos(2, 13)), rparen(pos(2, 15))];

        // Act
        let actual = tokenize(input)
            .collect::<::std::result::Result<Vec<_>, _>>()
            .expect("successful tokenisation");

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn single_number() {
        // Arrange
        let input = "123.4";
        let expected = vec![number(pos(1, 1), "123.4")];

        // Act
        let actual = tokenize(input)
            .collect::<::std::result::Result<Vec<_>, _>>()
            .expect("successful tokenisation");

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn multiple_numbers() {
        // Arrange
        let input = "123.4
            567.8 -92";
        let expected = vec![
            number(pos(1, 1), "123.4"),
            number(pos(2, 13), "567.8"),
            number(pos(2, 19), "-92"),
        ];

        // Act
        let actual = tokenize(input)
            .collect::<::std::result::Result<Vec<_>, _>>()
            .expect("successful tokenisation");

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn single_string() {
        // Arrange
        let input = "\"hello world\"";
        let expected = vec![string(pos(1, 1), "\"hello world\"")];

        // Act
        let actual = tokenize(input)
            .collect::<::std::result::Result<Vec<_>, _>>()
            .expect("successful tokenisation");

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn multiple_strings() {
        // Arrange
        let input = "\"hello\"
            \"world again\"";
        let expected = vec![
            string(pos(1, 1), "\"hello\""),
            string(pos(2, 13), "\"world again\""),
        ];

        // Act
        let actual = tokenize(input)
            .collect::<::std::result::Result<Vec<_>, _>>()
            .expect("successful tokenisation");

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn single_atom() {
        // Arrange
        let input = "hello-world";
        let expected = vec![atom(pos(1, 1), "hello-world")];

        // Act
        let actual = tokenize(input)
            .collect::<::std::result::Result<Vec<_>, _>>()
            .expect("successful tokenisation");

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn multiple_atoms() {
        // Arrange
        let input = "hello
            world -again != hello? world!";
        let expected = vec![
            atom(pos(1, 1), "hello"),
            atom(pos(2, 13), "world"),
            atom(pos(2, 19), "-again"),
            atom(pos(2, 26), "!="),
            atom(pos(2, 29), "hello?"),
            atom(pos(2, 36), "world!"),
        ];

        // Act
        let actual = tokenize(input)
            .collect::<::std::result::Result<Vec<_>, _>>()
            .expect("successful tokenisation");

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn trailing_short_atom() {
        // Arrange
        let input = "hello a b";
        let expected = vec![
            atom(pos(1, 1), "hello"),
            atom(pos(1, 7), "a"),
            atom(pos(1, 9), "b"),
        ];

        // Act
        let actual = tokenize(input)
            .map(|i| {
                println!("{i:?}");
                i
            })
            .collect::<::std::result::Result<Vec<_>, _>>()
            .expect("successful tokenisation");

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn complex_expressions() {
        // Arrange
        let input = "(+ a b .123 (
            c d))";
        let expected = vec![
            lparen(pos(1, 1)),
            atom(pos(1, 2), "+"),
            atom(pos(1, 4), "a"),
            atom(pos(1, 6), "b"),
            number(pos(1, 8), ".123"),
            lparen(pos(1, 13)),
            atom(pos(2, 13), "c"),
            atom(pos(2, 15), "d"),
            rparen(pos(2, 16)),
            rparen(pos(2, 17)),
        ];

        // Act
        let actual = tokenize(input)
            .collect::<::std::result::Result<Vec<_>, _>>()
            .expect("successful tokenisation");

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn complex_expressions_with_comment() {
        // Arrange
        let input = "(+ a b ( ; ignore this
            ; also ignore this
            ; and this
            c d))";
        let expected = vec![
            lparen(pos(1, 1)),
            atom(pos(1, 2), "+"),
            atom(pos(1, 4), "a"),
            atom(pos(1, 6), "b"),
            lparen(pos(1, 8)),
            atom(pos(4, 13), "c"),
            atom(pos(4, 15), "d"),
            rparen(pos(4, 16)),
            rparen(pos(4, 17)),
        ];

        // Act
        let actual = tokenize(input)
            .collect::<::std::result::Result<Vec<_>, _>>()
            .expect("successful tokenisation");

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn trailing_comment() {
        // Arrange
        let input = "(+ 4 5) ; 9";
        let expected = vec![
            lparen(pos(1, 1)),
            atom(pos(1, 2), "+"),
            number(pos(1, 4), "4"),
            number(pos(1, 6), "5"),
            rparen(pos(1, 7)),
        ];

        // Act
        let actual = tokenize(input)
            .collect::<::std::result::Result<Vec<_>, _>>()
            .expect("successful tokenisation");

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn invalid_number() {
        // Arrange
        let input = "123.a";
        let expected = vec![Err(
            Error::UnexpectedCharacter { character: 'a' }.for_position(pos(1, 5))
        )];

        // Act
        let actual = tokenize(input).collect::<Vec<_>>();

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn multiple_invalid_numbers() {
        // Arrange
        let input = "123.4
            567.8 92a 5";
        let expected = vec![
            Ok(number(pos(1, 1), "123.4")),
            Ok(number(pos(2, 13), "567.8")),
            Err(Error::UnexpectedCharacter { character: 'a' }.for_position(pos(2, 21))),
        ];

        // Act
        let actual = tokenize(input).collect::<Vec<_>>();

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn invalid_negative_number() {
        // Arrange
        let input = "-123.a";
        let expected = vec![Err(
            Error::UnexpectedCharacter { character: 'a' }.for_position(pos(1, 6))
        )];

        // Act
        let actual = tokenize(input).collect::<Vec<_>>();

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn unterminated_string() {
        // Arrange
        let input = "a \"hello world";
        let expected = vec![
            Ok(atom(pos(1, 1), "a")),
            Err(Error::UnexpectedEnd.for_position(pos(1, 14))),
        ];

        // Act
        let actual = tokenize(input).collect::<Vec<_>>();

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn unexpected_character() {
        // Arrange
        let input = "a \"hello world\" |";
        let expected = vec![
            Ok(atom(pos(1, 1), "a")),
            Ok(string(pos(1, 3), "\"hello world\"")),
            Err(Error::UnexpectedCharacter { character: '|' }.for_position(pos(1, 17))),
        ];

        // Act
        let actual = tokenize(input).collect::<Vec<_>>();

        // Assert
        assert_eq!(expected, actual);
    }

    pub fn pos(row: PositionValue, column: PositionValue) -> Position {
        Position { row, column }
    }

    pub fn lparen(position: Position) -> Token<'static> {
        Token {
            value: Value::LeftParenthesis,
            position,
        }
    }

    pub fn rparen(position: Position) -> Token<'static> {
        Token {
            value: Value::RightParenthesis,
            position,
        }
    }

    pub fn number<'a>(position: Position, value: &'a str) -> Token<'a> {
        Token {
            value: Value::Number { value },
            position,
        }
    }

    pub fn string<'a>(position: Position, value: &'a str) -> Token<'a> {
        Token {
            value: Value::String { value },
            position,
        }
    }

    pub fn atom<'a>(position: Position, value: &'a str) -> Token<'a> {
        Token {
            value: Value::Atom { value },
            position,
        }
    }
}
