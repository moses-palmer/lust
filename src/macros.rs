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
/// #[derive(Clone, Copy, Debug, PartialEq)]
/// #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
/// enum Tag {}
/// impl lust::val::Tag for Tag {}
///
/// // The context can be anything, but in order to use the eval! macro, it must implement Default
/// #[derive(Default)]
/// struct Context;
///
/// lust::commands_all! {
///     enum Commands<
///         Tag = Tag,
///         Context = Context,
///     > {
///         /// This command is invoked as `(test 1 ...)`, and the first argument must be an
///         /// unsigned 32 bit integer
///         "test" => Test(script, ctx, env, param: u32, ...args) {
///             args.next()
///                 .map(|e| script.value(e, ctx, env))
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
                    $script_name:ident,
                    $ctx_name:ident,
                    $env_name:ident
                    $(
                        ,$arg_name:ident$(: $arg_type:ty)?
                        $(=> |$transform_arg:ident| $transform:expr)?
                    )*
                    $(
                        ,...$($args_name:ident)?
                        $(=> |$va_transform_node:ident, $va_transform_arg:ident| $va_transform:expr)?
                    )?
                    $(,)?
                )
                $($code:block)?
            )*
        }
    ) => {
        $(#[$struct_meta])*
        #[derive(Clone, Debug, PartialEq)]
        #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
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
                head: &'a $crate::ast::Node,
                tail: &'a [$crate::ast::Node],
            ) -> ::std::result::Result<
                $crate::Expression<Self>,
                $crate::exp::Error<'a>,
            > {
                use $crate::{ast, exp::*};

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
                                                (|$transform_arg: &ast::Node| {
                                                    $transform
                                                })(argument_node)))
                                        )?
                                        .unwrap_or_else(|| Expression::try_from(argument_node))?
                                }),*
                            ];

                            // ...then handle the varargs...
                            for n in iter {
                                arguments.push(Expression::try_from(n)?);
                            }
                            arguments
                        };

                        // ...and finally apply the varargs transform if provided; this may
                        // override the command returned
                        $($(
                            return (
                                |
                                    $va_transform_node,
                                    #[allow(unused_mut)]
                                    mut $va_transform_arg: Vec<Expression<Self>>,
                                | {
                                    $va_transform
                                }
                            )(head, arguments);
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

            fn evaluate<'a, 'b>(
                &'a self,
                script: &'a $crate::Script<Self>,
                ctx: &Self::Context,
                env: &$crate::Environment<'a, 'b, Self>,
            ) -> $crate::exp::Result<'a, Self> {
                // Add a few type aliases and uses for convenience
                #[allow(unused)]
                use $crate::{ast, exp::*, val};
                type Tag = <$name as $crate::Command>::Tag;
                #[allow(unused)]
                type Value<'a> = $crate::Value<'a, Tag>;

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
                                let $arg_name: $arg_type = script.value($arg_name, ctx, env)?
                                    .try_into()
                                    .map_err($crate::exp::Error::from)?;
                            )?
                        )*
                        $($(
                            #[allow(unused)]
                            let mut $args_name = args;
                        )?)?
                        {
                            let $script_name = script;
                            let $ctx_name = ctx;
                            let $env_name = env;
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
                "+" => Add(script, ctx, env, first: f32, ...rest) {
                    rest.fold(
                        Ok(first),
                        |acc, i| Ok(acc? + f32::try_from(script.value(i, ctx, env)?)?)
                    )
                    .map(Value::from)
                }

                /// Subtract all values passed from the first one.
                ///
                /// # Examples
                /// ```lisp
                /// (- 3 2 1) ; 0
                /// ```
                "-" => Subtract(script, ctx, env, first: f32, ...rest) {
                    rest.fold(
                        Ok(first),
                        |acc, i| Ok(acc? - f32::try_from(script.value(i, ctx, env)?)?)
                    )
                    .map(Value::from)
                }

                /// Multiply all values passed.
                ///
                /// # Examples
                /// ```lisp
                /// (* 1 2 3) ; 6
                /// ```
                "*" => Multiply(script, ctx, env, first: f32, ...rest) {
                    rest.fold(
                        Ok(first),
                        |acc, i| Ok(acc? * f32::try_from(script.value(i, ctx, env)?)?)
                    )
                    .map(Value::from)
                }

                /// Divide all values passed.
                ///
                /// # Examples
                /// ```lisp
                /// (/ 8 4) ; 2
                /// ```
                "/" => Divide(script, ctx, env, first: f32, ...rest) {
                    rest.fold(
                        Ok(first),
                        |acc, i| {
                            let a = acc?;
                            let b = f32::try_from(script.value(i, ctx, env)?)?;
                            if b != 0.0 {
                                Ok((a / b).into())
                            } else {
                                $crate::fail!("division by zero");
                            }
                        }
                    )
                    .map(Value::from)
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
            ),*> impl ( arithmetic ) {
                $($rest)*
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Environment, Expression, Value, ast,
        exp::Error,
        test_helpers::{Context, Tag},
        val::owned,
    };

    commands_all! {
        pub enum Command<
            Tag = Tag,
            Context = Context,
        > {
            "fixed" => Fixed(script, ctx, env, a, b) {
                script.value(b, ctx, env)
            }
            "optional" => Optional(script, ctx, env, a, ...b) {
                b.last().map(|e| script.value(e, ctx, env))
                    .unwrap_or_else(|| script.value(a, ctx, env))
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
        let actual = Expression::<Command>::try_from(&ast);

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
        let actual = Expression::<Command>::try_from(&ast);

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
        let actual = Expression::<Command>::try_from(&ast);

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn additional_arguments_allowed_for_optional() {
        // Arrange
        let script = "(optional 1 2 3 4)";
        let expected = Ok(Value::from(4.0).try_into().unwrap());

        // Act
        let ast = ast::parse(&mut ast::tokenize(script)).unwrap();
        let expression = Expression::<Command>::try_from(&ast)
            .expect("compiles")
            .link();
        let actual = expression.evaluate(&Context, &Environment::empty());

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
        let expression = Expression::<Command>::try_from(&ast).expect("compiles");
        let actual = {
            let mut r = Vec::new();
            expression.for_each(|e| r.push(e.to_string()));
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
        let expected_result1 = owned::Value::from(Value::from(7.0));
        let expected_result2 = owned::Value::from(Value::from(8.0));

        // Act
        let ast = ast::parse(&mut ast::tokenize(script)).unwrap();
        let mut expression = Expression::<Command>::try_from(&ast).expect("compiles");
        let actual_order = {
            let mut r = Vec::new();
            expression.for_each_mut(|e| r.push(e.to_string()));
            r
        };
        let actual_result1 = owned::Value::try_from(
            expression
                .clone()
                .link()
                .evaluate(&Context, &Environment::empty())
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
                .evaluate(&Context, &Environment::empty())
                .expect("evaluates"),
        )
        .expect("serializable");

        assert_eq!(expected_order, actual_order);
        assert_eq!(expected_result1, actual_result1);
        assert_eq!(expected_result2, actual_result2);
    }
}
