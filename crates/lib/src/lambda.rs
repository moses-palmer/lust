use crate::{Command, Cons, Environment, Expression, Value, alloc, exp};

/// A callable expression.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Lambda<C> {
    /// The number of arguments.
    argument_count: usize,

    /// The lambda expression.
    expression: Expression<C>,
}

impl<C> Lambda<C>
where
    C: Command,
{
    /// Creates a new lambda instance.
    ///
    /// # Arguments
    /// *  `argument_count` - The number of arguments.
    /// *  `expression` - The lambda body.
    pub fn new(argument_count: usize, expression: Expression<C>) -> Self {
        Self {
            argument_count,
            expression,
        }
    }

    /// Evaluates the lambda expression.
    ///
    /// The [environment](Environment) is generated from the argument names and values.
    ///
    /// # Arguments
    /// *  `script` - The linked script.
    /// *  `alloc` - The allocator to use.
    /// *  `ctx` - The evaluation context.
    /// *  `arguments` - The arguments to pass.
    pub fn invoke<'a, A>(
        &'a self,
        script: &'a crate::Script<C>,
        alloc: &A,
        ctx: &C::Context,
        arguments: &[Value<'a, C::Tag>],
    ) -> exp::Result<'a, C>
    where
        A: alloc::Allocator<'a, Item = Cons<'a, Value<'a, C::Tag>>> + 'a,
        <C as Command>::Tag: 'a,
    {
        if self.argument_count != arguments.len() {
            Err(exp::Error::InvalidInvocation {
                expected: self.argument_count,
                actual: arguments.len(),
            })
        } else {
            script.value(
                &self.expression,
                alloc,
                ctx,
                &Environment::empty().with_scope(arguments),
            )
        }
    }
}

/// A collection of lambdas.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Store<C> {
    /// The backing list.
    data: Vec<Lambda<C>>,
}

impl<C> Default for Store<C> {
    fn default() -> Self {
        Self { data: Vec::new() }
    }
}

impl<C> Store<C> {
    /// Registers a new lambda.
    ///
    /// The return value can be used to reference the lambda in a type-erased manner.
    ///
    /// # Arguments
    /// *  `lambda` - The lambda to register.
    pub fn register(&mut self, lambda: Lambda<C>) -> Ref {
        Ref {
            index: {
                self.data.push(lambda);
                self.data.len() - 1
            },
        }
    }

    /// Attempts to resolve a lambda reference.
    ///
    /// # Arguments
    /// *  `lambda_ref` - A lambda reference previously acquired by [`Self::register`].
    pub fn resolve(&self, lambda_ref: Ref) -> Option<&Lambda<C>> {
        self.data.get(lambda_ref.index)
    }
}

/// A type-erased reference to a registered lambda.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Ref {
    /// The index of this reference in this generation.
    index: usize,
}

impl ::std::fmt::Display for Ref {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.index)
    }
}

#[cfg(test)]
mod tests {
    use crate::{exp, test_helpers::*};

    use super::{Lambda, Ref, Store};

    #[test]
    fn invoke_too_few_arguments() {
        // Arrange
        let script = Default::default();
        let alloc = crate::alloc::zero::Allocator::<Cons>::default();
        let tested = Lambda {
            argument_count: 2,
            expression: Expression::Number(42.0),
        };
        let expected = Err(exp::Error::InvalidInvocation {
            expected: 2,
            actual: 1,
        });

        // Act
        let actual = tested.invoke(&script, &alloc, &Context, &[Value::Void]);

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn invoke_too_many_arguments() {
        // Arrange
        let script = Default::default();
        let alloc = crate::alloc::zero::Allocator::<Cons>::default();
        let tested = Lambda {
            argument_count: 1,
            expression: Expression::Number(42.0),
        };
        let expected = Err(exp::Error::InvalidInvocation {
            expected: 1,
            actual: 2,
        });

        // Act
        let actual = tested.invoke(&script, &alloc, &Context, &[Value::Void, Value::Void]);

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn invoke() {
        // Arrange
        let script = Default::default();
        let alloc = crate::alloc::zero::Allocator::<Cons>::default();
        let tested = Lambda {
            argument_count: 1,
            expression: Expression::Number(42.0),
        };
        let expected = Ok((42.0).into());

        // Act
        let actual = tested.invoke(&script, &alloc, &Context, &[Value::Void]);

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn register_single() {
        // Arrange
        let lambda = lambda();
        let mut tested = Store::<Command>::default();
        let expected = Ref { index: 0 };

        // Act
        let actual = tested.register(lambda.clone());

        // Assert
        assert_eq!(expected, actual);
        assert!(tested.resolve(actual).is_some())
    }

    #[test]
    fn register_multiple() {
        // Arrange
        let lambda = lambda();
        let mut tested = Store::<Command>::default();

        // Act
        let actual = (0..10)
            .map(|_| tested.register(lambda.clone()))
            .collect::<Vec<_>>();

        // Assert
        for (index, &actual) in actual.iter().enumerate() {
            let expected = Ref { index };
            assert_eq!(expected, actual);
            assert!(tested.resolve(actual).is_some())
        }
    }

    #[test]
    fn resolve_empty() {
        // Arrange
        let tested = Store::<Command>::default();
        let expected = None;

        // Act
        let actual = tested.resolve(Ref { index: 0 });

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn resolve_invalid_index() {
        // Arrange
        let tested = Store::<Command>::default();
        let expected = None;

        // Act
        let actual = tested.resolve(Ref { index: 0 });

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn resolve() {
        // Arrange
        let lambda = lambda();
        let mut tested = Store::<Command>::default();
        let expected = Some(&lambda);

        // Act
        let lambda_ref = tested.register(lambda.clone());
        let actual = tested.resolve(lambda_ref);

        // Assert
        assert_eq!(expected, actual);
    }

    fn lambda() -> Lambda<Command> {
        Lambda {
            argument_count: 1,
            expression: Expression::Number(42.0),
        }
    }
}
