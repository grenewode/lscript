use std::collections::BTreeSet;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Copy, Clone)]
pub struct Ref(usize);

impl Display for Ref {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "&0x{:x}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct LinkedLambda {
    context: Vec<Ref>,
    body: Expr<Ref, Self>,
}

impl Display for LinkedLambda {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "[{}] => {}",
            self.context
                .iter()
                .map(|r| r.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            self.body
        )
    }
}

impl LinkedLambda {
    pub fn call(&self, outer_stack: &[Value<Self>], argument: Value<Self>) -> Value<Self> {
        let mut stack: Vec<_> = self
            .context
            .iter()
            .map(|r| outer_stack[r.0].clone())
            .collect();
        stack.push(argument);
        dbg!(&stack);

        self.body.eval_impl(&stack)
    }
}

#[derive(Debug, Clone)]
pub struct UnlinkedLambda {
    argument: String,
    body: Expr<String, Self>,
}

impl Display for UnlinkedLambda {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} => {}", self.argument, self.body)
    }
}

impl UnlinkedLambda {
    pub fn new(argument: impl Into<String>, body: impl Into<Expr<String, Self>>) -> Self {
        UnlinkedLambda {
            argument: argument.into(),
            body: body.into(),
        }
    }
}

impl UnlinkedLambda {
    fn get_unbound_idents<'s>(&'s self, idents: &mut BTreeSet<&'s str>) {
        self.body.get_unbound_idents(idents);
        idents.remove(self.argument.as_str());
    }

    fn link(&self, outer_stack: &[String]) -> LinkedLambda {
        let mut inner_stack = Vec::new();
        let mut context = Vec::new();
        let mut idents_to_bind = BTreeSet::new();
        self.get_unbound_idents(&mut idents_to_bind);

        // Link items in the body to the outer context
        for (depth, value) in outer_stack.iter().rev().enumerate() {
            if idents_to_bind.is_empty() {
                break;
            }

            if idents_to_bind.remove(value.as_str()) {
                inner_stack.push(value.clone());
                context.push(Ref(depth));
            }
        }

        dbg!(&outer_stack);
        dbg!(&idents_to_bind);
        assert!(idents_to_bind.is_empty());

        inner_stack.push(self.argument.clone());

        LinkedLambda {
            context,
            body: self.body.link_impl(&inner_stack),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value<L> {
    String(String),
    Lambda(Box<L>),
    Tuple(Vec<Value<L>>),
}

impl<L> Display for Value<L>
where
    L: Display,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::String(string) => write!(f, "\"{}\"", string),
            Value::Lambda(lambda) => lambda.fmt(f),
            Value::Tuple(fields) => write!(
                f,
                "{{ {} }}",
                fields
                    .iter()
                    .map(|field| field.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

impl<L> Value<L> {
    pub fn lambda(self) -> Option<L> {
        if let Value::Lambda(lambda) = self {
            Some(*lambda)
        } else {
            None
        }
    }
}

impl<L> Default for Value<L> {
    fn default() -> Self {
        Value::Tuple(vec![])
    }
}

impl Value<UnlinkedLambda> {
    fn get_unbound_idents<'s>(&'s self, idents: &mut BTreeSet<&'s str>) {
        match self {
            Value::Lambda(lambda) => lambda.get_unbound_idents(idents),
            Value::Tuple(fields) => {
                for field in fields {
                    field.get_unbound_idents(idents);
                }
            }
            _ => {}
        }
    }

    fn link(&self, stack: &[String]) -> Value<LinkedLambda> {
        match self {
            Value::Lambda(lambda) => Value::Lambda(lambda.link(stack).into()),
            Value::Tuple(fields) => {
                Value::Tuple(fields.iter().map(|field| field.link(stack)).collect())
            }
            Value::String(string) => Value::String(string.clone()),
        }
    }
}

impl<I, L> From<Value<L>> for Expr<I, L> {
    fn from(value: Value<L>) -> Self {
        Expr::Const(value)
    }
}

impl<I> From<UnlinkedLambda> for Expr<I, UnlinkedLambda> {
    fn from(value: UnlinkedLambda) -> Self {
        Expr::Const(Value::Lambda(Box::new(value)))
    }
}

impl<I> From<LinkedLambda> for Expr<I, LinkedLambda> {
    fn from(value: LinkedLambda) -> Self {
        Expr::Const(Value::Lambda(Box::new(value)))
    }
}

impl<L> From<String> for Expr<String, L> {
    fn from(value: String) -> Self {
        Expr::Ident(value)
    }
}
impl<'s, L> From<&'s str> for Expr<String, L> {
    fn from(value: &'s str) -> Self {
        Expr::Ident(value.into())
    }
}
impl<L> From<char> for Expr<String, L> {
    fn from(value: char) -> Self {
        Expr::Ident(value.to_string())
    }
}

impl<L> From<Ref> for Expr<Ref, L> {
    fn from(value: Ref) -> Self {
        Expr::Ident(value)
    }
}

#[derive(Debug, Clone)]
pub enum Expr<I, L> {
    Const(Value<L>),
    Ident(I),
    Call {
        target: Box<Self>,
        argument: Box<Self>,
    },
    Tuple(Vec<Self>),
}

impl<I, L> Display for Expr<I, L>
where
    I: Display,
    L: Display,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Expr::Const(value) => value.fmt(f),
            Expr::Ident(ident) => ident.fmt(f),
            Expr::Call { target, argument } => write!(f, "({}) {}", target, argument),
            Expr::Tuple(fields) => write!(
                f,
                "{{ {} }}",
                fields
                    .iter()
                    .map(|field| field.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

impl<I, L> Expr<I, L> {
    pub fn call(target: impl Into<Self>, argument: impl Into<Self>) -> Self {
        Expr::Call {
            target: Box::new(target.into()),
            argument: Box::new(argument.into()),
        }
    }
}

impl Expr<String, UnlinkedLambda> {
    fn get_unbound_idents<'s>(&'s self, idents: &mut BTreeSet<&'s str>) {
        match self {
            Expr::Const(value) => value.get_unbound_idents(idents),
            Expr::Ident(name) => {
                idents.insert(name.as_str());
            }
            Expr::Call { target, argument } => {
                target.get_unbound_idents(idents);
                argument.get_unbound_idents(idents);
            }
            Expr::Tuple(fields) => {
                for field in fields {
                    field.get_unbound_idents(idents);
                }
            }
        }
    }

    fn link_impl(&self, stack: &[String]) -> Expr<Ref, LinkedLambda> {
        match self {
            Expr::Const(value) => Expr::Const(value.link(stack)),
            Expr::Ident(ident) => {
                Expr::Ident(Ref(stack.iter().rposition(|item| item == ident).unwrap()))
            }
            Expr::Call { target, argument } => Expr::Call {
                target: target.link_impl(stack).into(),
                argument: argument.link_impl(stack).into(),
            },
            Expr::Tuple(fields) => {
                Expr::Tuple(fields.iter().map(|field| field.link_impl(stack)).collect())
            }
        }
    }

    pub fn link(&self) -> Expr<Ref, LinkedLambda> {
        self.link_impl(&[])
    }
}

impl Expr<Ref, LinkedLambda> {
    fn eval_impl(&self, stack: &[Value<LinkedLambda>]) -> Value<LinkedLambda> {
        match self {
            Expr::Const(value) => value.clone(),
            Expr::Ident(reference) => stack[reference.0].clone(),
            Expr::Call { target, argument } => {
                let lambda = target.eval_impl(stack).lambda().unwrap();
                lambda.call(stack, argument.eval_impl(stack))
            }
            Expr::Tuple(fields) => {
                Value::Tuple(fields.iter().map(|field| field.eval_impl(stack)).collect())
            }
        }
    }

    pub fn eval(&self) -> Value<LinkedLambda> {
        self.eval_impl(&[])
    }
}

macro_rules! expr {
    ($var:ident $($vars:ident)* => $body:tt) => {
        Expr::<String, _>::from(
            UnlinkedLambda::new(
                stringify!($var).to_string(),
                expr!($($vars)* $body)
            )
        )
    };
    ($var:ident) => {
       Expr::<String, _>::from(stringify!($var).to_string())
    };
    ({ $($field:tt),* }) => {
        Expr::Tuple(vec![ $(expr!($field)),* ])
    };
    ({ $($field:tt,)+ }) => {
        Expr::Tuple(vec![ $(expr!($field)),* ])
    };
    ( ($($item:tt)*) ) => {
        expr!($($item)*)
    };
    ($target:tt $arg:tt $($args:tt)*) => {
        Expr::call(expr!($target), expr!($arg $($args)*))
    };
    ($e:expr) => {Value::from($e)};
}
