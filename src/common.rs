/// A marker trait for serialisable types.
#[cfg(feature = "serde")]
pub trait Serializable: ::serde::Serialize + for<'a> ::serde::Deserialize<'a> {}

#[cfg(feature = "serde")]
impl<T> Serializable for T where T: ::serde::Serialize + for<'de> ::serde::Deserialize<'de> {}

/// A marker trait for types that would be serialisable when the `serde` feature is active.
#[cfg(not(feature = "serde"))]
pub trait Serializable {}

/// Writes a list to a formatter.
///
/// # Arguments
/// *  `list` - The list to write.
/// *  `f` - The formatter.
pub(crate) fn write_list<I, T>(list: I, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result
where
    I: Iterator<Item = T>,
    T: ::std::fmt::Display,
{
    use crate::ast::token::tokenizer::Tokenizer;
    write!(f, "{}", Tokenizer::LEFT_PARENTHESIS)?;
    for (i, v) in list.enumerate() {
        if i > 0 {
            write!(f, " ")?;
        }
        write!(f, "{}", v)?;
    }
    write!(f, "{}", Tokenizer::RIGHT_PARENTHESIS)
}
