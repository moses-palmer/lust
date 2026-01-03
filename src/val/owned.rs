use crate::ast::{self, token::tokenizer::Tokenizer};
use crate::common::write_list;

/// An owned value.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Value<T> {
    /// Nothing.
    Void,

    /// An AST node.
    AST(ast::Node),

    /// An opaque tagged value.
    ///
    /// Values of this kind can be exposed from the environment as tagged values.
    Tag(T),

    /// A boolean value.
    Boolean(bool),

    /// A number.
    Number(f32),

    /// An atom.
    Atom(String),

    /// A string.
    String(String),

    /// A list.
    List(Vec<Self>),
}

impl<T> ::std::fmt::Display for Value<T>
where
    T: super::Tag,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // This is a copy of [super::Value::fmt]
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
            Boolean(v) => write!(
                f,
                "{}",
                if *v {
                    super::Value::<T>::TRUE
                } else {
                    super::Value::<T>::FALSE
                }
            ),
            Number(v) => write!(f, "{v}"),
            String(v) => write!(f, "{v}"),
            List(v) => {
                write!(f, "{}", Tokenizer::LEFT_PARENTHESIS)?;
                write_list(v.iter(), f)?;
                write!(f, "{}", Tokenizer::RIGHT_PARENTHESIS)
            }
        }
    }
}

impl<'a, T> TryFrom<&'a Value<T>> for super::Value<'a, T>
where
    T: super::Tag,
{
    type Error = super::Error;

    fn try_from(value: &'a Value<T>) -> Result<super::Value<'a, T>, Self::Error> {
        use Value::*;
        match value {
            Void => Ok(super::Value::Void),
            AST(v) => Ok(super::Value::AST(v)),
            Tag(v) => Ok(super::Value::Tag(*v)),
            Boolean(v) => Ok(super::Value::Boolean(*v)),
            Number(v) => Ok(super::Value::Number(*v)),
            Atom(v) => Ok(super::Value::Atom(v)),
            String(v) => Ok(super::Value::String(v)),
            _ => Err(super::Error::Conversion {
                from_type: "owned",
                to_type: "value",
            }),
        }
    }
}

impl<T> TryFrom<super::Value<'_, T>> for Value<T>
where
    T: super::Tag,
{
    type Error = super::Error;

    fn try_from(value: super::Value<'_, T>) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl<T> TryFrom<&'_ super::Value<'_, T>> for Value<T>
where
    T: super::Tag,
{
    type Error = super::Error;

    fn try_from(value: &super::Value<'_, T>) -> Result<Self, Self::Error> {
        use super::Value::*;
        Ok(match value {
            Void => Self::Void,
            AST(v) => Self::AST(Clone::clone(v)),
            Tag(v) => Self::Tag(*v),
            Boolean(v) => Self::Boolean(*v),
            Number(v) => Self::Number(*v),
            Atom(v) => Self::Atom(v.to_string()),
            String(v) => Self::String(v.to_string()),
            Lambda(_) => return Err(super::Error::Operation("cannot serialize lambda")),
            List(v) => v
                .iter()
                .map(Self::try_from)
                .collect::<Result<Vec<_>, _>>()
                .map(Self::List)?,
        })
    }
}

impl<T> From<Vec<Value<T>>> for Value<T> {
    fn from(value: Vec<Value<T>>) -> Self {
        Self::List(value)
    }
}

impl<T> TryFrom<Vec<super::Value<'_, T>>> for Value<T>
where
    T: super::Tag,
{
    type Error = super::Error;

    fn try_from(value: Vec<super::Value<T>>) -> Result<Self, Self::Error> {
        Ok(Self::List(
            value
                .into_iter()
                .map(Self::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}
