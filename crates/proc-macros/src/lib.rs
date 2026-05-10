use lust_lib::ast::{self, PositionedErrorCause, parser::Error};
use proc_macro::TokenStream;
use syn::{Lit, parse_str};

/// Generates an AST from a script.
///
/// The value generated is an AST literal.
///
/// Using this macro ensures that a string is a valid script representation, but not that it can
/// actually be parsed, since that requires knowing about what [`Command`](lust_lib::Command)
/// implementation is going to be used to run it.
///
/// # Examples
///
/// ```ignore
/// let ast = lust::ast! {r#"
///   (+ 1 2) ; Add 1 and two, this comment is discarded
/// "#};
/// ```
#[proc_macro]
pub fn ast(items: TokenStream) -> TokenStream {
    // Parse the input as a literal string and then unescape it using syn
    let input = items.to_string();
    let script = match parse_str::<Lit>(input.as_str()).and_then(|lit| match lit {
        Lit::Str(v) => Ok(v.value()),
        e => Err(syn::Error::new_spanned(e, "expected string literal")),
    }) {
        Ok(v) => v,
        Err(e) => panic!("failed to parse: {e}"),
    };

    // Determine the offset of reported errors
    let offset = items
        .into_iter()
        .next()
        .map(|t| {
            (
                // The line offset is applied to all positions
                t.span().line() as u16 - 1,
                // The column offset is only applied to the first line, and it includes the string
                // prefix
                t.span().column() as u16 - 1 + input.find("\"").map(|i| i as u16 + 1).unwrap_or(0),
            )
        })
        .unwrap_or((0, 0));
    let reposition = |pos: ast::Position| {
        ast::Position::new(
            pos.row() + offset.0,
            pos.column() + if pos.row() == 1 { offset.1 } else { 0 },
        )
    };

    let root = match ast::parse(&mut ast::tokenize(&script)) {
        Ok(n) => n,
        Err(e) => panic!(
            "failed to parse: {}",
            // Add the offset to synchronise the error message with the source file
            match e {
                Error::TokenizerError { cause } => Error::TokenizerError {
                    cause: cause.cause().for_position(reposition(cause.position())),
                },
                Error::UnexpectedToken { token } => Error::UnexpectedToken {
                    token: token.value().for_position(reposition(token.position())),
                },
                Error::UnexpectedEnd => e,
            }
        ),
    };
    let node = Node(&root);
    quote::quote! {
        #node
    }
    .into()
}

/// A wrapper around an AST node.
struct Node<'a>(&'a ast::Node);

impl<'a> quote::ToTokens for Node<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let position = Position(self.0.position());
        let quoted = self.0.quoted();
        let node_value = NodeValue(self.0.value());
        tokens.extend(quote::quote! {
            ::lust::ast::Node::new(
                #position,
                #quoted,
                #node_value,
            )
        });
    }
}

/// A wrapper around an AST position.
struct Position<'a>(&'a ast::Position);

impl<'a> quote::ToTokens for Position<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let row = self.0.row();
        let column = self.0.column();
        tokens.extend(quote::quote! {
            ::lust::ast::Position::new(
                #row,
                #column,
            )
        });
    }
}

/// A wrapper around a node value.
struct NodeValue<'a>(&'a ast::NodeValue);

impl<'a> quote::ToTokens for NodeValue<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use ast::NodeValue::*;
        tokens.extend(match self.0 {
            Leaf(value) => {
                let value = Value(value);
                quote::quote! {
                    ::lust::ast::NodeValue::leaf(#value)
                }
            }
            Tree(nodes) => {
                let nodes = nodes.iter().map(Node);
                quote::quote! {
                    ::lust::ast::NodeValue::tree(const { &[#(#nodes),*] })
                }
            }
            v => panic!("internal error, {v} unexpected"),
        });
    }
}

/// A wrapper around an AST value.
struct Value<'a>(&'a ast::Value);

impl<'a> quote::ToTokens for Value<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use ast::Value::*;
        tokens.extend(match self.0 {
            Atom { value } => quote::quote! {
                ::lust::ast::Value::atom(#value)
            },
            Number { value } => quote::quote! {
                ::lust::ast::Value::number(#value)
            },
            String { value } => quote::quote! {
                ::lust::ast::Value::string(#value)
            },
            v => panic!("internal error, {v} unexpected"),
        });
    }
}
