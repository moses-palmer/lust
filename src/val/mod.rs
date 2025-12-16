use thiserror::Error;

use crate::{
    ast::{self, token::tokenizer::Tokenizer},
    lambda,
};

pub mod owned;

/// A value operation failed.
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("operation failed: {0}")]
    Operation(&'static str),

    #[error("AST node is not a value: {0}")]
    Type(ast::Node),

    #[error("cannot convert {from_type} to {to_type}")]
    Conversion {
        /// The value being converted.
        from_type: &'static str,

        /// The target type.
        to_type: &'static str,
    },
}

/// An opaque tagged value.
pub trait Tag: Copy + PartialEq + ::std::fmt::Debug + crate::Serializable {}

/// A value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value<'a, T> {
    /// Nothing.
    Void,

    /// An AST node.
    AST(&'a ast::Node),

    /// An opaque tagged value.
    ///
    /// Values of this kind can be exposed from the environment as tagged values.
    Tag(T),

    /// A boolean value.
    Boolean(bool),

    /// A number.
    Number(f32),

    /// An atom.
    Atom(&'a str),

    /// A string.
    String(&'a str),

    /// A reference to a lambda.
    Lambda(lambda::Ref),
}

impl<T> Value<'_, T> {
    /// The string representation of `true`.
    pub const TRUE: &'static str = "true";

    /// The string representation of `false`.
    pub const FALSE: &'static str = "false";

    /// The nil value.
    pub const NIL: Self = Value::Void;

    /// The printable type name of the value.
    pub fn type_name(&self) -> &'static str {
        use Value::*;
        match self {
            Void => "void",
            AST(_) => "ast",
            Tag(_) => "tag",
            Boolean(_) => "bool",
            Number(_) => "number",
            Atom(_) => "atom",
            String(_) => "string",
            Lambda(_) => "lambda",
        }
    }
}

impl<T> ::std::fmt::Display for Value<'_, T>
where
    T: Copy + ::std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Value::*;
        match self {
            Void => write!(
                f,
                "{}{}",
                Tokenizer::LEFT_PARENTHESIS,
                Tokenizer::RIGHT_PARENTHESIS
            ),
            Tag(v) => write!(f, "{v:?}"),
            AST(v) => write!(f, "{v}"),
            Atom(v) => write!(f, "{v}"),
            Boolean(v) => write!(f, "{}", if *v { Self::TRUE } else { Self::FALSE }),
            Number(v) => write!(f, "{v}"),
            String(v) => write!(f, "{v}"),
            Lambda(v) => write!(f, "<lambda {v}>"),
        }
    }
}

impl<T> From<()> for Value<'_, T> {
    fn from(_: ()) -> Self {
        Value::Void
    }
}

impl<'a, I, T> From<Option<I>> for Value<'a, T>
where
    I: Into<Value<'a, T>>,
{
    fn from(value: Option<I>) -> Self {
        if let Some(value) = value {
            value.into()
        } else {
            Value::Void
        }
    }
}

impl<T> From<lambda::Ref> for Value<'_, T> {
    fn from(value: lambda::Ref) -> Self {
        Value::Lambda(value)
    }
}

/// Generates a conversion from a type to a value enum value.
macro_rules! wrap {
    ($kind:ident { $($v:ident: $type:ty => $($unowned:expr)? $(=> $owned:expr)?,)* }) => {
        $(
            $(
                impl<'a, T> From<$type> for Value<'a, T>
                where
                    T: Copy + ::std::fmt::Debug,
                {
                    fn from(value: $type) -> Self {
                        let $v = value;
                        Value::$kind($unowned)
                    }
                }
            )?

            $(
                impl<T> From<$type> for owned::Value<T>
                where
                    T: Copy + ::std::fmt::Debug,
                {
                    fn from(value: $type) -> Self {
                        let $v = value;
                        owned::Value::$kind($owned)
                    }
                }
            )?
        )*
    };
}

/// Generates a conversion from a value enum value to a type.
macro_rules! unwrap {
    ($type:ty { $($kind:ident($v:ident) => $expr:expr,)* }) => {
        impl<'a, T> TryFrom<Value<'a, T>> for $type
        where
            T: Copy + ::std::fmt::Debug,
        {
            type Error = Error;

            fn try_from(value: Value<'a, T>) -> Result<Self, Self::Error> {
                match value {
                    $(
                        Value::$kind($v) => Ok($expr),
                    )*

                    _ => Err(Error::Conversion {
                        from_type: value.type_name(),
                        to_type: ::std::any::type_name::<Self>(),
                    }),
                }
            }
        }
    };
}

wrap!(Boolean {
    v: bool => v => v,
});
wrap!(Number {
    v: i8 => v as f32 => v as f32,
    v: u8 => v as f32 => v as f32,
    v: i32 => v as f32 => v as f32,
    v: u32 => v as f32 => v as f32,
    v: i64 => v as f32 => v as f32,
    v: u64 => v as f32 => v as f32,
    v: f32 => v => v,
});
wrap!(String {
    v: &'a str => v,
    v: String => => v,
});

unwrap!(bool {
    Boolean(v) => v,
    Number(v) => v != 0.0,
    String(v) => !v.is_empty(),
});
unwrap!(i8 {
    Number(v) => (v as i128).try_into().map_err(|_| Error::Conversion {
        from_type: "number",
        to_type: "i8",
    })?,
});
unwrap!(u8 {
    Number(v) => (v as i128).try_into().map_err(|_| Error::Conversion {
        from_type: "number",
        to_type: "u8",
    })?,
});
unwrap!(i16 {
    Number(v) => (v as i128).try_into().map_err(|_| Error::Conversion {
        from_type: "number",
        to_type: "i16",
    })?,
});
unwrap!(u16 {
    Number(v) => (v as i128).try_into().map_err(|_| Error::Conversion {
        from_type: "number",
        to_type: "u16",
    })?,
});
unwrap!(i32 {
    Number(v) => (v as i128).try_into().map_err(|_| Error::Conversion {
        from_type: "number",
        to_type: "i32",
    })?,
});
unwrap!(u32 {
    Number(v) => (v as i128).try_into().map_err(|_| Error::Conversion {
        from_type: "number",
        to_type: "u32",
    })?,
});
unwrap!(i64 {
    Number(v) => (v as i128).try_into().map_err(|_| Error::Conversion {
        from_type: "number",
        to_type: "i64",
    })?,
});
unwrap!(u64 {
    Number(v) => (v as i128).try_into().map_err(|_| Error::Conversion {
        from_type: "number",
        to_type: "u64",
    })?,
});
unwrap!(f32 {
    Number(v) => v,
});
unwrap!(&'a str {
    String(v) => v,
});
unwrap!(String {
    String(v) => v.to_string(),
});
unwrap!(&'a ast::Node {
    AST(v) => v,
});
unwrap!(lambda::Ref {
    Lambda(v) => v,
});

/// Generates an implementation for an operation for [`Value`].
macro_rules! op {
    ($op:path => $meth:ident { $(
        $kind:ident($self:ident, $other:ident) => $expr:expr,
    )* }) => {
        impl<T> $op for Value<'_, T>
        where
            T: Tag,
        {
            type Output = Result<Self, Error>;

            fn $meth(self, other: Self) -> Self::Output {
                use Value::*;
                match (self, other) {
                    $(
                        ($kind($self), $kind($other)) => $expr,
                    )*
                    _ => Err(Error::Conversion {
                        from_type: self.type_name(),
                        to_type: other.type_name(),
                    }),
                }
            }
        }
    };
}

op!(::std::ops::Add => add {
    Number(a, b) => Ok((a + b).into()),
});
op!(::std::ops::Sub => sub {
    Number(a, b) => Ok((a - b).into()),
});
op!(::std::ops::Mul => mul {
    Number(a, b) => Ok((a * b).into()),
});
op!(::std::ops::Div => div {
    Number(a, b) => if b != 0.0 {
        Ok((a / b).into())
    } else {
        Err(Error::Operation("division by zero"))
    },
});

/// A transient collection of values.
pub type Values<'a, T> = ::smallvec::SmallVec<[Value<'a, T>; 8]>;

#[cfg(test)]
mod tests {
    use super::Error;

    use crate::test_helpers::*;

    #[test]
    fn type_name() {
        // Arrange, act and assert
        assert_eq!(Value::Tag(Tag::A).type_name(), "tag");
        assert_eq!(Value::AST(&parse("(1 2 3)")).type_name(), "ast");
        assert_eq!(Value::Atom("a".into()).type_name(), "atom");
        assert_eq!(Value::Boolean(true).type_name(), "bool");
        assert_eq!(Value::Number(1.0).type_name(), "number");
        assert_eq!(Value::String("a".into()).type_name(), "string");
    }

    #[test]
    fn to_string_atom() {
        // Arrange
        let tested = Value::Atom("some-atom".into());
        let expected = "some-atom".to_string();

        // Act
        let actual = tested.to_string();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn to_string_true() {
        // Arrange
        let tested = Value::Boolean(true);
        let expected = "true".to_string();

        // Act
        let actual = tested.to_string();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn to_string_false() {
        // Arrange
        let tested = Value::Boolean(false);
        let expected = "false".to_string();

        // Act
        let actual = tested.to_string();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn to_string_number() {
        // Arrange
        let tested = Value::Number(123.0);
        let expected = "123".to_string();

        // Act
        let actual = tested.to_string();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn to_string_string() {
        // Arrange
        let tested = Value::String("hello, \"world\"".into());
        let expected = "hello, \"world\"".to_string();

        // Act
        let actual = tested.to_string();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn try_into() {
        // Arrange, act and assert
        assert_eq!(bool::try_from(Value::Boolean(true)), Ok(true));
        assert_eq!(bool::try_from(Value::Boolean(false)), Ok(false));
        assert_eq!(bool::try_from(Value::Number(0.0)), Ok(false));
        assert_eq!(bool::try_from(Value::Number(1.0)), Ok(true));
        assert_eq!(bool::try_from(Value::String("".into())), Ok(false));
        assert_eq!(bool::try_from(Value::String("a".into())), Ok(true));
        assert_eq!(i8::try_from(Value::Number(123.0)), Ok(123i8));
        assert_eq!(u8::try_from(Value::Number(123.0)), Ok(123u8));
        assert_eq!(i16::try_from(Value::Number(123.0)), Ok(123i16));
        assert_eq!(u16::try_from(Value::Number(123.0)), Ok(123u16));
        assert_eq!(i32::try_from(Value::Number(123.0)), Ok(123i32));
        assert_eq!(u32::try_from(Value::Number(123.0)), Ok(123u32));
        assert_eq!(i64::try_from(Value::Number(123.0)), Ok(123i64));
        assert_eq!(u64::try_from(Value::Number(123.0)), Ok(123u64));
        assert_eq!(f32::try_from(Value::Number(123.0)), Ok(123.0f32));
        assert_eq!(<&str>::try_from(Value::String("a".into())), Ok("a"));
        assert_eq!(
            String::try_from(Value::String("a".into())),
            Ok("a".to_string())
        );

        assert_eq!(
            bool::try_from(Value::Atom("".into())),
            Err(Error::Conversion {
                from_type: "atom",
                to_type: "bool"
            })
        );
        assert_eq!(
            i8::try_from(Value::Number(1234.0)),
            Err(Error::Conversion {
                from_type: "number",
                to_type: "i8",
            }),
        );
        assert_eq!(
            u8::try_from(Value::Number(-1.0)),
            Err(Error::Conversion {
                from_type: "number",
                to_type: "u8",
            }),
        );
        assert_eq!(
            i16::try_from(Value::Number(123456.0)),
            Err(Error::Conversion {
                from_type: "number",
                to_type: "i16",
            }),
        );
        assert_eq!(
            u16::try_from(Value::Number(-1.0)),
            Err(Error::Conversion {
                from_type: "number",
                to_type: "u16",
            }),
        );
        assert_eq!(
            i32::try_from(Value::Number(12345678901.0)),
            Err(Error::Conversion {
                from_type: "number",
                to_type: "i32",
            }),
        );
        assert_eq!(
            u32::try_from(Value::Number(-1.0)),
            Err(Error::Conversion {
                from_type: "number",
                to_type: "u32",
            }),
        );
    }

    #[test]
    fn from() {
        // Arrange, act and assert
        assert_eq!(Value::from(Option::<bool>::None), Value::Void);
        assert_eq!(Value::from(()), Value::Void);
        assert_eq!(Value::from(true), Value::Boolean(true));
        assert_eq!(Value::from(false), Value::Boolean(false));
        assert_eq!(Value::from(123i8), Value::Number(123.0));
        assert_eq!(Value::from(123u8), Value::Number(123.0));
        assert_eq!(Value::from(123i32), Value::Number(123.0));
        assert_eq!(Value::from(123u32), Value::Number(123.0));
        assert_eq!(Value::from(123i64), Value::Number(123.0));
        assert_eq!(Value::from(123u64), Value::Number(123.0));
        assert_eq!(Value::from(123.0f32), Value::Number(123.0));
        assert_eq!(Value::from("a"), Value::String("a".into()));
    }

    #[test]
    fn add_number() {
        // Arrange
        let a = Value::Number(12.0);
        let b = Value::Number(34.0);
        let expected = Ok(Value::Number(46.0));

        // Act
        let actual = a + b;

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn add_other() {
        // Arrange
        let a = Value::String("hello ".into());
        let b = Value::Number(34.0);
        let expected = Err(Error::Conversion {
            from_type: "string",
            to_type: "number",
        });

        // Act
        let actual = a + b;

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn sub_number() {
        // Arrange
        let a = Value::Number(46.0);
        let b = Value::Number(34.0);
        let expected = Ok(Value::Number(12.0));

        // Act
        let actual = a - b;

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn sub_other() {
        // Arrange
        let a = Value::String("hello ".into());
        let b = Value::Number(34.0);
        let expected = Err(Error::Conversion {
            from_type: "string",
            to_type: "number",
        });

        // Act
        let actual = a - b;

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn mul_number() {
        // Arrange
        let a = Value::Number(3.0);
        let b = Value::Number(4.0);
        let expected = Ok(Value::Number(12.0));

        // Act
        let actual = a * b;

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn mul_other() {
        // Arrange
        let a = Value::String("hello ".into());
        let b = Value::Number(34.0);
        let expected = Err(Error::Conversion {
            from_type: "string",
            to_type: "number",
        });

        // Act
        let actual = a * b;

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn div_number() {
        // Arrange
        let a = Value::Number(12.0);
        let b = Value::Number(3.0);
        let expected = Ok(Value::Number(4.0));

        // Act
        let actual = a / b;

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn div_zero() {
        // Arrange
        let a = Value::Number(12.0);
        let b = Value::Number(0.0);
        let expected = Err(Error::Operation("division by zero"));

        // Act
        let actual = a / b;

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn div_other() {
        // Arrange
        let a = Value::String("hello ".into());
        let b = Value::Number(34.0);
        let expected = Err(Error::Conversion {
            from_type: "string",
            to_type: "number",
        });

        // Act
        let actual = a / b;

        // Assert
        assert_eq!(actual, expected);
    }
}
