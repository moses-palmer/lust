use thiserror::Error;

use super::{
    Node, NodeValue, Position, PositionedError, PositionedErrorCause, Value,
    token::{self, tokenizer::StringTokenizer},
};

/// An error occurring during transformation of a token stream to an abstract syntax tree.
#[derive(Debug, Error, PartialEq)]
pub enum Error<'a> {
    /// An error occurring during tokenisation.
    #[error("tokenizer error: {cause}")]
    TokenizerError {
        #[from]
        cause: PositionedError<token::Error>,
    },

    /// The tokens terminated unexpectedly.
    #[error("unexpected end of tokens")]
    UnexpectedEnd,

    /// An unexpected token was encountered.
    #[error("unexpected token: {token}")]
    UnexpectedToken { token: token::Token<'a> },
}

impl<'a> PositionedErrorCause for Error<'a> {}

/// The result of parser operations.
pub type Result<'a> = ::std::result::Result<Node, Error<'a>>;

/// Parses a stream of tokens into an AST.
///
/// # Arguments
/// *  `tokens` - The token stream.
pub fn parse<'a>(tokens: &mut impl Iterator<Item = token::Result<'a>>) -> Result<'a> {
    let ast = parse_first(tokens, false)?;

    if let Some(token) = tokens.next().transpose()? {
        Err(Error::UnexpectedToken { token })
    } else {
        Ok(ast)
    }
}

/// Parses a stream of tokens into an AST.
///
/// # Arguments
/// *  `tokens` - The token stream.
/// *  `quoted` - Whether the following expression is quoted.
fn parse_first<'a>(
    tokens: &mut impl Iterator<Item = token::Result<'a>>,
    quoted: bool,
) -> Result<'a> {
    use token::Value::*;

    match tokens.next().transpose()? {
        Some(token) => match token.value() {
            LeftParenthesis => parse_tree(*token.position(), quoted, tokens),
            Quote => parse_first(tokens, true),
            String { value } if !quoted => Ok(parse_string(*token.position(), value)),
            Number { value } if !quoted => Ok(parse_number(*token.position(), value)),
            Atom { value } => Ok(parse_atom(*token.position(), value, quoted)),
            _ => Err(Error::UnexpectedToken { token }),
        },
        None => Err(Error::UnexpectedEnd),
    }
}

/// Parses a tree where the initial parenthesis has already been consumed.
///
/// # Arguments
/// *  `position` - The position of the start of the tree.
/// *  `tokens` - The token stream.
/// *  `quoted` - Whether the expression is quoted.
fn parse_tree<'a>(
    position: Position,
    quoted: bool,
    tokens: &mut impl Iterator<Item = token::Result<'a>>,
) -> Result<'a> {
    use token::Value::*;

    let mut nodes = Vec::new();
    Ok(Node {
        position,
        quoted,
        value: NodeValue::Tree(loop {
            match parse_first(tokens, false) {
                Ok(node) => nodes.push(node),
                Err(Error::UnexpectedToken { token })
                    if matches!(token.value(), RightParenthesis) =>
                {
                    break Ok(nodes);
                }
                Err(e) => break Err(e),
            }
        }?),
    })
}

/// Parses a single string, unescaping escaped characters.
///
/// # Arguments
/// *  `position` - The token position.
/// *  `value` - The string value to parse.
/// *  `quoted` - Whether the expression is quoted.
fn parse_string(position: Position, value: &str) -> Node {
    Node {
        position,
        quoted: false,
        value: NodeValue::Leaf(Value::String {
            value: value
                .chars()
                .skip(1)
                .take(value.len() - 2)
                .fold(
                    (String::with_capacity(value.len() - 2), false),
                    |(mut string, escaped), character| {
                        if escaped {
                            (string, false)
                        } else if character == StringTokenizer::ESCAPE {
                            (string, true)
                        } else {
                            string.push(character);
                            (string, false)
                        }
                    },
                )
                .0,
        }),
    }
}

/// Parses a single number.
///
/// # Arguments
/// *  `position` - The token position.
/// *  `value` - The number value to parse.
/// *  `quoted` - Whether the expression is quoted.
fn parse_number(position: Position, value: &str) -> Node {
    Node {
        position,
        quoted: false,
        value: NodeValue::Leaf(Value::Number {
            value: value.parse().unwrap(),
        }),
    }
}

/// Parses a single atom.
///
/// # Arguments
/// *  `position` - The token position.
/// *  `value` - The atom value to parse.
/// *  `quoted` - Whether the expression is quoted.
fn parse_atom(position: Position, value: &str, quoted: bool) -> Node {
    Node {
        position,
        quoted,
        value: NodeValue::Leaf(Value::Atom {
            value: value.into(),
        }),
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::ast::{
        tests::*,
        token::{tests as token, tokenize},
    };

    use super::*;

    #[test]
    fn empty() {
        // Arrange
        let script = "()";
        let expected = tree(token::pos(1, 1), &[]);

        // Act
        let actual = parse(&mut tokenize(script)).unwrap();

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn single_token() {
        // Arrange
        let script = "(single-atom)";
        let expected = tree(
            token::pos(1, 1),
            &[leaf(token::pos(1, 2), atom("single-atom"))],
        );

        // Act
        let actual = parse(&mut tokenize(script)).unwrap();

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn single_quoted_token() {
        // Arrange
        let script = "('single-atom)";
        let expected = tree(
            token::pos(1, 1),
            &[leaf_quoted(token::pos(1, 3), atom("single-atom"))],
        );

        // Act
        let actual = parse(&mut tokenize(script)).unwrap();

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn multiple_tokens() {
        // Arrange
        let script = "(atom 123.4 \"string\")";
        let expected = tree(
            token::pos(1, 1),
            &[
                leaf(token::pos(1, 2), atom("atom")),
                leaf(token::pos(1, 7), number(123.4)),
                leaf(token::pos(1, 13), string("string")),
            ],
        );

        // Act
        let actual = parse(&mut tokenize(script)).unwrap();

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn multiple_trees() {
        // Arrange
        let script = "(atom (123.4 \"string\" (a b (c))))";
        let expected = tree(
            token::pos(1, 1),
            &[
                leaf(token::pos(1, 2), atom("atom")),
                tree(
                    token::pos(1, 7),
                    &[
                        leaf(token::pos(1, 8), number(123.4)),
                        leaf(token::pos(1, 14), string("string")),
                        tree(
                            token::pos(1, 23),
                            &[
                                leaf(token::pos(1, 24), atom("a")),
                                leaf(token::pos(1, 26), atom("b")),
                                tree(token::pos(1, 28), &[leaf(token::pos(1, 29), atom("c"))]),
                            ],
                        ),
                    ],
                ),
            ],
        );

        // Act
        let actual = parse(&mut tokenize(script)).unwrap();

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn quoted_multiple_trees() {
        // Arrange
        let script = "(atom (123.4 \"string\" '(a b (c))))";
        let expected = tree(
            token::pos(1, 1),
            &[
                leaf(token::pos(1, 2), atom("atom")),
                tree(
                    token::pos(1, 7),
                    &[
                        leaf(token::pos(1, 8), number(123.4)),
                        leaf(token::pos(1, 14), string("string")),
                        tree_quoted(
                            token::pos(1, 24),
                            &[
                                leaf(token::pos(1, 25), atom("a")),
                                leaf(token::pos(1, 27), atom("b")),
                                tree(token::pos(1, 29), &[leaf(token::pos(1, 30), atom("c"))]),
                            ],
                        ),
                    ],
                ),
            ],
        );

        // Act
        let actual = parse(&mut tokenize(script)).unwrap();

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn unterminated_tree() {
        // Arrange
        let script = "(atom (123.4 \"string\")";
        let expected = Err(Error::UnexpectedEnd);

        // Act
        let actual = parse(&mut tokenize(script));

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn additional_parentheses() {
        // Arrange
        let script = "(atom (123.4 \"string\")))";
        let expected = Err(Error::UnexpectedToken {
            token: token::rparen(token::pos(1, 24)),
        });

        // Act
        let actual = parse(&mut tokenize(script));

        // Assert
        assert_eq!(expected, actual);
    }
}
