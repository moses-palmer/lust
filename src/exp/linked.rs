use crate::{
    Command, Cons, Context, Environment, Expression, Value, Values, alloc, ast, lambda, val,
};

/// The context passed when evaluating commands.
pub struct EvaluationContext<'a, 'b, 'c, A, C>
where
    A: alloc::Allocator<'a, Item = Cons<'a, Value<'a, C::Tag>>> + 'a,
    C: Command,
    'a: 'c,
    'b: 'c,
{
    /// The current script.
    pub script: &'a Script<C>,

    /// The current allocator.
    pub alloc: &'c A,

    /// The current script context.
    pub ctx: &'c C::Context,

    /// The current environment.
    pub env: &'c crate::Environment<'a, 'b, C>,
}

impl<'a, 'b, 'c, A, C> EvaluationContext<'a, 'b, 'c, A, C>
where
    A: alloc::Allocator<'a, Item = crate::Cons<'a, Value<'a, C::Tag>>> + 'a,
    C: Command,
    'a: 'c,
    'b: 'c,
{
    /// Evaluates an expression.
    ///
    /// # Arguments
    /// *  `e` - The expression to evaluate.
    pub fn value(&self, e: &'a Expression<C>) -> super::Result<'a, C> {
        self.script.value(e, self.alloc, self.ctx, self.env)
    }

    /// Allocates a _cons_.
    ///
    /// # Argument
    /// *  `value` - The value to allocate.
    pub fn alloc(&self, value: A::Item) -> Result<&'a A::Item, alloc::Error> {
        self.alloc.alloc(value)
    }
}

/// A linked expression.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
    pub fn evaluate<'a, 'b, A>(
        &'a self,
        alloc: &A,
        ctx: &C::Context,
    ) -> Result<val::owned::Value<C::Tag>, super::Error<'a>>
    where
        A: alloc::Allocator<'a, Item = Cons<'a, Value<'a, C::Tag>>> + 'a,
        <C as Command>::Tag: 'a,
    {
        self.value(&self.root, alloc, ctx, &Environment::empty())
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
        ctx.on_evaluate()?;

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
                .resolve(*v)
                .ok_or_else(|| super::Error::UnknownReference {
                    value: format!("#{v}"),
                }),
            Boolean(v) => Ok((*v).into()),
            Number(v) => Ok((*v).into()),
            String(v) => Ok(v.as_str().into()),
            Command(v) => {
                ctx.on_evaluate()?;
                v.evaluate(self, alloc, ctx, env)
            }
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

impl<C> ::std::str::FromStr for Script<C>
where
    C: Command + ::std::fmt::Display,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut context = super::ParseContext::default();
        Ok(Expression::parse(
            &mut context,
            &ast::parse(&mut ast::tokenize(s)).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?
        .link())
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

#[cfg(feature = "serde")]
impl<'de, C> serde::Deserialize<'de> for Script<C>
where
    C: Command,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "lowercase")]
        enum Field {
            Root,
            Lambdas,
        }
        struct ScriptVisitor<C>(::std::marker::PhantomData<C>);

        impl<'de, C> serde::de::Visitor<'de> for ScriptVisitor<C>
        where
            C: Command,
        {
            type Value = Script<C>;

            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                formatter.write_str("String")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(v.parse().map_err(|e| E::custom(e))?)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                println!("Deserializing Script from map");

                let mut root = None;
                let mut lambdas = None;
                while let Some(key) = map.next_key::<Field>()? {
                    match key {
                        Field::Root => {
                            if root.replace(map.next_value()?).is_some() {
                                return Err(serde::de::Error::duplicate_field("root"));
                            }
                        }
                        Field::Lambdas => {
                            if lambdas.replace(map.next_value()?).is_some() {
                                return Err(serde::de::Error::duplicate_field("lambdas"));
                            }
                        }
                    }
                }

                match (root, lambdas) {
                    (Some(root), Some(lambdas)) => Ok(Script { root, lambdas }),
                    (Some(root), None) => Ok(Script {
                        root,
                        lambdas: lambda::Store::default(),
                    }),
                    (None, _) => Err(serde::de::Error::missing_field("root")),
                }
            }
        }

        deserializer.deserialize_any(ScriptVisitor(Default::default()))
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::*;

    use crate::test_helpers::{Context, Tag};

    commands_all! {
        #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
        pub enum Command<
            Tag = Tag,
            Context = Context,
        > {
            "debug" => Debug(_ctx, _a) {
                Ok(().into())
            }
        }
    }

    #[test]
    fn parse_valid() {
        // Arrange
        let data = r#"(debug 42)"#;
        let expected = Script::<Command> {
            root: Expression::Command(Command::Debug(vec![Expression::Number(42.0)])),
            lambdas: lambda::Store::default(),
        };

        // Act
        let actual = data.parse::<Script<Command>>().unwrap();

        // Assert
        assert_eq!(actual.root, expected.root);
    }

    #[test]
    fn parse_invalid() {
        // Arrange
        let data = r#"(debug 42"#;
        let expected = "unexpected end of tokens";

        // Act
        let actual = data.parse::<Script<Command>>().unwrap_err();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn deserialize_str() {
        // Arrange
        let data = r#""(debug 42)""#;
        let expected = Script::<Command> {
            root: Expression::Command(Command::Debug(vec![Expression::Number(42.0)])),
            lambdas: lambda::Store::default(),
        };

        // Act
        let actual = serde_json::from_str::<Script<Command>>(data).unwrap();

        // Assert
        assert_eq!(actual.root, expected.root);
    }

    #[test]
    fn deserialize_struct() {
        // Arrange
        let data = r#"{
            "root": {
                "Command": {
                    "Debug": [
                        {
                            "Number": 42.0
                        }
                    ]
                }
            },
            "lambdas": {
                "data": []
            }
        }"#;
        let expected = Script::<Command> {
            root: Expression::Command(Command::Debug(vec![Expression::Number(42.0)])),
            lambdas: lambda::Store::default(),
        };

        // Act
        let actual = serde_json::from_str::<Script<Command>>(data).unwrap();

        // Assert
        assert_eq!(actual.root, expected.root);
    }
}
