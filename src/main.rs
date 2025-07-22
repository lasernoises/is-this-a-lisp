use std::{path::PathBuf, rc::Rc};

use builtins::{BuiltinFn, BuiltinMacro};
use clap::Parser;
use io::Io;
use parser::parse;

mod builtins;
mod io;
mod parser;

#[derive(Parser)]
struct Cli {
    path: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    let code = match std::fs::read_to_string(cli.path) {
        Ok(code) => code,
        Err(e) => {
            println!("{e}");
            return;
        }
    };

    let result = dbg!(parse(&code).and_then(|ast| eval_program(&ast)));

    if let Ok(Value::Io(io)) = result {
        dbg!(io.execute()).ok();
    }
}

#[derive(Debug)]
pub struct BadProgram;

pub type Result<T> = std::result::Result<T, BadProgram>;

#[derive(Debug)]
pub struct UserFn {
    scope: Rc<Scope>,
    params: Rc<Vec<Value>>,
    content: Vec<Value>,
}

impl UserFn {
    pub fn call(&self, params: impl ExactSizeIterator<Item = Result<Value>>) -> Result<Value> {
        if self.params.len() != params.len() {
            return Err(BadProgram);
        }

        let mut scope = self.scope.clone();

        let mut params_def = self.params.iter();

        for param in params {
            scope = scope.with(
                match params_def.next() {
                    Some(Value::Symbol(name)) => name,
                    // The lenth of params_def gets checked above and at function definition
                    // only symbols are allowed.
                    _ => unreachable!(),
                },
                param?,
            );
        }

        eval_block(scope.clone(), &self.content)
    }
}

#[derive(Clone)]
pub enum Function {
    Builtin(BuiltinFn),
    User(Rc<UserFn>),
    Fn(Rc<dyn Fn(&mut dyn ExactSizeIterator<Item = Result<Value>>) -> Result<Value>>),
}

impl std::fmt::Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Builtin(arg0) => f.debug_tuple("Builtin").field(arg0).finish(),
            Self::User(arg0) => f.debug_tuple("User").field(arg0).finish(),
            Self::Fn(_) => f.debug_tuple("Fn").finish(),
        }
    }
}

impl Function {
    pub fn call(&self, mut params: impl ExactSizeIterator<Item = Result<Value>>) -> Result<Value> {
        match self {
            Function::Builtin(builtin_fn) => builtin_fn.call(params),
            Function::User(user_fn) => user_fn.call(params),
            Function::Fn(f) => f(&mut params),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Number(f64),
    String(Rc<String>),
    Symbol(&'static str), // TODO: interning
    List(Rc<Vec<Value>>),
    Fn(Function),
    Macro(BuiltinMacro),
    Io(Rc<Io>),
    Nil,
}

// This used contain a hash-map such that each level could have multiple keys. The advantage of that
// would likely have been in large blocks with lots of ifs, but then the map would need to be cloned
// when the scope gets captured by an if in the middle. I don't expect blocks to contain that many
// values usually, so this might actually be better because we can avoid the overhead of a hash-map,
// but that's pure speculation.
//
// And of course there is lots of potential for optimization. For example there could be a hash-map
// that we keep extending (regardless of whether we're in the same block) until something captures
// it (making the refcount more than 1), at which point we'd start a new level. Or I think Clojure
// has an interesting data-structure for this.
#[derive(Clone, Debug)]
pub enum Scope {
    Empty,
    Value {
        parent: Rc<Scope>,
        name: &'static str,
        value: Value,
    },
}

impl Scope {
    fn resolve(&self, name: &str) -> Result<&Value> {
        Ok(match self {
            Scope::Empty => builtins::resolve(name)?,
            Scope::Value {
                name: this_name,
                value,
                ..
            } if *this_name == name => value,
            Scope::Value { parent, .. } => parent.resolve(name)?,
        })
    }

    fn with(self: Rc<Self>, name: &'static str, value: Value) -> Rc<Scope> {
        Rc::new(Scope::Value {
            parent: self,
            name,
            value,
        })
    }
}

fn eval_program(content: &Value) -> Result<Value> {
    let root_scope = Rc::new(Scope::Empty);

    eval(&root_scope, content)
}

fn eval(scope: &Rc<Scope>, input: &Value) -> Result<Value> {
    match input {
        v @ (Value::Number(_) | Value::String(_)) => Ok(v.clone()),
        Value::List(values) => {
            if let [callable, ..] = values.as_slice() {
                let callable = eval(scope, callable)?;
                call(scope, &callable, &values[1..])
            } else {
                Err(BadProgram)
            }
        }
        Value::Symbol(name) => scope.resolve(name).map(Clone::clone),
        _ => Err(BadProgram),
    }
}

fn eval_block(mut scope: Rc<Scope>, content: &[Value]) -> Result<Value> {
    let Some((last, statements)) = content.split_last() else {
        return Err(BadProgram);
    };

    for statement in statements {
        if let &Value::List(ref list) = statement
            && let [Value::Symbol("let"), Value::Symbol(name), expr] = list.as_slice()
        {
            let value = eval(&scope, expr)?;

            scope = scope.with(name, value);
        } else {
            return Err(BadProgram);
        }
    }

    eval(&scope, last)
}

// A purely syntactic transformation would also work here. But what is this? LISP?
fn eval_do_block(scope: &Rc<Scope>, content: &[Value]) -> Result<Rc<Io>> {
    let (first, rest) = content.split_first().ok_or(BadProgram)?;

    match if let Value::List(list) = first {
        Some(list.as_slice())
    } else {
        None
    } {
        Some([Value::Symbol("let"), Value::Symbol(name), expr]) => {
            if rest.len() < 1 {
                return Err(BadProgram);
            }

            let value = eval(scope, expr)?;

            let scope = scope.clone().with(*name, value);

            eval_do_block(&scope, rest)
        }
        Some([Value::Symbol("use"), Value::Symbol(name), expr]) => {
            if rest.len() < 1 {
                return Err(BadProgram);
            }

            let value = eval(scope, expr)?;

            let Value::Io(io) = value else {
                return Err(BadProgram);
            };

            io.bind(&Function::Fn(Rc::new({
                let name = *name;
                let scope = scope.clone();

                // Rather unfortunate copying here. Having an abstraction for reference counted
                // slices where different sub-slices share a reference count for an allocation would
                // be helpful here.
                //
                // ...Or linked lists, I guess. But what is this? LISP?
                let rest = rest.to_vec();
                move |params| {
                    let (Some(value), None) = (params.next(), params.next()) else {
                        return Err(BadProgram);
                    };

                    let scope = scope.clone().with(name, value?);

                    let io = eval_do_block(&scope, &rest)?;

                    Ok(Value::Io(io))
                }
            })))
        }
        _ => {
            let value = eval(scope, first)?;

            let Value::Io(io) = value else {
                return Err(BadProgram);
            };

            if rest.len() >= 1 {
                Ok(io.then(eval_do_block(scope, rest)?))
            } else {
                Ok(io)
            }
        }
    }
}

fn call(scope: &Rc<Scope>, callable: &Value, params: &[Value]) -> Result<Value> {
    match callable {
        Value::Macro(builtin_macro) => builtin_macro.call(scope, params),
        Value::Fn(function) => function.call(params.iter().map(|param| eval(scope, param))),
        _ => Err(BadProgram),
    }
}
