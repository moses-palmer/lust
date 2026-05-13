//! # Script expressions.
//!
//! This module provides means to convert an abstract syntax tree into an executable form.

use std::convert::Infallible;

use thiserror::Error;

use crate::{Value, alloc, ast, common::write_list, lambda, val};

pub mod cmd;
pub mod env;

/// An error occurring when evaluating an expression.
#[derive(Debug, Error, PartialEq)]
pub enum Error<'a> {
    #[error("syntax error: {message}")]
    Syntax { message: &'static str },

    #[error("evaluation of {node} failed: {message}")]
    Eval {
        node: &'a ast::Node,
        message: &'static str,
    },

    #[error("unknown reference: {value}")]
    UnknownReference { value: String },

    #[error("invalid operation: {0}")]
    InvalidOperation(#[from] val::Error),

    #[error("invalid invocation: expected {expected} arguments, found {actual}")]
    InvalidInvocation { expected: usize, actual: usize },

    #[error("allocation failure")]
    Allocation(#[from] alloc::Error),

    #[error("no more executions permitted")]
    ExecutionLimited,
}

impl From<Infallible> for Error<'_> {
    fn from(_value: Infallible) -> Self {
        unreachable!()
    }
}

/// The result of evaluating an expression.
pub type Result<'a, C> = ::std::result::Result<Value<'a, <C as cmd::Command>::Tag>, Error<'a>>;

/// The context used while parsing an expression.
pub struct ParseContext<C> {
    /// The known variables in this scope.
    pub scope: Vec<String>,

    /// The lambdas.
    pub lambdas: lambda::Store<C>,
}

impl<C> ParseContext<C> {
    /// Attempts to resolve a variable name.
    ///
    /// # Arguments
    /// *  `name` - The name of the variable to resolve.
    pub fn resolve(&self, name: &str) -> Option<Expression<C>> {
        self.scope
            .iter()
            .rev()
            .position(|n| n == name)
            .map(Expression::Reference)
    }
}

impl<C> Default for ParseContext<C> {
    fn default() -> Self {
        Self {
            scope: Default::default(),
            lambdas: Default::default(),
        }
    }
}

/// An expression.
///
/// Once linked, an expression can be evaluated.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Expression<C> {
    /// Nothing
    #[default]
    Void,

    /// A list of expressions.
    List(Vec<Self>),

    /// A map of expressions.
    Map(Vec<String>, Vec<Self>),

    /// A quoted AST node.
    AST(ast::Node),

    /// A variable reference.
    Reference(usize),

    /// A boolean value.
    Boolean(bool),

    /// A number.
    Number(f32),

    /// A string.
    String(String),

    /// A built-in command.
    Command(C),

    /// A reference to a lambda.
    Lambda(lambda::Ref),
}

impl<C> Expression<C>
where
    C: cmd::Command,
{
    /// Parses an AST node into an expression.
    ///
    /// # Arguments
    /// *  `context` - The parsing context.
    /// *  `node` - The AST node to parse.
    pub fn parse<'a>(
        context: &mut ParseContext<C>,
        node: &'a ast::Node,
    ) -> ::std::result::Result<Self, Error<'a>> {
        use ast::NodeValue::*;
        if node.quoted() {
            Ok(Expression::AST(node.clone()))
        } else {
            match node.value() {
                Leaf(ast::Value::Atom { value }) if value == Value::<C::Tag>::TRUE => {
                    Ok(Expression::Boolean(true))
                }
                Leaf(ast::Value::Atom { value }) if value == Value::<C::Tag>::FALSE => {
                    Ok(Expression::Boolean(false))
                }
                Leaf(ast::Value::Atom { value }) => {
                    context
                        .resolve(value)
                        .ok_or_else(|| Error::UnknownReference {
                            value: value.clone(),
                        })
                }
                Leaf(ast::Value::Number { value }) => Ok(Expression::Number(*value)),
                Leaf(ast::Value::String { value }) => Ok(Expression::String(value.clone())),
                Tree(v) if !v.is_empty() => {
                    let (head, tail) = (&v[0], &v[1..]);
                    C::parse(context, head, tail).or_else(|e| match e {
                        Error::UnknownReference { .. } => Ok(Expression::List(
                            v.iter()
                                .map(|n| Expression::parse(context, n))
                                .collect::<::std::result::Result<Vec<_>, _>>()
                                .map_err(|_| e)?,
                        )),
                        _ => Err(e),
                    })
                }
                _ => Err(Error::Eval {
                    node,
                    message: "unexpected node",
                })?,
            }
        }
    }

    /// Calls a function for this and each sub-expression.
    ///
    /// # Argument
    /// *  `f` - The callback.
    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(&Self),
    {
        fn visit<C, F>(e: &Expression<C>, f: &mut F)
        where
            C: cmd::Command,
            F: FnMut(&Expression<C>),
        {
            f(e);
            match e {
                Expression::List(v) => {
                    for e in v {
                        visit(e, f);
                    }
                }
                Expression::Map(_, v) => {
                    for e in v {
                        visit(e, f);
                    }
                }
                Expression::Command(c) => {
                    for e in c.arguments() {
                        visit(e, f);
                    }
                }
                _ => {}
            }
        }

        visit(self, &mut f);
    }

    /// Calls a function for this and each sub-expression.
    ///
    /// # Argument
    /// *  `f` - The callback.
    pub fn for_each_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Self),
    {
        fn visit<C, F>(e: &mut Expression<C>, f: &mut F)
        where
            C: cmd::Command,
            F: FnMut(&mut Expression<C>),
        {
            f(e);
            match e {
                Expression::List(v) => {
                    for e in v {
                        visit(e, f);
                    }
                }
                Expression::Map(_, v) => {
                    for e in v {
                        visit(e, f);
                    }
                }
                Expression::Command(c) => {
                    for e in c.arguments_mut() {
                        visit(e, f);
                    }
                }
                _ => {}
            }
        }

        visit(self, &mut f);
    }

    /// Attempts to convert an AST node to a list of argument names.
    ///
    /// # Arguments
    /// *  `node` - The node to convert.
    pub fn as_argument_list(node: &ast::Node) -> Option<Vec<String>> {
        match node.value() {
            ast::NodeValue::Tree(nodes) => nodes
                .iter()
                .map(|node| match node {
                    ast::Node {
                        value: ast::NodeValue::Leaf(ast::Value::Atom { value }),
                        ..
                    } => Ok(value.clone()),
                    _ => Err(()),
                })
                .collect::<::std::result::Result<Vec<_>, _>>()
                .ok(),
            _ => None,
        }
    }

    /// Attempts to convert an AST node to a map of names to expressions.
    ///
    /// # Arguments
    /// *  `node` - The node to convert.
    pub fn as_map(context: &mut ParseContext<C>, node: &ast::Node) -> Option<Self> {
        match node.value() {
            ast::NodeValue::Tree(nodes) => nodes
                .iter()
                .map(|node| match node {
                    ast::Node {
                        value: ast::NodeValue::Tree(vs),
                        ..
                    } if vs.len() == 2 => {
                        let key = match vs.first() {
                            Some(ast::Node {
                                value: ast::NodeValue::Leaf(ast::Value::Atom { value }),
                                ..
                            }) => Ok(value.clone()),
                            _ => Err(()),
                        }?;
                        let val = Self::parse(context, &vs[1]).map_err(|_| ())?;
                        Ok((key, val))
                    }
                    _ => Err(()),
                })
                .collect::<::std::result::Result<Vec<_>, _>>()
                .map(|l| l.into_iter().unzip::<_, _, Vec<_>, Vec<_>>())
                .map(|(k, v)| Expression::Map(k, v))
                .ok(),
            _ => None,
        }
    }
}

impl<C> ::std::fmt::Display for Expression<C>
where
    C: cmd::Command + ::std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Expression::*;
        match self {
            Void => write!(f, "{}", Value::<C::Tag>::Void),
            List(v) => write_list(v.iter(), f),
            Map(v, _) => write_list(v.iter(), f),
            AST(v) => write!(f, "{v}"),
            Reference(v) => write!(f, "{v}"),
            Boolean(v) => write!(f, "{v}"),
            Number(v) => write!(f, "{v}"),
            String(v) => write!(f, "{v}"),
            Command(v) => write!(f, "{v}"),
            Lambda(v) => write!(f, "<lambda {v}>"),
        }
    }
}
