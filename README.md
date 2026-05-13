# LUST - LISP in Rust

This is a library that enables sciptability of applications using a simplified LISP.


## Quickstart

Use the following code to define a standard LUST interpreter:

```rust
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
# #[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
pub struct Tag;
lust::tag!(Tag);

lust::commands_all! {
#   #[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
    pub enum Commands<
        Context = (),
        Tag = Tag,
    > {
        // Define custom commands here
    }
}
```

This will define an `enum Commands` with a variant for each LUST command. A runnable instance can
be created using the following code:

```rust
# #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
# #[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
# struct Tag; lust::tag!(Tag); lust::commands_all! {
# #[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
# enum Commands<Context = (), Tag = Tag> {} }
# fn main() -> Result<(), String> {
let script: lust::Script<Commands> = r#"
    (let
        (
            (a 5)
            (b (* 0.1 0.3)))
        (+ a b))
"#.parse()?;
assert_eq!(
    script.evaluate(
        &lust::alloc::zero::Allocator::default(),
        &(),
    ),
    Ok(5.03.into()),
);
# Ok(()) }
```

### Custom commands

To add custom commands, add them to the macro:

```rust
# #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
# #[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
# struct Tag; lust::tag!(Tag);
# fn main() -> Result<(), lust::eval::Error> {
lust::commands_all! {
#   #[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
    enum Commands<
        Context = (),
        Tag = Tag,
    > {
        /// "debug" is the name of the command in scripts, and Debug is its enum variant name.
        ///
        /// This comment will be added as documentation for the variant.
        "debug" => Debug(
            // The first argument is always an execution context.
            ctx,

            // Following is a list of required arguments, optionally typed; if no type is
            // specified, an expression is passed, otherwise the expression is evaluated, and
            // its result is converted to the required type if possible.
            v,
        ) {
            // v is an expression, so we use the context to evaluate it
            let value = ctx.value(v)?;
            dbg!(value);
            Ok(value)
        }
    }
}

assert_eq!(
    lust::eval!(r#"
        (debug (+ 40 2))
    "# => Commands),
    42.into(),
);
# Ok(()) }
```

For more information, see [_Defining commands_](#defining-commands).


### Passing data to a script

To pass data to a script, as opposed to have the script pull data using a command, use a
[`Function`](crate::script::Function). Using it requires that the script consists of a single
top-level `lambda`command:

```rust
# #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
# #[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
# struct Tag; lust::tag!(Tag); lust::commands_all! {
# #[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
# enum Commands<Context = (), Tag = Tag> {} }
# fn main() -> Result<(), String> {
let script: lust::Script<Commands> = r#"
    (lambda (a b c) (do
        (+ a (* b c))
    ))
"#.parse()?;
let function: lust::script::Function<_> = script.try_into().expect("top-level lambda");

assert_eq!(
    function.invoke(
        &lust::alloc::zero::Allocator::default(),
        &(),
        &[1.into(), 2.into(), 3.into()],
    ),
    Ok(7.into()),
);
# Ok(()) }
```


## Script syntax

A script is a sequence of expressions in S-expression form:

*  A **list** is zero or more expressions enclosed in parentheses, for example `(a b c)` or `()`.
   The first element is the command to invoke and the rest are its arguments.
*  A **number** is an optional leading `-` followed by ASCII digits, optionally followed by a `.`
   and more digits, for example `42`, `-1.5`, `.5` or `-.5`. There is no exponent syntax. All
   numbers are floating point.
*  A **string** is enclosed in double quotes, for example `"hello"`. A backslash escapes the next
   character, so `"\"quoted\""` and `"line\nbreak"` are valid.
*  An **atom** is any other contiguous run of non-whitespace characters that is not a parenthesis,
   a quote or a comment, for example `foo`, `+`, `null?` or `<=`. Atoms are typically command
   names or argument identifiers.
*  A **quoted form** is any expression prefixed with `'`, for example `'foo` or `'(1 2 3)`. A
   quoted expression is passed to the receiving command as an [`ASTNode`](crate::ast::Node)
   instead of being evaluated.
*  A **comment** starts with `;` and runs to the end of the line.

Whitespace separates tokens but is otherwise insignificant.


## The _Commands_ `enum`

This `enum` controls what commands are available. It can be constructed using the
[`commands!`](crate::commands!) or the [`commands_all!`](crate::commands_all!) macros. The former
constructs an empty enum by default, whereas the latter includes all predefined commands.

It is possible to include only a selection of predefined commands:

```rust
# #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
# #[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
# struct Tag; lust::tag!(Tag);
lust::commands! {
#   #[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
    enum Commands<
        Context = (),
        Tag = Tag,
    > impl (arithmetic, boolean, lambda) { }
}
```


## Defining commands

A command is declared inside the [`commands!`](crate::commands!) or
[`commands_all!`](crate::commands_all!) macro with this shape:

```text
"name" => Variant(
    ctx,
    arg1,
    arg2: f32,
    ...rest,
) {
    // body returns a lust::exp::Result
}
```

*  `"name"` is the atom used to invoke the command in scripts.
*  `Variant` is the name of the generated enum variant.
*  `ctx` is the [`EvaluationContext`](crate::script::EvaluationContext) passed in at runtime.

Each named argument after `ctx` is a positional script argument. The parser rejects invocations
with the wrong number of arguments before the body ever runs.


### Untyped arguments

If the argument is untyped, the body receives the unevaluated
[`&Expression`](crate::Expression). Use `ctx.value(expr)?` to evaluate it. This is what you want
for arguments that should be evaluated lazily or conditionally, or selected between, as `if` does
between its two branches.


### Typed arguments

If the argument is typed with the type `T`, the parser evaluates the argument eagerly and converts
the result with `TryFrom` to the requested type. The body receives a value of type `T` directly. The
conversion error is bubbled up as a script error.

The following type aliases are in scope inside the body and are intended for use as argument types:

*  `ASTNode<'a>`: `&'a `[`ast::Node`](crate::ast::Node). Receives [`Value::AST`](crate::Value::AST),
   produced by quoted forms.
*  `Cons<'a>`: [`Cons<'a, Value<'a>>`](crate::Cons). Use as `&Cons<'a>` to receive a list.
*  `Lambda`: [`lambda::Ref`](crate::lambda::Ref). A lambda that can be invoked using
   `ctx.invoke(lambda, args)`.
*  `Tag`: the configured tag type.
*  `Value<'a>`: [`Value<'a, Tag>`](crate::Value). This is a catch-all that accepts every value kind.

Plain Rust types with a `TryFrom<Value>` implementation work too. The library provides
implementations for `bool`, `f32`, every integer type from `i8` through `u64`, `&str` and `String`.
Please see [_Values_](#values) for information about conversions.


### AST transforms on required arguments

```text
name
    => |ctx, node| { /* parse-time setup: returns Expression */ }
    => |ctx, args| { /* parse-time teardown after later args are parsed */ },
```

A required argument may carry a parse-time transform that turns the argument's raw AST node into
an [`Expression`](crate::Expression). The syntax is `name => |ctx, node| expr`, where `ctx` is the
[`ParseContext`](crate::exp::ParseContext) (allowing manipulation of the active variable scope)
and `node` is the [`&ast::Node`](crate::ast::Node) being parsed.

The transform is the way to introduce parser-time effects, such as binding new names that the
arguments parsed *after* it should be able to reference. The standard `let` and `lambda` commands
use this to push their bound names onto `ctx.scope` so the body can resolve them.

A second `=> |ctx, args| expr` block may follow the transform to *finalize* the parsed arguments
once all required positional arguments have been parsed. It receives `&[Expression]` and is
typically used to undo the parser-side effects, for example popping the names that the first
transform pushed. Its return value is discarded, but parsing terminates if it fails.


### Trailing varargs and the vararg transform

A definition may end with a single trailing varargs entry, written `...name`:

```text
name1, name2, ...rest
```

At runtime, `rest` is an iterator of unevaluated [`Expression`](crate::Expression)s holding every
remaining argument. Without a varargs entry, the parser rejects extra arguments.

The varargs entry also accepts a parse-time transform with three parameters:

```text
...rest => |ctx, head, args| { /* returns Expression */ }
```

*  `ctx` is the [`ParseContext`](crate::exp::ParseContext).
*  `head` is the [`&ast::Node`](crate::ast::Node) for the command name.
*  `args` is the full `Vec<Expression>` of every parsed argument (required ones plus the varargs).

Unlike the per-argument transforms above, this transform *replaces* the
[`Expression`](crate::Expression) the macro would otherwise produce: the value it returns becomes
the result of parsing the whole invocation. That makes it useful for command forms whose surface
syntax does not map one-to-one onto a runtime command. For example, the standard `if` uses it to
synthesize a `void` else-branch when the script omits it, and `lambda` uses it to register the
body in the lambda store and emit an [`Expression::Lambda`](crate::Expression::Lambda).

The varargs name is itself optional: writing `...` with no name followed by a transform applies the
transform but still rejects extra arguments at parse time.


### Examples

```rust
# #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
# #[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
# struct Tag; lust::tag!(Tag);
# fn main() -> Result<(), lust::eval::Error> {
# lust::commands_all! {
#   #[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
#   enum Commands<Context = (), Tag = Tag> {
// ...

// Untyped: `e` is an expression; evaluation is up to the body
"identity" => Identity(ctx, e) {
    ctx.value(e)
}

// Typed: `n` is already an f32; the script must produce a number
"double" => Double(ctx, n: f32) {
    Ok((n * 2.0).into())
}

// Mapped: we produce a lambda that doubles its value
"double_lambda" => DoubleLambda(
    _ctx,
    args
        => |ctx, n| {
            ctx.scope.extend(Expression::<Self>::as_argument_list(n)
                .ok_or_else(|| Error::Syntax {
                    message: "expected argument list",
                })?);
            Ok(Expression::AST(n.clone()))
        }
        => |ctx, a| {
            if let Some(Expression::AST(n)) = a.get(0) {
                ctx.scope.truncate(ctx.scope.len() - n.len());
            }
        },
    body,
    ... => |ctx, _node, a| {
        let argument_count = lust::extract!(&a[0], Expression::AST(v) => v.len())
            .unwrap();
        let body = Expression::Command(Commands::Double(vec![a.pop().unwrap()]));

        Ok(Expression::Lambda(ctx.lambdas.register(
            lambda::Lambda::new(argument_count, body),
        )))
    })
// ...
#   }
# }

assert_eq!(
    lust::eval!(r#"(identity 7)"# => Commands),
    7.into(),
);
assert_eq!(
    lust::eval!(r#"(double 21)"# => Commands),
    42.into(),
);
assert_eq!(
    lust::eval!(r#"
        (let
            ((d (double_lambda (a b) (+ a b))))
            (d 2 5))"# => Commands),
    14.into(),
);
# Ok(()) }
```

## The `Tag` type

A tag is an opaque, application-defined value that can be passed around inside a script as a regular
value. Scripts cannot construct or inspect tags themselves; they can only receive them, pass them
around, and hand them back to native commands. This makes tags useful for exposing handles to host
data  such as entity IDs, resources or file handles without exposing their representation to
scripts.

A tag type must be `Copy`, `PartialEq`, `PartialOrd`, `Debug` and
[`Serializable`](crate::Serializable).

The `Serializable` trait depends on the crate features:
*  _if `serde` is enabled_, then the trait is implemented for every type that implements both
   [`::serde::Serialize`] and [`::serde::Deserialize`].
*  _if `serde` is not enabled_, then the trait is empty and is implemented for every type with no
   constraints.

The [`tag!`](crate::tag) macro implements the [`Tag`](crate::val::Tag) trait for the type and adds
the conversions to and from [`Value`](crate::Value):

```rust
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
# #[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
struct EntityId(u32);
lust::tag!(EntityId);
```

The tag type is then plugged into the `Commands` enum via `Tag = ...`, and shows up as the
[`Value::Tag`](crate::Value::Tag) variant inside command implementations:

```rust
# #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
# #[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
# struct EntityId(u32); lust::tag!(EntityId);
# fn main() -> Result<(), lust::eval::Error> {
lust::commands_all! {
#   #[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
    enum Commands<
        Context = (),
        Tag = EntityId,
    > {
        "player" => Player(ctx) {
            // Hand a tag value back to the script
            Ok(EntityId(42).into())
        }
    }
}

assert_eq!(
    lust::eval!(r#"(player)"# => Commands),
    EntityId(42).into(),
);
# Ok(()) }
```


## The `Context` type

The context is an application-defined value passed to every native command during evaluation. It is
how commands read from or write to host state such as a world, a logger or a configuration. Any type
implementing the [`Context`](crate::Context) trait can be used; the unit type `()` is the simplest
context.

The trait has a single method, `on_evaluate`, which is invoked before every subexpression is
evaluated. Returning an error from it aborts the script; this is how policies such as cancellation
or resource limits are implemented.

```rust
# fn main() -> Result<(), lust::eval::Error> {
# #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
# #[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
# struct Tag; lust::tag!(Tag);
struct Logger;
impl lust::Context for Logger {}
impl Logger {
    pub fn log(&self, expression: &lust::Expression<Commands>) {
        println!("Evaluating expression: {expression:?}");
    }
}

lust::commands_all! {
    #[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
    enum Commands<
        Context = Logger,
        Tag = Tag,
    > {
        "log" => Log(ctx, message) {
            ctx.ctx.log(message);
            let value = ctx.value(message)?;
            Ok(value)
        }
    }
}
# Ok(()) }
```


### Constraining resources

[`ResourceConstrainer`](crate::exp::cmd::ResourceConstrainer) is a built-in
[`Context`](crate::Context) implementation that bounds how much work a script may perform. It is
constructed from an `isize` budget; each evaluation step decrements the counter, and once it
reaches zero the script aborts with [`Error::ExecutionLimited`](crate::exp::Error). Use it when
running untrusted scripts.

```rust
use lust::exp::cmd::ResourceConstrainer;

# #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
# #[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
# struct Tag; lust::tag!(Tag); lust::commands_all! {
# #[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
# enum Commands<Context = ResourceConstrainer, Tag = Tag> {} }
# fn main() -> Result<(), String> {
let script: lust::Script<Commands> = "(+ 1 2 3)".parse()?;
let alloc = lust::alloc::zero::Allocator::default();

// A budget of 5 is exactly enough: one for the root, one for `+`, one per argument.
assert_eq!(script.evaluate(&alloc, &ResourceConstrainer::from(5)), Ok(6.0.into()));

// A budget of 4 is too small and the script aborts.
assert!(script.evaluate(&alloc, &ResourceConstrainer::from(4)).is_err());
# Ok(()) }
```


### Composing contexts

Tuples of up to eight contexts also implement [`Context`](crate::Context); each element's
`on_evaluate` is called in order. This lets separate concerns (logging, metrics or resource limits)
be combined without defining a dedicated context type:

```rust
# use lust::exp::cmd::ResourceConstrainer;
struct Logger;
impl lust::Context for Logger {
    fn on_evaluate<'a>(&self) -> ::std::result::Result<(), lust::exp::Error<'a>> {
        println!("Evaluating a subexpression!");
        Ok(())
    }
}

// (Logger, ResourceConstrainer) is itself a Context.
let _ctx: (Logger, ResourceConstrainer) = (Logger, 1024.into());
```


## Allocators

Some list operations, such as `cons` or `list`, need somewhere to put the new [`Cons`](crate::Cons)
cells. That storage is provided by an [`Allocator`](crate::Allocator) supplied by the caller.

The library ships two implementations:

*  [`alloc::zero::Allocator`](crate::alloc::zero::Allocator) cannot allocate at all. Every
   `alloc` call returns an [`alloc::Error`](crate::alloc::Error). It is appropriate for scripts
   that only do arithmetic, comparisons or lambdas with no list construction.
*  [`alloc::bounded::Allocator<N, T>`](crate::alloc::bounded::Allocator) is an arena with capacity
   for `N` items. Its storage is held inline in the allocator itself, so no heap allocation is
   performed. The arena owns the values until it is dropped, so all references handed out by
   `alloc` are valid for the lifetime of the allocator. Once `N` items have been allocated, every
   further `alloc` returns an [`alloc::Error`](crate::alloc::Error). It cannot be reused.

Pick `bounded` when scripts may build lists, and size `N` so it bounds the runtime allocation
budget. The [`eval!`](crate::eval!) macro uses `bounded::Allocator::<128, _>` internally.

```rust
# #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
# #[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
# struct Tag; lust::tag!(Tag); lust::commands_all! {
# #[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
# enum Commands<Context = (), Tag = Tag> {} }
# fn main() -> Result<(), String> {
let script: lust::Script<Commands> = "(cons 1 (list 2))".parse()?;

// Two cons cells are needed; a capacity of 2 succeeds
let alloc = lust::alloc::bounded::Allocator::<2, _>::default();
assert!(script.evaluate(&alloc, &()).is_ok());

// A capacity of 1 is too small; allocation fails
let alloc = lust::alloc::bounded::Allocator::<1, _>::default();
assert!(script.evaluate(&alloc, &()).is_err());

// A zero capacity allocator also fails
let alloc = lust::alloc::zero::Allocator::default();
assert!(script.evaluate(&alloc, &()).is_err());
# Ok(()) }
```


## Values

A value in a LUST script has a type. The runtime representation is `Value<'a, Tag>`, where each
variant carries the script data as shown below:

*  _Void_ (`Value::Void`): represents the empty list / nil. Printed as `()` and exposed via
   `Value::NIL`. Many optional/absent values convert to `Void`, for example `Option::None` and `()`.
*  _AST_ (`Value::AST(&ast::Node)`): an abstract syntax tree node produced by quoted forms
   (e.g. `'foo` or `'(1 2 3)`). Quoted expressions are passed to commands as `ASTNode<'a>` so the
   command can inspect or transform the raw `ast::Node`.
*  _Tag_ (`Value::Tag(Tag)`): an opaque, application-defined handle supplied by the host. Scripts
   cannot construct or inspect tags themselves — they may only receive and pass them back to
   native commands. The tag type must implement `Copy`, `PartialEq`, `PartialOrd`, `Debug` and,
   if the `serde` feature is active, `Serializable`. The `tag!` macro implements the required glue
   and `From/Into` conversions.
*  _Boolean_ (`Value::Boolean(bool)`): the two truth values `true` and `false`. The display uses
   the constants `Value::TRUE` and `Value::FALSE`. `bool::try_from(Value)` accepts booleans
   directly and also converts numbers (`0.0` → `false`, non-zero → `true`) and strings
   (empty → `false`, non-empty → `true`).
*  _Number_ (`Value::Number(f32)`): all numeric literals are parsed as 32-bit floating point values
   (`f32`). The library provides `From` conversions from Rust integer types and `f32` into
   `Value::Number`, and `TryFrom<Value>` implementations to convert a script number into integer
   types with proper range checking. Arithmetic (`+`, `-`, `*`, `/`) is defined for `Number`;
   division by zero produces an operation error.
*  _Atom_ (`Value::Atom(&'a str)`): a symbol or token. This kind is normally not exposed to
   commands, but rather wrapped in a `Value::AST(_)` instance.
*  _String_ (`Value::String(&'a str)`): double-quoted string literals.
*  _Lambda_ (`Value::Lambda(lambda::Ref)`): a reference to a invokable lambda value. The `Display`
   implementation renders lambdas as `<lambda {id}>`.
*  _List_ (`Value::List(&Cons<'a, Value<'a, Tag>>`) ): a linked cons cell list. Lists are printed
   in S-expression form  with parentheses around their elements. Construction of lists requires an
   `Allocator` (see the Allocators section) because cons cells must be stored somewhere the
   runtime can borrow from.

`PartialOrd` is implemented for a subset of variants (tags, booleans, numbers and strings). The
transient collection type `Values<'a, Tag>` is a smallvec alias used for passing multiple runtime
values efficiently.
