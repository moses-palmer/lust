#[macro_export]
macro_rules! extract {
    ($source:expr, $pattern:pat => $value:expr, $node:ident, $message:literal $(,)?) => {
        $crate::extract!($source, $pattern => $value).ok_or_else(|| $crate::exp::Error::Eval {
            node: $node,
            message: $message,
        })
    };
    ($source:expr, $pattern:pat => $value:expr $(,)?) => {
        match $source {
            $pattern => Some($value),
            _ => None,
        }
    };
}

#[macro_export]
macro_rules! capture {
    ($_t:tt => $sub:expr) => {
        $sub
    };
}

#[macro_export]
macro_rules! fail {
    ($message:literal) => {
        return Err($crate::val::Error::Operation($message).into())
    };
}

/// Defines a type as a tag for values.
///
/// This macto also implements converting from a value of the tag type to the tag type itself.
#[macro_export]
macro_rules! tag {
    ($name:ty) => {
        impl $crate::val::Tag for $name {}

        impl From<$name> for $crate::val::Value<'_, $name> {
            fn from(value: $name) -> Self {
                $crate::val::Value::Tag(value)
            }
        }

        impl TryFrom<$crate::val::Value<'_, $name>> for $name {
            type Error = $crate::val::Error;

            fn try_from(value: $crate::val::Value<'_, Tag>) -> Result<Self, Self::Error> {
                use $crate::val::Value::*;
                match value {
                    Tag(tag) => Ok(tag),
                    _ => Err($crate::val::Error::Conversion {
                        from_type: value.type_name(),
                        to_type: "tag",
                    }),
                }
            }
        }
    };
}

/// Defines a collection of built-in commands.
///
/// This macro is used to define the generic type for an executable
/// [`Expression`](crate::Expression).
///
/// ```
/// # fn main() -> Result<(), lust::eval::Error> {
/// # use lust::*;
/// #
/// // Tagged values must implement a few traits
/// #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
/// #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
/// enum Tag {}
/// lust::tag!(Tag);
///
/// // The context can be anything implementing lust::exp::cmd::Context, but in order to use the
/// // eval! macro, it must implement Default
/// #[derive(Default)]
/// struct Context;
/// impl lust::Context for Context {}
///
/// // The context can control evaluation; using `ContrainedCommands`, which has `AtomicIsize` as
/// // its context type, will yield an error if the expression is too complex
/// lust::commands_all! {
///     #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
///     enum ConstrainedCommands<
///         Tag = Tag,
///         Context = lust::exp::cmd::ResourceConstrainer,
///     > { }
/// }
///
/// fn constrain(resources: isize, script: &str) -> Result<val::owned::Value<Tag>, eval::Error> {
///     let ast = ast::parse(&mut ast::tokenize(script))
///         .map_err(eval::Error::from)?;
///     let script = Expression::<ConstrainedCommands>::parse(
///         &mut Default::default(),
///         &ast,
///     )?.link();
///     let alloc = alloc::bounded::Allocator::<128, _>::default();
///     script.evaluate(&alloc, &resources.into()).map_err(eval::Error::from)
/// }
///
/// assert_eq!(
///     constrain(4, r"
///         ; One to evaluate the root
///         (
///             ; One to evaluate the + command, and one for each argument
///             + 1 2 3)"),
///     Err(eval::Error::Eval("no more executions permitted".into())),
/// );
/// assert_eq!(
///     constrain(5, r"
///         ; One to evaluate the root
///         (
///             ; One to evaluate the + command, and one for each argument
///             + 1 2 3)"),
///     Ok((1 + 2 + 3).into()),
/// );
/// assert_eq!(
///     constrain(26, r"
///         (let
///             (
///                 (f (lambda (a b) (do
///                     (+ a b b)))))
///             (do
///                 (f 1 2)
///                 (f 1 2)
///             ))"),
///     Err(eval::Error::Eval("no more executions permitted".into())),
/// );
/// assert_eq!(
///     constrain(27, r"
///         (let
///             (
///                 (f (lambda (a b) (do
///                     (+ a b b)))))
///             (do
///                 (f 1 2)
///                 (f 1 2)
///             ))"),
///     Ok((1 + 2 + 2).into()),
/// );
///
/// lust::commands_all! {
///     #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
///     enum Commands<
///         Tag = Tag,
///         Context = Context,
///     > {
///         /// This command is invoked as `(test 1 ...)`, and the first argument must be an
///         /// unsigned 32 bit integer
///         "test" => Test(ctx, param: u32, ...args) {
///             args.next()
///                 .map(|e| ctx.value(e))
///                 .unwrap_or_else(|| Ok(Value::NIL))
///         }
///     }
/// }
///
/// let result = eval!("(test 1 2 3)" => Commands);
/// println!("Result: {result}");
///
/// assert_eq!(
///     eval!("(+ 1 2 3)" => Commands),
///     (1 + 2 + 3).into(),
/// );
///
/// assert_eq!(
///     eval!("(- 1 2 3)" => Commands),
///     (1 - 2 - 3).into(),
/// );
///
/// assert_eq!(
///     eval!("(* 2 3 4)" => Commands),
///     (2 * 3 * 4).into(),
/// );
///
/// assert_eq!(
///     eval!("(/ 2 3 4)" => Commands),
///     (2. / 3. / 4.).into(),
/// );
///
/// assert_eq!(
///     eval!("(mod 9 4)" => Commands),
///     1.0.into(),
/// );
///
/// assert_eq!(
///     eval!("(abs -2)" => Commands),
///     2.0.into(),
/// );
///
/// assert_eq!(
///     eval!("(min 1 2 3)" => Commands),
///     1.0.into(),
/// );
///
/// assert_eq!(
///     eval!("(max 1 2 3)" => Commands),
///     3.0.into(),
/// );
///
/// assert_eq!(
///     eval!("(or false false true)" => Commands),
///     true.into(),
/// );
///
/// assert_eq!(
///     eval!("(and true true false)" => Commands),
///     false.into(),
/// );
///
/// assert_eq!(
///     eval!("(xor true true false true)" => Commands),
///     true.into(),
/// );
///
/// assert_eq!(
///     eval!("(let ((a 1) (b 2)) (+ a b 3))" => Commands),
///     (1 + 2 + 3).into(),
/// );
///
/// assert_eq!(
///     eval!("(let ((a (lambda (a1 a2) (+ a1 a2)))) (a 3 5))" => Commands),
///     (3 + 5).into(),
/// );
///
/// assert_eq!(
///     eval!("(if (= 1 2) 1 42)" => Commands),
///     42.into(),
/// );
///
/// assert_eq!(
///     eval!("(if (= 1 2) 1)" => Commands),
///     ().into(),
/// );
///
/// assert_eq!(
///     eval!("(< 1 2)" => Commands),
///     (1 < 2).into(),
/// );
///
/// assert_eq!(
///     eval!("(<= 2 1)" => Commands),
///     (2 <= 1).into(),
/// );
///
/// assert_eq!(
///     eval!("(= 1 (- 2 1))" => Commands),
///     (1 == (2 - 1)).into(),
/// );
///
/// assert_eq!(
///     eval!("(= 1 2)" => Commands),
///     (1 == 2).into(),
/// );
///
/// assert_eq!(
///     eval!("(= \"a string\" \"a string\")" => Commands),
///     ("a string" == "a string").into(),
/// );
///
/// assert_eq!(
///     eval!("(= 1 true)" => Commands),
///     false.into(),
/// );
///
/// assert_eq!(
///     eval!("(!= 1 2)" => Commands),
///     true.into(),
/// );
///
/// assert_eq!(
///     eval!("(>= 2 1)" => Commands),
///     (2 >= 1).into(),
/// );
///
/// assert_eq!(
///     eval!("(> 1 0)" => Commands),
///     (1 > 0).into(),
/// );
///
/// assert_eq!(
///     eval!("(list 1 2 3 5 8)" => Commands),
///     vec![1.into(), 2.into(), 3.into(), 5.into(), 8.into()].into(),
/// );
///
/// assert_eq!(
///     eval!("(list 1 2 3 5 8)" => Commands),
///     vec![1.into(), 2.into(), 3.into(), 5.into(), 8.into()].into(),
/// );
///
/// assert_eq!(
///     eval!("(cdr (list 1 2 3 5 8))" => Commands),
///     vec![2.into(), 3.into(), 5.into(), 8.into()].into(),
/// );
///
/// # Ok(())
/// # }
/// ```
///
/// # Defining commands
///
/// A command definition consists of four major parts.
///
/// The first is the name used in scripts. This must be a valid
/// [atom](crate::ast::token::Value::Atom).
///
/// The second is the internal enum name for the command.
///
/// The third is an argument list enclosed in parentheses. The three two arguments are a linked
/// script host, a context and the current variable scope.
///
/// *  The context is a value that is generally used to provide access to the context from within
///    command handlers.
/// *  The current variable scope, or environment, is used for variable lookup.
///
/// Following these always present arguments is a list of required command arguments. They may have
/// any type that is constructible from a [value](crate::Value). If less arguments are present in
/// the script, compilation will fail. If no type is given for an argument, it will be passed on as
/// an [expression](crate::Expression).
///
/// A required argument may be annotated with a transform by prepending the expression `=> |n| {
/// ... }`. This defines a closure that, given an AST node, will yield a replacement
/// [expression value](crate::Expression).
///
/// Following the list of required arguments is an optional argument preceded with `...`. If this
/// is present, the command will support optional arguments. An iterator of
/// [expressions](crate::Expression) will be available as the declared name. If this argument is
/// not present, compilation will fail if more arguments are provided. The optional argument list
/// also supports transformations, but it will be provided the full list of parsed arguments, and
/// it is expected to return an [expression](crate::Expression). To apply this final transformation
/// for commands without optional arguments, do not provide a name for the optional argument.
///
/// Following the argument list is a block with the actual implementation of the command. This
/// block must result in a [script result](crate::exp::Result), and it may use any try
/// operations which are compatible with [expression errors](crate::exp::Error).
#[macro_export]
macro_rules! commands {
    (
        $(#[$struct_meta:meta])*
        $enum_vis:vis enum $name:ident<$(
            $associated_type:ident = $concrete_type:ty
        ),* $(,)?> $(impl ())? {
            $(
                $(#[$command_meta:meta])*
                $atom:literal => $command:ident(
                    $ctx_name:ident
                    $(
                        ,$arg_name:ident$(: $arg_type:ty)?
                        $(
                            => |
                                $transform_ctx:ident,
                                $transform_node:ident $(,)?
                            | $transform_expr:expr
                            $(
                                => |
                                    $finalize_ctx:ident,
                                    $finalize_arg:ident $(,)?
                                | $finalize_expr:expr
                            )?
                        )?
                    )*
                    $(
                        ,...$($args_name:ident)?
                        $(=> |
                            $va_transform_ctx:ident,
                            $va_transform_node:ident,
                            $va_transform_arg:ident $(,)?
                        | $va_transform_expr:expr)?
                    )?
                    $(,)?
                )
                $($code:block)?
            )*
        }
    ) => {
        $(#[$struct_meta])*
        #[derive(Clone, Debug, PartialEq)]
        #[doc = "# Commands"]
        $(
            #[doc = concat!(
                "### `(",
                $atom,
                $(
                    concat!(" ", stringify!($arg_name)),
                )*
                $($(
                     concat!(" ...", stringify!($args_name)),
                )?)?
                ")`",
            )]
            $(#[$command_meta])*
        )*
        $enum_vis enum $name {
            $(
                #[doc(hidden)]
                $command(Vec<$crate::Expression<Self>>),
            )*
        }

        impl $crate::Command for $name {
            $(
                type $associated_type = $concrete_type;
            )*

            fn name(&self) -> &'static str {
                match self {
                    $(
                        Self::$command(_) => $atom,
                    )*
                }
            }

            fn arguments(&self) -> &[$crate::Expression<Self>] {
                match self {$(
                    Self::$command(args) => args,
                )*}
            }

            fn arguments_mut(&mut self) -> &mut [$crate::Expression<Self>] {
                match self {$(
                    Self::$command(args) => args,
                )*}
            }

            fn parse<'a>(
                context: &mut $crate::exp::ParseContext,
                head: &'a $crate::ast::Node,
                tail: &'a [$crate::ast::Node],
            ) -> ::std::result::Result<
                $crate::Expression<Self>,
                $crate::exp::Error<'a>,
            > {
                use $crate::{ast, exp::*, lambda};

                let command_name = if let ast::NodeValue::Leaf(ast::Value::Atom {
                    value,
                }) = head.value() {
                    value
                } else {
                    return Err($crate::exp::Error::Eval {
                        node: head,
                        message: "unexpected node",
                    })
                };

                match command_name.as_str() {$(
                    $atom => {
                        let arguments = tail;

                        // Calculate the number of required arguments by incrementing a value once
                        // for each argument
                        let arguments_required = {
                            #[allow(unused_mut)]
                            let mut i = 0;
                            $($crate::capture!($arg_name => i += 1);)*
                            i
                        };

                        // Construct a range for the range of allowed arguments; if varargs are
                        // allowed, make it open-ended
                        #[allow(unused)]
                        let arguments_range = arguments_required..=arguments_required;
                        $($(
                            let arguments_range = $crate::capture!(
                                $args_name => arguments_required..
                            );
                        )?)?

                        if !arguments_range.contains(&arguments.len()) {
                            return Err(Error::InvalidInvocation {
                                expected: arguments_required,
                                actual: arguments.len(),
                            })
                        }

                        #[allow(unused)]
                        let arguments = {
                            let mut iter = arguments.iter();

                            // First extract our named arguments...
                            #[allow(unused)]
                            let mut arguments: Vec<Expression<Self>> = vec![
                                $({
                                    // We have already verified the length
                                    let argument_node = iter.next().unwrap();

                                    // Extract the expression using a transform if specified
                                    Option::<::std::result::Result<Expression<Self>, Error>>::None
                                        $(
                                            .or_else(|| Some(
                                                (|
                                                    $transform_ctx: &mut ParseContext,
                                                    $transform_node: &ast::Node,
                                                | {
                                                    $transform_expr
                                                })(&mut *context, argument_node)))
                                        )?
                                        .unwrap_or_else(|| Expression::parse(
                                            context,
                                            argument_node,
                                        ))?
                                }),*
                            ];

                            // ...the finalise the arguments...
                            $($($(
                                let $finalize_ctx = &mut *context;
                                let $finalize_arg = &arguments;
                                $finalize_expr
                            )?)?)*

                            // ...then handle the varargs...
                            for n in iter {
                                arguments.push(Expression::parse(context, n)?);
                            }
                            arguments
                        };

                        // ...and finally apply the varargs transform if provided; this may
                        // override the command returned
                        $($(
                            return (
                                |
                                    $va_transform_ctx: &mut ParseContext,
                                    $va_transform_node,
                                    #[allow(unused_mut)]
                                    mut $va_transform_arg: Vec<Expression<Self>>,
                                | {
                                    $va_transform_expr
                                }
                            )(context, head, arguments);
                        )?)?

                        // Fall back to simply returning the command
                        #[allow(unreachable_code)]
                        Ok(Expression::Command($name::$command(arguments)))
                    })*

                    // Error on unknown commands
                    n => Err(Error::UnknownReference {
                        value: n.to_string(),
                    }),
                }
            }

            fn evaluate<'a, 'b, A>(
                &'a self,
                script: &'a $crate::Script<Self>,
                alloc: &A,
                ctx: &Self::Context,
                env: &$crate::Environment<'a, 'b, Self>,
            ) -> $crate::exp::Result<'a, Self>
            where
                A: $crate::alloc::Allocator<
                    'a,
                    Item = $crate::Cons<'a, $crate::Value<'a, Self::Tag>>,
                > + 'a,
                Self::Tag: 'a,
            {
                // Add a few type aliases and uses for convenience
                #[allow(unused)]
                use $crate::{ast, exp::*, val};
                type Tag = <$name as $crate::Command>::Tag;

                #[allow(unused)]
                type ASTNode<'a> = &'a $crate::ast::Node;
                #[allow(unused)]
                type ASTKind<'a> = $crate::ast::NodeValue;
                #[allow(unused)]
                type ASTValue<'a> = $crate::ast::Value;
                #[allow(unused)]
                type Cons<'a> = $crate::Cons<'a, Value<'a>>;
                #[allow(unused)]
                type Lambda = $crate::lambda::Ref;
                #[allow(unused)]
                type Value<'a> = $crate::Value<'a, Tag>;
                #[allow(unused)]
                type Values<'a> = $crate::Values<'a, Tag>;

                match self {$(
                    Self::$command(args) => {
                        #[allow(unused_mut)]
                        let mut args = args.into_iter();
                        $(
                            // The length of the argument vector has already been checked
                            let expression = args.next().unwrap();

                            // If we have an argument type, evaluate immediately, otherwise keep
                            // the expression
                            #[allow(unused)]
                            let $arg_name = expression;
                            $(
                                let $arg_name: $arg_type = script.value($arg_name, alloc, ctx, env)?
                                    .try_into()
                                    .map_err($crate::exp::Error::from)?;
                            )?
                        )*
                        $($(
                            #[allow(unused)]
                            let mut $args_name = args;
                        )?)?
                        {
                            let $ctx_name = $crate::exp::linked::EvaluationContext {
                                script,
                                alloc,
                                ctx,
                                env,
                            };
                            Option::<$crate::exp::Result<Self>>::None
                            $(
                                .or(Some($code))
                            )?
                            .unwrap_or_else(|| Ok(().into()))
                        }
                    }
                )*}
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                <Self as $crate::Command>::name(self).fmt(f)
            }
        }
    };

    (
        $(#[$struct_meta:meta])*
        $enum_vis:vis enum $name:ident<$(
            $associated_type:ident = $concrete_type:ty
        ),* $(,)?>
        impl ( arithmetic $(, $rest_features:ident)* )
        {
            $($rest:tt)*
        }
    ) => {
        $crate::commands! {
            $(#[$struct_meta])*
            $enum_vis enum $name<$(
                $associated_type = $concrete_type
            ),*> impl ( $($rest_features),* ) {
                /// Add all values passed.
                ///
                /// # Examples
                /// ```lisp
                /// (+ 1 2 3) ; 6
                /// ```
                "+" => Add(ctx, first: f32, ...rest) {
                    rest.fold(
                        Ok(first),
                        |acc, i| Ok(acc? + f32::try_from(ctx.value(i)?)?)
                    )
                    .map(Value::from)
                }

                /// Subtract all values passed from the first one.
                ///
                /// # Examples
                /// ```lisp
                /// (- 3 2 1) ; 0
                /// ```
                "-" => Subtract(ctx, first: f32, ...rest) {
                    rest.fold(
                        Ok(first),
                        |acc, i| Ok(acc? - f32::try_from(ctx.value(i)?)?)
                    )
                    .map(Value::from)
                }

                /// Multiply all values passed.
                ///
                /// # Examples
                /// ```lisp
                /// (* 1 2 3) ; 6
                /// ```
                "*" => Multiply(ctx, first: f32, ...rest) {
                    rest.fold(
                        Ok(first),
                        |acc, i| Ok(acc? * f32::try_from(ctx.value(i)?)?)
                    )
                    .map(Value::from)
                }

                /// Divide all values passed.
                ///
                /// # Examples
                /// ```lisp
                /// (/ 8 4) ; 2
                /// ```
                "/" => Divide(ctx, first: f32, ...rest) {
                    rest.fold(
                        Ok(first),
                        |acc, i| {
                            let a = acc?;
                            let b = f32::try_from(ctx.value(i)?)?;
                            if b != 0.0 {
                                Ok((a / b).into())
                            } else {
                                $crate::fail!("division by zero");
                            }
                        }
                    )
                    .map(Value::from)
                }

                /// Calculates _a (mod) b_.
                ///
                /// # Examples
                /// ```lisp
                /// (mod 9 4) ; 1
                /// ```
                "mod" => Mod(_ctx, a: f32, b: f32) {
                    if b != 0.0 {
                        Ok((a % b).into())
                    } else {
                        $crate::fail!("division by zero");
                    }
                }

                /// Calculates the absolute of a value.
                ///
                /// # Examples
                /// ```lisp
                /// (abs -2) ; 2
                /// ```
                "abs" => Abs(_ctx, a: f32) {
                    Ok(a.abs().into())
                }

                /// Calculates the minimum value.
                ///
                /// # Examples
                /// ```lisp
                /// (min 1 2 3) ; 1
                /// ```
                "min" => Min(ctx, first: f32, ...rest) {
                    rest.fold(
                        Ok(first),
                        |acc, i| Ok(acc?.min(f32::try_from(ctx.value(i)?)?))
                    )
                    .map(Value::from)
                }

                /// Calculates the maximum value.
                ///
                /// # Examples
                /// ```lisp
                /// (max 1 2 3) ; 3
                /// ```
                "max" => Max(ctx, first: f32, ...rest) {
                    rest.fold(
                        Ok(first),
                        |acc, i| Ok(acc?.max(f32::try_from(ctx.value(i)?)?))
                    )
                    .map(Value::from)
                }

                $($rest)*
            }
        }
    };

    (
        $(#[$struct_meta:meta])*
        $enum_vis:vis enum $name:ident<$(
            $associated_type:ident = $concrete_type:ty
        ),* $(,)?>
        impl ( control $(, $rest_features:ident)* )
        {
            $($rest:tt)*
        }
    ) => {
        $crate::commands! {
            $(#[$struct_meta])*
            $enum_vis enum $name<$(
                $associated_type = $concrete_type
            ),*> impl ( $($rest_features),* ) {
                /// Sequentially evaluate a list of expressions and return the last value
                ///
                /// # Examples
                /// ```lisp
                /// (do 1 2 3) ; 3
                /// ```
                "do" => Do(
                    ctx,
                    ...expressions,
                ) {
                    expressions
                        .fold(
                            Ok(Value::NIL),
                            |acc, e| {
                                acc?;
                                ctx.value(e)
                            },
                        )
                }

                /// Evaluate expressions conditionally.
                ///
                /// The `false` value is optional; if not provided, this command returns `void`.
                ///
                /// # Examples
                /// ```lisp
                /// (let ((a 1) (b 2)) (if (> a b) 3 4)) ; 4
                /// ```
                "if" => If(
                    ctx,
                    cond: bool,
                    if_true,
                    ...if_false => |_ctx, _node, a| {
                        // Either we return the third argument or nil on false
                        match a.len() {
                            2 => Ok(Expression::<Self>::Void),
                            3 => Ok(a.pop().unwrap()),
                            _ => Err(Error::Syntax { message: "at most one else clause expected" }),
                        }
                    },
                ) {
                    ctx.value(if cond { if_true } else { if_false.next().unwrap() })
                }

                $($rest)*
            }
        }
    };

    (
        $(#[$struct_meta:meta])*
        $enum_vis:vis enum $name:ident<$(
            $associated_type:ident = $concrete_type:ty
        ),* $(,)?>
        impl ( boolean $(, $rest_features:ident)* )
        {
            $($rest:tt)*
        }
    ) => {
        $crate::commands! {
            $(#[$struct_meta])*
            $enum_vis enum $name<$(
                $associated_type = $concrete_type
            ),*> impl ( $($rest_features),* ) {
                /// _¬a_
                ///
                /// # Examples
                /// ```lisp
                /// (not false) ; true
                /// ```
                "not" => Not(_ctx, a: bool) {
                    Ok((!a).into())
                }

                /// _a ⋀ ..._
                ///
                /// # Examples
                /// ```lisp
                /// (and true true false) ; false
                /// ```
                "and" => And(ctx, a: bool, ...values) {
                    values.fold(
                        Ok(a),
                        |acc, e| Ok(acc? && bool::try_from(ctx.value(e)?)?),
                    )
                    .map(Value::from)
                }

                /// _a ⋁ ..._
                ///
                /// # Examples
                /// ```lisp
                /// (or false false true) ; true
                /// ```
                "or" => Or(ctx, a: bool, ...values) {
                    values.fold(
                        Ok(a),
                        |acc, e| Ok(acc? || bool::try_from(ctx.value(e)?)?),
                    )
                    .map(Value::from)
                }

                /// _a ⊕ ..._
                ///
                /// # Examples
                /// ```lisp
                /// (xor true true false true) ; true
                /// ```
                "xor" => Xor(ctx, a: bool, ...values) {
                    values.fold(
                        Ok(a),
                        |acc, e| Ok(acc? ^ bool::try_from(ctx.value(e)?)?),
                    )
                    .map(Value::from)
                }

                $($rest)*
            }
        }
    };

    (
        $(#[$struct_meta:meta])*
        $enum_vis:vis enum $name:ident<$(
            $associated_type:ident = $concrete_type:ty
        ),* $(,)?>
        impl ( let $(, $rest_features:ident)* )
        {
            $($rest:tt)*
        }
    ) => {
        $crate::commands! {
            $(#[$struct_meta])*
            $enum_vis enum $name<$(
                $associated_type = $concrete_type
            ),*> impl ( $($rest_features),* ) {
                /// Bind variables to values.
                ///
                /// The defined variables will be available in `body`
                ///
                /// # Examples
                /// ```lisp
                /// (let ((a 1) (b 2)) (+ a b)) ; 3
                /// ```
                "let" => Let(
                    ctx,
                    definitions
                        => |ctx, n| {
                            if let Some(Expression::Map(names, values)) = Expression::as_map(
                                ctx,
                                n,
                            ) {
                                ctx.scope.extend(names.iter().map(Clone::clone));
                                Ok(Expression::List(values))
                            } else {
                                Err(Error::Syntax {
                                    message: "expected map",
                                })
                            }
                        }
                        => |ctx, a| {
                            if let Some(Expression::Map(names, _)) = a.get(0) {
                                ctx.scope.truncate(ctx.scope.len() - names.len());
                            }
                        },
                    body,
                ) {
                    let expressions = $crate::extract!(definitions, Expression::List(v) => v)
                        .unwrap();
                    let values = expressions.iter()
                        .map(|e| ctx.value(e))
                        .collect::<::std::result::Result<Values, _>>()?;
                    ctx.script.value(body, ctx.alloc, ctx.ctx, &ctx.env.with_scope(&values))
                }

                $($rest)*
            }
        }
    };

    (
        $(#[$struct_meta:meta])*
        $enum_vis:vis enum $name:ident<$(
            $associated_type:ident = $concrete_type:ty
        ),* $(,)?>
        impl ( lambda $(, $rest_features:ident)* )
        {
            $($rest:tt)*
        }
    ) => {
        $crate::commands! {
            $(#[$struct_meta])*
            $enum_vis enum $name<$(
                $associated_type = $concrete_type
            ),*> impl ( $($rest_features),* ) {
                /// Create a lambda.
                ///
                /// To later invoke the lambda, bind it to a variable using `let`.
                ///
                /// A lambda does not capture its environment; the only variables available are
                /// the arguments.
                ///
                /// # Examples
                /// ```lisp
                /// (let ((add (lambda (a b) (+ a b)))) (add 1 2)) ; 3
                /// ```
                "lambda" => Lambda(
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
                    ... => |_ctx, _node, a| {
                        let argument_count = $crate::extract!(&a[0], Expression::AST(v) => v.len())
                            .unwrap();
                        let body = a.pop().unwrap();

                        Ok(Expression::LambdaDef(vec![lambda::Lambda::new(argument_count, body)]))
                    })

                $($rest)*
            }
        }
    };

    (
        $(#[$struct_meta:meta])*
        $enum_vis:vis enum $name:ident<$(
            $associated_type:ident = $concrete_type:ty
        ),* $(,)?>
        impl ( cmp $(, $rest_features:ident)* )
        {
            $($rest:tt)*
        }
    ) => {
        $crate::commands! {
            $(#[$struct_meta])*
            $enum_vis enum $name<$(
                $associated_type = $concrete_type
            ),*> impl ( $($rest_features),* ) {
                /// Check whether `a < b`.
                ///
                /// # Examples
                /// ```lisp
                /// (< 1 2) ; true
                /// ```
                "<" => Lt(_ctx, a: Value, b: Value) {
                    Ok((a < b).into())
                }

                /// Check whether `a <= b`.
                ///
                /// # Examples
                /// ```lisp
                /// (<= 1 2) ; true
                /// ```
                "<=" => LtE(_ctx, a: Value, b: Value) {
                    Ok((a <= b).into())
                }

                /// Check whether `a == b`.
                ///
                /// # Examples
                /// ```lisp
                /// (= 1 2) ; false
                /// ```
                "=" => Eq(_ctx, a: Value, b: Value) {
                    Ok((a == b).into())
                }

                /// Check whether `a != b`.
                ///
                /// # Examples
                /// ```lisp
                /// (!= 1 2) ; true
                /// ```
                "!=" => Neq(_ctx, a: Value, b: Value) {
                    Ok((a != b).into())
                }

                /// Check whether `a >= b`.
                ///
                /// # Examples
                /// ```lisp
                /// (>= 1 2) ; false
                /// ```
                ">=" => GtE(_ctx, a: Value, b: Value) {
                    Ok((a >= b).into())
                }

                /// Check whether `a > b`.
                ///
                /// # Examples
                /// ```lisp
                /// (> 1 2) ; false
                /// ```
                ">" => Gt(_ctx, a: Value, b: Value) {
                    Ok((a > b).into())
                }

                $($rest)*
            }
        }
    };

    (
        $(#[$struct_meta:meta])*
        $enum_vis:vis enum $name:ident<$(
            $associated_type:ident = $concrete_type:ty
        ),* $(,)?>
        impl ( list $(, $rest_features:ident)* )
        {
            $($rest:tt)*
        }
    ) => {
        $crate::commands! {
            $(#[$struct_meta])*
            $enum_vis enum $name<$(
                $associated_type = $concrete_type
            ),*> impl ( $($rest_features),* ) {
                /// Construct a list.
                ///
                /// # Example
                /// ```lisp
                /// (list 1 2 3) ; (1 2 3)
                /// ```
                "list" => List(ctx, ...items) {
                    items.rev()
                        .try_fold(
                            Value::NIL,
                            |acc, e| {
                                let v = ctx.value(e)?;
                                if let Value::List(cons) = acc {
                                    Ok(ctx.alloc(cons.prepend(v))?.into())
                                } else {
                                    Ok(ctx.alloc(Cons::single(v))?.into())
                                }
                            },
                        )
                }

                /// Get the head of a list.
                ///
                /// # Example
                /// ```lisp
                /// (car (list 1 2 3)) ; 1
                /// ```
                "car" => Car(_ctx, cons: &'a Cons<'a>) {
                    Ok(*cons.car())
                }

                /// Get the tail of a list.
                ///
                /// # Example
                /// ```lisp
                /// (cdr (list 1 2 3)) ; (2 3)
                /// ```
                "cdr" => Cdr(_ctx, cons: &'a Cons<'a>) {
                    Ok(cons.cdr().next().map(Into::into).unwrap_or(Value::NIL))
                }

                $($rest)*
            }
        }
    };
}

/// Defines a collection of built-in commands with all standard commands available.
#[macro_export]
macro_rules! commands_all {
    (
        $(#[$struct_meta:meta])*
        $enum_vis:vis enum $name:ident<$(
            $associated_type:ident = $concrete_type:ty
        ),* $(,)?> {
            $($rest:tt)*
        }
    ) => {
        $crate::commands! {
            $(#[$struct_meta])*
            $enum_vis enum $name<$(
                $associated_type = $concrete_type
            ),*> impl ( boolean, control, arithmetic, let, lambda, cmp, list ) {
                $($rest)*
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Cons, Expression, Value, ast,
        exp::Error,
        test_helpers::{Context, Tag},
        val::owned,
    };

    commands_all! {
        #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
        pub enum Command<
            Tag = Tag,
            Context = Context,
        > {
            "fixed" => Fixed(ctx, a, b) {
                ctx.value(b)
            }
            "optional" => Optional(ctx, a, ...b) {
                b.last().map(|e| ctx.value(e)).unwrap_or_else(|| ctx.value(a))
            }
        }
    }

    #[test]
    fn too_few_arguments_fails_for_fixed() {
        // Arrange
        let script = "(fixed 1)";
        let expected = Err(Error::InvalidInvocation {
            expected: 2,
            actual: 1,
        });

        // Act
        let ast = ast::parse(&mut ast::tokenize(script)).unwrap();
        let actual = Expression::<Command>::parse(&mut Default::default(), &ast);

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn too_many_arguments_fails_for_fixed() {
        // Arrange
        let script = "(fixed 1 2 3)";
        let expected = Err(Error::InvalidInvocation {
            expected: 2,
            actual: 3,
        });

        // Act
        let ast = ast::parse(&mut ast::tokenize(script)).unwrap();
        let actual = Expression::<Command>::parse(&mut Default::default(), &ast);

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn too_few_arguments_fails_for_optional() {
        // Arrange
        let script = "(optional)";
        let expected = Err(Error::InvalidInvocation {
            expected: 1,
            actual: 0,
        });

        // Act
        let ast = ast::parse(&mut ast::tokenize(script)).unwrap();
        let actual = Expression::<Command>::parse(&mut Default::default(), &ast);

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn additional_arguments_allowed_for_optional() {
        // Arrange
        let script = "(optional 1 2 3 4)";
        let alloc = crate::alloc::zero::Allocator::<Cons<Value<Tag>>>::default();
        let expected = Ok(Value::from(4.0).try_into().unwrap());

        // Act
        let ast = ast::parse(&mut ast::tokenize(script)).unwrap();
        let expression = Expression::<Command>::parse(&mut Default::default(), &ast)
            .expect("compiles")
            .link();
        let actual = expression.evaluate(&alloc, &Context);

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn expression_for_each() {
        // Arrange
        let script = r"(
            fixed
            (optional 3 1 5)
            (optional 1 3 7))";
        let expected = vec![
            "fixed".to_string(),
            "optional".to_string(),
            "3".to_string(),
            "1".to_string(),
            "5".to_string(),
            "optional".to_string(),
            "1".to_string(),
            "3".to_string(),
            "7".to_string(),
        ];

        // Act
        let ast = ast::parse(&mut ast::tokenize(script)).unwrap();
        let expression =
            Expression::<Command>::parse(&mut Default::default(), &ast).expect("compiles");
        let actual = {
            let mut r = Vec::new();
            expression.for_each(|e| r.push(e.to_string()));
            r
        };

        assert_eq!(expected, actual);
    }

    #[test]
    fn expression_for_each_list() {
        // Arrange
        let expected = vec![
            Expression::<Command>::List(vec![
                Expression::Number(1.0),
                Expression::Number(2.0),
                Expression::List(vec![Expression::Number(3.0), Expression::Number(4.0)]),
            ]),
            Expression::Number(1.0),
            Expression::Number(2.0),
            Expression::List(vec![Expression::Number(3.0), Expression::Number(4.0)]),
            Expression::Number(3.0),
            Expression::Number(4.0),
        ];

        // Act
        let expression = Expression::<Command>::List(vec![
            Expression::Number(1.0),
            Expression::Number(2.0),
            Expression::List(vec![Expression::Number(3.0), Expression::Number(4.0)]),
        ]);
        let actual = {
            let mut r = Vec::new();
            expression.for_each(|e| r.push(e.clone()));
            r
        };

        assert_eq!(expected, actual);
    }

    #[test]
    fn expression_for_each_map() {
        // Arrange
        let expected = vec![
            Expression::<Command>::Map(
                vec!["a".into(), "b".into(), "c".into()],
                vec![
                    Expression::Number(1.0),
                    Expression::Number(2.0),
                    Expression::Map(
                        vec!["d".into(), "e".into()],
                        vec![Expression::Number(3.0), Expression::Number(4.0)],
                    ),
                ],
            ),
            Expression::Number(1.0),
            Expression::Number(2.0),
            Expression::Map(
                vec!["d".into(), "e".into()],
                vec![Expression::Number(3.0), Expression::Number(4.0)],
            ),
            Expression::Number(3.0),
            Expression::Number(4.0),
        ];

        // Act
        let expression = Expression::<Command>::Map(
            vec!["a".into(), "b".into(), "c".into()],
            vec![
                Expression::Number(1.0),
                Expression::Number(2.0),
                Expression::Map(
                    vec!["d".into(), "e".into()],
                    vec![Expression::Number(3.0), Expression::Number(4.0)],
                ),
            ],
        );
        let actual = {
            let mut r = Vec::new();
            expression.for_each(|e| r.push(e.clone()));
            r
        };

        assert_eq!(expected, actual);
    }

    #[test]
    fn expression_for_each_mut() {
        // Arrange
        let script = r"(
            fixed
            (optional 3 1 5)
            (optional 1 3 7))";
        let alloc = crate::alloc::zero::Allocator::<Cons<Value<Tag>>>::default();
        let expected_order = vec![
            "fixed".to_string(),
            "optional".to_string(),
            "3".to_string(),
            "1".to_string(),
            "5".to_string(),
            "optional".to_string(),
            "1".to_string(),
            "3".to_string(),
            "7".to_string(),
        ];
        let expected_result1 = owned::Value::try_from(Value::from(7.0)).expect("owned");
        let expected_result2 = owned::Value::try_from(Value::from(8.0)).expect("owned");

        // Act
        let ast = ast::parse(&mut ast::tokenize(script)).unwrap();
        let mut expression =
            Expression::<Command>::parse(&mut Default::default(), &ast).expect("compiles");
        let actual_order = {
            let mut r = Vec::new();
            expression.for_each_mut(|e| r.push(e.to_string()));
            r
        };
        let actual_result1 = owned::Value::try_from(
            expression
                .clone()
                .link()
                .evaluate(&alloc, &Context)
                .expect("evaluates"),
        )
        .expect("serializable");
        expression.for_each_mut(|e| match e {
            Expression::Number(v) => {
                *v += 1.0;
            }
            _ => {}
        });
        let actual_result2 = owned::Value::try_from(
            expression
                .clone()
                .link()
                .evaluate(&alloc, &Context)
                .expect("evaluates"),
        )
        .expect("serializable");

        assert_eq!(expected_order, actual_order);
        assert_eq!(expected_result1, actual_result1);
        assert_eq!(expected_result2, actual_result2);
    }

    #[test]
    fn expression_for_each_mut_list() {
        // Arrange
        let expected = vec![
            Expression::<Command>::List(vec![
                Expression::Number(1.0),
                Expression::Number(2.0),
                Expression::List(vec![Expression::Number(3.0), Expression::Number(4.0)]),
            ]),
            Expression::Number(1.0),
            Expression::Number(2.0),
            Expression::List(vec![Expression::Number(3.0), Expression::Number(4.0)]),
            Expression::Number(3.0),
            Expression::Number(4.0),
        ];

        // Act
        let mut expression = Expression::<Command>::List(vec![
            Expression::Number(1.0),
            Expression::Number(2.0),
            Expression::List(vec![Expression::Number(3.0), Expression::Number(4.0)]),
        ]);
        let actual = {
            let mut r = Vec::new();
            expression.for_each_mut(|e| r.push(e.clone()));
            r
        };

        assert_eq!(expected, actual);
    }

    #[test]
    fn expression_for_each_mut_map() {
        // Arrange
        let expected = vec![
            Expression::<Command>::Map(
                vec!["a".into(), "b".into(), "c".into()],
                vec![
                    Expression::Number(1.0),
                    Expression::Number(2.0),
                    Expression::Map(
                        vec!["d".into(), "e".into()],
                        vec![Expression::Number(3.0), Expression::Number(4.0)],
                    ),
                ],
            ),
            Expression::Number(1.0),
            Expression::Number(2.0),
            Expression::Map(
                vec!["d".into(), "e".into()],
                vec![Expression::Number(3.0), Expression::Number(4.0)],
            ),
            Expression::Number(3.0),
            Expression::Number(4.0),
        ];

        // Act
        let mut expression = Expression::<Command>::Map(
            vec!["a".into(), "b".into(), "c".into()],
            vec![
                Expression::Number(1.0),
                Expression::Number(2.0),
                Expression::Map(
                    vec!["d".into(), "e".into()],
                    vec![Expression::Number(3.0), Expression::Number(4.0)],
                ),
            ],
        );
        let actual = {
            let mut r = Vec::new();
            expression.for_each_mut(|e| r.push(e.clone()));
            r
        };

        assert_eq!(expected, actual);
    }

    #[test]
    fn expression_link() {
        // Arrange
        let script = r"(lambda (a b) (+ a b))";

        // Act
        let ast = ast::parse(&mut ast::tokenize(script)).unwrap();
        let expression = Expression::<Command>::parse(&mut Default::default(), &ast)
            .expect("compiles")
            .link();
        let actual_order = {
            let mut r = Vec::new();
            expression.for_each(|e| r.push(e.clone()));
            r
        };

        assert_eq!(actual_order.len(), 1);
        assert!(matches!(actual_order[0], Expression::LambdaRef(_)));
    }
}
