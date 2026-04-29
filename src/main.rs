#![allow(unused)]
use std::{
    env, error, fs,
    path::{Path, PathBuf},
    process,
};

#[macro_use]
pub mod macros;

mod alloc;
mod ast;
mod common;
mod eval;
mod exp;
mod lambda;
mod script;
mod test_helpers;
mod val;

pub use common::Serializable;

pub use exp::{
    Expression,
    cmd::{Command, Context},
    env::Environment,
};
pub use script::Script;
pub use val::{Value, Values, cons::Cons};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
struct Tag;
impl val::Tag for Tag {}

commands_all! {
    #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
    enum C<
        Context = (),
        Tag = Tag,
    > {
        "debug" => Debug(ctx, expression) {
            let value = ctx.value(expression)?;
            println!("DEBUG: {expression:?} = {value}");
            Ok(value)
        }
    }
}

fn main() {
    match run() {
        Ok(_) => {
            process::exit(0);
        }
        Err(e) => {
            eprint!("Error: {e}");
            process::exit(1);
        }
    }
}

fn run() -> Result<(), String> {
    let args = env::args_os();
    let paths = args
        .into_iter()
        .skip(1)
        .map(PathBuf::try_from)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("invalid path in argument list: {e}"))?;

    if !paths.is_empty() {
        execute(&paths)
    } else {
        repl()
    }
}

fn execute<P>(paths: &[P]) -> Result<(), String>
where
    P: AsRef<Path>,
{
    for path in paths {
        let display_path = path.as_ref().to_string_lossy();
        let data = fs::read_to_string(path)
            .map_err(|e| format!("failed to read path '{display_path}': {e}"))?;

        // Allow shebangs
        let script_string = if data.starts_with("#!") {
            data.find("\n")
                .map(|start| &data[start..])
                .unwrap_or(data.as_str())
        } else {
            data.as_str()
        };

        let ast = match ast::parse(&mut ast::tokenize(&script_string)) {
            Ok(ast) => ast,
            Err(e) => {
                println!("! invalid input: {e}");
                continue;
            }
        };
        let script = match Expression::<C>::parse(&mut Default::default(), &ast) {
            Ok(expression) => expression.link(),
            Err(e) => {
                println!("! failed to parse: {e}");
                continue;
            }
        };
        let alloc = alloc::bounded::Allocator::<128, _>::default();
        let ctx = ();

        match script.evaluate(&alloc, &ctx) {
            Ok(value) => println!("= {}", value),
            Err(error) => println!("! {error}"),
        }
    }

    Ok(())
}

fn repl() -> Result<(), String> {
    let mut rl = rustyline::DefaultEditor::new()
        .map_err(|e| format!("failed to create line reader: {e}"))?;

    while let Ok(line) = rl.readline("> ") {
        let ctx = ();
        match compile(&line).map(Expression::link).and_then(|script| {
            let alloc = alloc::bounded::Allocator::<128, _>::default();
            script.evaluate(&alloc, &ctx).map_err(|e| e.to_string())
        }) {
            Ok(value) => println!("= {}", value),
            Err(error) => println!("! {error}"),
        }
    }

    Ok(())
}

fn compile(s: &str) -> Result<Expression<C>, String> {
    let ast = ast::parse(&mut ast::tokenize(s)).map_err(|e| format!("failed to parse AST: {e}"))?;
    Expression::<C>::parse(&mut Default::default(), &ast)
        .map_err(|e| format!("failed to parse expression: {e}"))
}
