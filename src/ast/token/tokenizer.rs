use std::str::CharIndices;

use crate::ast::PositionedError;

use super::{Error, Position, PositionedErrorCause, Token, Value};

/// A tokeniser.
pub(crate) struct Tokenizer<'a> {
    /// The input string.
    input: &'a str,

    /// The characters of the input string.
    characters: CharIndices<'a>,

    /// The current character.
    current: Option<(usize, char)>,

    /// The current position.
    position: Position,
}

impl<'a> Tokenizer<'a> {
    /// The comment character.
    pub const COMMENT: char = ';';

    /// A newline.
    pub const NEWLINE: char = '\n';

    /// A left parenthesis.
    pub const LEFT_PARENTHESIS: char = '(';

    /// A right parenthesis.
    pub const RIGHT_PARENTHESIS: char = ')';

    /// A quote.
    pub const QUOTE: char = '\'';

    /// Moves the tokeniser one character and returns the current character, if any.
    fn next(&mut self) -> Result<(usize, char), PositionedError<Error>> {
        // If the current character is a newline, skip to the next row
        if let Some((_, Self::NEWLINE)) = self.current {
            self.position = self.position.next_row();
        }

        if let Some(value) = self.characters.next() {
            self.current = Some(value);
            self.position = self.position.next_column();
            Ok(value)
        } else {
            self.current = None;
            Err(Error::UnexpectedEnd.for_position(self.position))
        }
    }

    /// Expects the current character to fulfill a predicate.
    ///
    /// This method does not move forward.
    ///
    /// # Arguments
    /// *  `check` - The check to perform.
    fn expect<F>(&mut self, check: F) -> Result<(), PositionedError<Error>>
    where
        F: Fn(char) -> bool,
    {
        if let Some((_, character)) = self.current {
            if check(character) {
                Ok(())
            } else {
                Err(self.terminate(Error::UnexpectedCharacter { character }))
            }
        } else {
            Err(Error::UnexpectedEnd.for_position(self.position))
        }
    }

    /// Retrieves the byte index of the current character.
    ///
    /// This method does not move forward.
    fn index(&mut self) -> Result<usize, PositionedError<Error>> {
        if let Some(value) = self.current {
            Ok(value.0)
        } else {
            Err(Error::UnexpectedEnd.for_position(self.position))
        }
    }

    /// Skips to the end of the current line.
    fn skip_line(&mut self) {
        loop {
            match self.next() {
                Ok((_, Self::NEWLINE))
                | Err(PositionedError {
                    cause: Error::UnexpectedEnd,
                    ..
                }) => break,
                _ => continue,
            }
        }
    }

    /// Skips whitespace from the current position and returns the last token.
    ///
    /// If a comment is encountered, it will also be skipped.
    fn skip_until_next(&mut self) -> Option<(usize, char)> {
        while self
            .current
            .map(|(_, character)| character.is_ascii_whitespace() || character == Self::COMMENT)
            .unwrap_or(false)
        {
            if let Ok((_, Self::COMMENT)) = self.next() {
                self.skip_line();
            }
        }

        self.current
    }

    /// Skips to the end of the input and returns an error.
    ///
    /// # Arguments
    /// *  `err` - The error to return.
    fn terminate(&mut self, err: Error) -> PositionedError<Error> {
        let position = self.position;
        for _ in self.characters.by_ref() {}
        self.current = None;
        err.for_position(position)
    }

    /// Reads a left parenthesis.
    ///
    /// This method moves to the character after the token.
    fn left_parenthesis(&mut self) -> super::Result<'a> {
        self.expect(|c| c == Self::LEFT_PARENTHESIS)?;
        let result = Token {
            value: Value::LeftParenthesis,
            position: self.position,
        };
        let _ = self.next();
        Ok(result)
    }

    /// Reads a right parenthesis.
    ///
    /// This method moves to the character after the token.
    fn right_parenthesis(&mut self) -> super::Result<'a> {
        self.expect(|c| c == Self::RIGHT_PARENTHESIS)?;
        let result = Token {
            value: Value::RightParenthesis,
            position: self.position,
        };
        let _ = self.next();
        Ok(result)
    }

    /// Reads a quote.
    ///
    /// This method moves to the character after the token.
    fn quote(&mut self) -> super::Result<'a> {
        self.expect(|c| c == Self::QUOTE)?;
        let result = Token {
            value: Value::Quote,
            position: self.position,
        };
        let _ = self.next();
        Ok(result)
    }

    /// Reads a string literal.
    ///
    /// This method moves to the character after the token.
    ///
    /// If the string literal is not closed, an error is returned and the input stream is closed.
    fn string(&mut self, position: Position, start: usize) -> super::Result<'a> {
        use StringTokenizer::*;
        let mut state = Start;
        let end = loop {
            state = state.consume(self)?;
            match state {
                End(index) => break index,
                _ => continue,
            }
        };

        Ok(Token {
            value: Value::String {
                value: &self.input[start..=end],
            },
            position,
        })
    }

    /// Reads a number literal.
    ///
    /// This method moves to the character after the token.
    ///
    /// If the number literal is invalid, an error is returned and the input stream is terminated.
    fn number(&mut self, position: Position, start: usize) -> super::Result<'a> {
        use NumberTokenizer::*;
        let mut state = Start;
        let (negative, end) = loop {
            state = state.consume(self)?;
            match state {
                IsAtom => return self.atom(position, start),
                End(negative, end) => break (negative, end),
                _ => continue,
            }
        };

        if negative && start == end {
            Ok(Token {
                value: Value::Atom {
                    value: &self.input[start..=end],
                },
                position,
            })
        } else {
            Ok(Token {
                value: Value::Number {
                    value: &self.input[start..=end],
                },
                position,
            })
        }
    }

    /// Reads an atom.
    ///
    /// This method moves to the character after the token.
    ///
    /// If the atom is invalid, an error is returned and the input stream is terminated.
    fn atom(&mut self, position: Position, start: usize) -> super::Result<'a> {
        use AtomTokenizer::*;
        let mut state = Start;
        let end = loop {
            state = state.consume(self)?;
            match state {
                End(index) => break index,
                _ => continue,
            }
        };

        Ok(Token {
            value: Value::Atom {
                value: &self.input[start..=end],
            },
            position,
        })
    }
}

impl<'a> From<&'a str> for Tokenizer<'a> {
    fn from(value: &'a str) -> Self {
        let mut characters = value.char_indices();
        let current = characters.next();
        Self {
            input: value,
            characters,
            current,
            position: Position::start(),
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = super::Result<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_until_next().map(|v| match v {
            (_, Self::LEFT_PARENTHESIS) => self.left_parenthesis(),
            (_, Self::RIGHT_PARENTHESIS) => self.right_parenthesis(),
            (_, Self::QUOTE) => self.quote(),

            (i, c) if StringTokenizer::is_start(c) => {
                let position = self.position;
                self.string(position, i)
            }
            (i, c) if NumberTokenizer::is_start(c) => {
                let position = self.position;
                self.number(position, i)
            }
            (i, c) if AtomTokenizer::is_start(c) => {
                let position = self.position;
                self.atom(position, i)
            }

            (_, c) => Err(self.terminate(Error::UnexpectedCharacter { character: c })),
        })
    }
}

/// A tokenizer for strings.
pub(crate) enum StringTokenizer {
    /// The initial state.
    Start,

    /// No escape active.
    Normal,

    /// The current character is escaped.
    Escaped,

    /// We have reached the end of the string.
    ///
    /// Once this state is reached, the tokeniser cannot be used any more.
    End(usize),
}

impl StringTokenizer {
    /// Escapable characters.
    pub const ESCAPABLES: &'static [char] = &[Self::QUOTE, Self::ESCAPE];

    /// The quote character.
    pub const QUOTE: char = '"';

    /// The escape character.
    pub const ESCAPE: char = '\\';

    /// Consumes a single character from the input.
    ///
    /// # Arguments
    /// *  `tokenizer` - The tokenizer.
    fn consume<'a>(self, tokenizer: &mut Tokenizer<'a>) -> Result<Self, PositionedError<Error>> {
        use StringTokenizer::*;
        match self {
            Start => {
                tokenizer.expect(Self::is_start)?;
                tokenizer.next()?;
                Ok(Normal)
            }
            Normal => match tokenizer.next()? {
                (_, Self::ESCAPE) => Ok(Escaped),
                (i, Self::QUOTE) => {
                    let _ = tokenizer.next();
                    Ok(End(i))
                }
                _ => Ok(Normal),
            },
            Escaped => {
                tokenizer.next()?;
                tokenizer.expect(|c| Self::ESCAPABLES.contains(&c))?;
                Ok(Normal)
            }
            End(_) => Err(tokenizer.terminate(Error::UnexpectedEnd)),
        }
    }

    /// Determines whether a character is the start of a string.
    ///
    /// # Arguments
    /// *  `character` - The character to check.
    fn is_start(character: char) -> bool {
        character == Self::QUOTE
    }
}

/// A tokenizer for numbers.
pub(crate) enum NumberTokenizer {
    /// The initial state.
    Start,

    /// We are parsing the integer part.
    ///
    /// The contained value indicates that the value is negative.
    IntegerPart(bool),

    /// The dot between the integer and fractional parts.
    Dot(bool),

    /// We are parsing the fractional part.
    FractionalPart(bool),

    /// The number was in fact an atom.
    IsAtom,

    /// We have reached the end of the string.
    ///
    /// Once this state is reached, the tokeniser cannot be used any more.
    End(bool, usize),
}

impl NumberTokenizer {
    /// The negative number character.
    pub const NEGATIVE: char = '-';

    /// The decimal character.
    pub const DOT: char = '.';

    /// Consumes a single character from the input.
    ///
    /// # Arguments
    /// *  `tokenizer` - The tokenizer.
    fn consume<'a>(self, tokenizer: &mut Tokenizer<'a>) -> Result<Self, PositionedError<Error>> {
        let index = tokenizer.index()?;

        use NumberTokenizer::*;
        match self {
            Start => {
                tokenizer.expect(Self::is_start)?;
                let value = tokenizer.current;
                match value {
                    Some((_, Self::DOT)) => Ok(Dot(false)),
                    Some((_, Self::NEGATIVE)) => Ok(IntegerPart(true)),
                    _ => Ok(IntegerPart(false)),
                }
            }
            IntegerPart(negative) => match tokenizer.next().map(|(_, c)| c).ok() {
                Some(c) if self.is_valid(c) => Ok(match (self, c) {
                    (IntegerPart(_), Self::DOT) => Dot(negative),
                    (IntegerPart(_), _) => IntegerPart(negative),
                    _ => unreachable!(),
                }),
                Some(c) if Self::is_end(c) => Ok(End(negative, index)),
                Some(c) if negative && AtomTokenizer::is_start(c) => Ok(IsAtom),
                Some(c) => Err(tokenizer.terminate(Error::UnexpectedCharacter { character: c })),
                None => Ok(End(negative, index)),
            },
            Dot(negative) | FractionalPart(negative) => {
                match tokenizer.next().map(|(_, c)| c).ok() {
                    Some(c) if self.is_valid(c) => Ok(FractionalPart(negative)),
                    Some(c) if Self::is_end(c) => Ok(End(negative, index)),
                    Some(c) => {
                        Err(tokenizer.terminate(Error::UnexpectedCharacter { character: c }))
                    }
                    None => Ok(End(negative, index)),
                }
            }
            IsAtom | End(_, _) => Err(tokenizer.terminate(Error::UnexpectedEnd)),
        }
    }

    /// Determines whether a character is the start of a string.
    ///
    /// # Arguments
    /// *  `character` - The character to check.
    fn is_start(character: char) -> bool {
        character.is_ascii_digit() || character == Self::NEGATIVE || character == Self::DOT
    }

    /// Determines whether a character is the end of an atom.
    ///
    /// # Arguments
    /// *  `character` - The character to check.
    fn is_end(character: char) -> bool {
        character.is_ascii_whitespace()
            || character == Tokenizer::LEFT_PARENTHESIS
            || character == Tokenizer::RIGHT_PARENTHESIS
    }

    /// Determines whether a character is valid for the current state.
    ///
    /// # Arguments
    /// *  `character` - The character to check.
    fn is_valid(&self, character: char) -> bool {
        use NumberTokenizer::*;
        match self {
            Start => Self::is_start(character),
            IntegerPart(_) => character != Self::NEGATIVE && Self::is_start(character),
            _ => character.is_ascii_digit(),
        }
    }
}

/// A tokenizer for atoms.
pub(crate) enum AtomTokenizer {
    /// The initial state.
    Start,

    /// Keep on consuming input.
    Consume,

    /// We have reached the end of the string.
    ///
    /// Once this state is reached, the tokeniser cannot be used any more.
    End(usize),
}

impl AtomTokenizer {
    /// Valid non-alphabetic characters for the start of atoms.
    pub const VALID_NON_ALPHABETIC: &'static [char] =
        &['+', '-', '*', '/', '_', '!', '?', '=', '<', '>'];

    /// Consumes a single character from the input.
    ///
    /// # Arguments
    /// *  `tokenizer` - The tokenizer.
    fn consume<'a>(self, tokenizer: &mut Tokenizer<'a>) -> Result<Self, PositionedError<Error>> {
        let index = tokenizer.index()?;

        use AtomTokenizer::*;
        match self {
            Start => {
                tokenizer.expect(Self::is_start)?;
                if tokenizer
                    .next()
                    .map(|(_, c)| Self::is_end(c))
                    .unwrap_or(true)
                {
                    Ok(End(index))
                } else {
                    Ok(Consume)
                }
            }
            Consume => match tokenizer.next().map(|(_, c)| c).ok() {
                Some(c) if self.is_valid(c) => Ok(Consume),
                Some(c) if Self::is_end(c) => Ok(End(index)),
                Some(c) => Err(tokenizer.terminate(Error::UnexpectedCharacter { character: c })),
                None => Ok(End(index)),
            },
            End(_) => Err(Error::UnexpectedEnd.for_position(tokenizer.position)),
        }
    }

    /// Determines whether a character is the start of an atom.
    ///
    /// # Arguments
    /// *  `character` - The character to check.
    fn is_start(character: char) -> bool {
        character.is_ascii_alphabetic() || Self::VALID_NON_ALPHABETIC.contains(&character)
    }

    /// Determines whether a character is valid for the current state.
    ///
    /// # Arguments
    /// *  `character` - The character to check.
    fn is_valid(&self, character: char) -> bool {
        Self::is_start(character) || character.is_ascii_digit()
    }

    /// Determines whether a character is the end of an atom.
    ///
    /// # Arguments
    /// *  `character` - The character to check.
    fn is_end(character: char) -> bool {
        character.is_ascii_whitespace()
            || character == Tokenizer::LEFT_PARENTHESIS
            || character == Tokenizer::RIGHT_PARENTHESIS
    }
}
