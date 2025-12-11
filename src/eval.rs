use thiserror::Error;

use crate::{ast, exp};

/// An error combining all errors occurring when evaluating.
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("failed to tokenize: {0}")]
    Tokenize(#[from] ast::token::Error),

    #[error("failed to parse AST: {0}")]
    Parse(String),

    #[error("failed to evaluate: {0}")]
    Eval(String),
}

impl From<ast::parser::Error<'_>> for Error {
    fn from(value: ast::parser::Error<'_>) -> Self {
        Self::Eval(value.to_string())
    }
}

impl From<exp::Error<'_>> for Error {
    fn from(value: exp::Error<'_>) -> Self {
        Self::Eval(value.to_string())
    }
}

#[macro_export]
macro_rules! eval {
    ($script:literal => $commands:ty) => {
        $crate::Expression::<$commands>::try_from(
            &$crate::ast::parse(&mut $crate::ast::tokenize($script))
                .map_err($crate::eval::Error::from)?,
        )?
        .link()
        .evaluate(
            &<$commands as $crate::Command>::Context::default(),
            &$crate::Environment::empty(),
        )
        .map_err($crate::eval::Error::from)
        .map($crate::val::owned::Value::from)?
    };
}
