use thiserror::Error;

use crate::{ast, exp, val};

/// An error combining all errors occurring when evaluating.
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("failed to tokenize: {0}")]
    Tokenize(#[from] ast::token::Error),

    #[error("invalid value: {0}")]
    Value(#[from] val::Error),

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
        match value {
            exp::Error::InvalidOperation(v) => v.into(),
            _ => Self::Eval(value.to_string()),
        }
    }
}

#[macro_export]
macro_rules! eval {
    ($script:literal => $commands:ty) => {
        {
            let ctx = <$commands as $crate::Command>::Context::default();
            $crate::eval!($script in &ctx => $commands)
        }
    };

    ($script:literal in $ctx:expr => $commands:ty) => {
        {
            use $crate::*;
            let ast = ast::parse(&mut ast::tokenize($script))
                .map_err(eval::Error::from)?;
            let script = Expression::<$commands>::try_from(&ast)?.link();
            let result = script.evaluate($ctx, &Environment::empty())
                .map_err($crate::eval::Error::from)?;
            result
        }
    };
}
