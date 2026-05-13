# LUST - LISP in Rust

This is a library that enables sciptability of applications using a simplified LISP.


## Quickstart

Use the following code to define a standard LUST interpreter:

```rust
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
# #[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
struct Tag;
lust::tag!(Tag);

lust::commands_all! {
#   #[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
    enum Commands<
        Context = (),
        Tag = Tag,
    > {
        // Define custom commands
    }
}
```

This will define an `enum` `Commands` with a variant for each LUST command. A runnable instance can
be created using the following code:

```rust
# #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
# #[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
# struct Tag; lust::tag!(Tag); lust::commands_all! {
# #[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
# enum Commands<Context = (), Tag = Tag> {} }
# fn main() -> Result<(), String> {
let script = r#"
(let
    (
        (a 5)
        (b (* 0.1 0.3)))
    (+ a b))
"#.parse::<lust::Script<Commands>>()?;
assert_eq!(
    script.evaluate(
        &lust::alloc::zero::Allocator::default(),
        &(),
    ),
    Ok(5.03.into()),
);
# Ok(()) }
```

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
        // "debug" is the name of the command in scripts, and Debug is its enum variant name
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


## The _Commands_ `enum`

This `enum` controls what commands are available. It can be constructed using the
[`lust::commands`](crate::commands) or the [`lust::commands_all`](crate::commands_all) macros. The
former constructs an empty enum by default, whereas the latter includes all predefined commands.

It is possible to include only a selection of predefined commnds:

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

## The `Tag` type

TODO: Write documentation


## The `Context` type

TODO: Write documentation, and mention the trait and composing tuples.


## Values

A value in a LUST script has one of several different types:

*  _Void_: an empty list.
*  _AST_: an abstract syntax tree.
*  _Tag_: a tagged value passed into the script when running it.
*  _Boolean_: either `true` or `false`.
*  _Number_: a floating point number.
*  _Atom_: a symbol name.
*  _String_: a string.
*  _Lambda_: an invokable lambda.
*  _List_: a list of values.


## Examples

