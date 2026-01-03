use crate::{Command, Cons, Environment, Value, Values, alloc, lambda, val};

/// A linked expression.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Script<C> {
    /// The root expression.
    root: super::Expression<C>,

    /// The lambda store
    lambdas: lambda::Store<C>,
}

impl<C> Script<C>
where
    C: Command,
{
    /// Evaluates the root expression given a context and environment.
    ///
    /// # Arguments
    /// *  `alloc` - The allocator to use.
    /// *  `ctx` - The context of the evaluation.
    /// *  `env` - The current variable scope.
    pub fn evaluate<'a, 'b, A>(
        &'a self,
        alloc: &A,
        ctx: &C::Context,
        env: &Environment<'a, 'b, C>,
    ) -> Result<val::owned::Value<C::Tag>, super::Error<'a>>
    where
        A: alloc::Allocator<'a, Item = Cons<'a, Value<'a, C::Tag>>> + 'a,
        <C as Command>::Tag: 'a,
    {
        self.value(&self.root, alloc, ctx, env)
            .and_then(|v| Ok(v.try_into()?))
    }

    /// Evaluates an expression given a context and environment.
    ///
    /// # Arguments
    /// *  `alloc` - The allocator to use.
    /// *  `ctx` - The context of the evaluation.
    /// *  `env` - The current variable scope.
    pub fn value<'a, 'b, A>(
        &'a self,
        e: &'a super::Expression<C>,
        alloc: &A,
        ctx: &C::Context,
        env: &Environment<'a, 'b, C>,
    ) -> super::Result<'a, C>
    where
        A: alloc::Allocator<'a, Item = Cons<'a, Value<'a, C::Tag>>> + 'a,
        <C as Command>::Tag: 'a,
    {
        use super::Expression::*;
        match e {
            List(v) => {
                let mut i = v.iter();
                if let Some(head) = i.next() {
                    let head = self.value(head, alloc, ctx, env)?;
                    if let Value::Lambda(lambda_ref) = head {
                        let arguments = i
                            .map(|e| self.value(e, alloc, ctx, env))
                            .collect::<Result<Values<_>, _>>()?;
                        self.invoke(alloc, ctx, lambda_ref, &arguments)
                            .unwrap_or_else(|| Err(val::Error::Operation("unknown lambda").into()))
                    } else {
                        i.try_fold(head, |_, e| self.value(e, alloc, ctx, env))
                    }
                } else {
                    Ok(Value::NIL)
                }
            }
            Map(_, _) => Err(super::Error::from(val::Error::Operation(
                "cannot evaluate map",
            ))),
            AST(v) => Ok(Value::AST(v)),
            Reference(v) => env
                .resolve(v)
                .ok_or_else(|| super::Error::UnknownReference { value: v.clone() }),
            Boolean(v) => Ok((*v).into()),
            Number(v) => Ok((*v).into()),
            String(v) => Ok(v.as_str().into()),
            Command(v) => v.evaluate(self, alloc, ctx, env),
            LambdaDef(_) => Err(val::Error::Operation("cannot evaluate lambda").into()),
            LambdaRef(v) => Ok((*v).into()),
        }
    }

    /// Evaluates a lambda.
    ///
    /// # Arguments
    /// *  `alloc` - The allocator to use.
    /// *  `ctx` - The evaluation context.
    /// *  `lambda_ref` - A reference to the lambda to evaluate.
    /// *  `arguments` - The arguments to pass.
    fn invoke<'a, A>(
        &'a self,
        alloc: &A,
        ctx: &C::Context,
        lambda_ref: lambda::Ref,
        arguments: &[Value<'a, C::Tag>],
    ) -> Option<super::Result<'a, C>>
    where
        A: alloc::Allocator<'a, Item = Cons<'a, Value<'a, C::Tag>>> + 'a,
        <C as Command>::Tag: 'a,
    {
        if let Some(lambda) = self.lambdas.resolve(lambda_ref) {
            Some(lambda.invoke(self, alloc, ctx, arguments))
        } else {
            None
        }
    }

    /// Calls a function for this and each sub-expression.
    ///
    /// # Argument
    /// *  `f` - The callback.
    pub fn for_each<F>(&self, f: F)
    where
        F: FnMut(&super::Expression<C>),
    {
        self.root.for_each(f);
    }
}

impl<C> Default for Script<C> {
    fn default() -> Self {
        Self {
            root: Default::default(),
            lambdas: Default::default(),
        }
    }
}

impl<C> ::std::fmt::Display for Script<C>
where
    C: ::std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.root.fmt(f)
    }
}

impl<C> From<super::Expression<C>> for Script<C>
where
    C: Command,
{
    fn from(mut value: super::Expression<C>) -> Self {
        // Replace all lambdas with lambda references
        let mut lambdas = lambda::Store::default();
        value.for_each_mut(|e| match e {
            super::Expression::LambdaDef(vs) if vs.len() == 1 => {
                *e = super::Expression::LambdaRef(lambdas.register(vs.pop().expect("lambda")))
            }
            _ => {}
        });

        Self {
            root: value,
            lambdas,
        }
    }
}
