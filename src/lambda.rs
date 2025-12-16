use crate::{Command, Environment, Expression, Value, exp};

/// A callable expression.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Lambda<C> {
    /// The names of the arguments.
    arguments: Vec<String>,

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
    /// *  `arguments` - The names of the arguments.
    /// *  `expression` - The lambda body.
    pub fn new(arguments: Vec<String>, expression: Expression<C>) -> Self {
        Self {
            arguments,
            expression,
        }
    }

    /// Evaluates the lambda expression.
    ///
    /// The [environment](Environment) is generated from the argument names and values.
    ///
    /// # Arguments
    /// *  `script` - The linked script.
    /// *  `ctx` - The evaluation context.
    /// *  `arguments` - The arguments to pass.
    pub fn invoke<'a>(
        &'a self,
        script: &'a crate::Script<C>,
        ctx: &C::Context,
        arguments: &[Value<'a, C::Tag>],
    ) -> exp::Result<'a, C> {
        if self.arguments.len() != arguments.len() {
            Err(exp::Error::InvalidInvocation {
                expected: self.arguments.len(),
                actual: arguments.len(),
            })
        } else {
            script.value(
                &self.expression,
                ctx,
                &Environment::empty().with_scope(self.arguments.as_slice(), arguments),
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
        let tested = Lambda {
            arguments: vec!["a".into(), "b".into()],
            expression: Expression::Number(42.0),
        };
        let expected = Err(exp::Error::InvalidInvocation {
            expected: 2,
            actual: 1,
        });

        // Act
        let actual = tested.invoke(&script, &Context, &[Value::Void]);

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn invoke_too_many_arguments() {
        // Arrange
        let script = Default::default();
        let tested = Lambda {
            arguments: vec!["a".into()],
            expression: Expression::Number(42.0),
        };
        let expected = Err(exp::Error::InvalidInvocation {
            expected: 1,
            actual: 2,
        });

        // Act
        let actual = tested.invoke(&script, &Context, &[Value::Void, Value::Void]);

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn invoke() {
        // Arrange
        let script = Default::default();
        let tested = Lambda {
            arguments: vec!["a".into()],
            expression: Expression::Number(42.0),
        };
        let expected = Ok((42.0).into());

        // Act
        let actual = tested.invoke(&script, &Context, &[Value::Void]);

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
            arguments: vec!["a".into()],
            expression: Expression::Number(42.0),
        }
    }
}
