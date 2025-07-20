use std::{collections::HashMap, rc::Rc};

use builtins::Builtin;
use parser::parse;

mod builtins;
mod parser;

#[derive(Debug)]
pub struct Fn {
    scope: Rc<Scope>,
    params: Rc<Vec<Value>>,
    content: Vec<Value>,
}

#[derive(Clone, Debug)]
pub enum Value {
    Number(f64),
    String(Rc<String>),
    Symbol(&'static str), // TODO: interning
    List(Rc<Vec<Value>>),
    Fn(Rc<Fn>),
    Builtin(Builtin),
    Error,
}

fn main() {
    let code = include_str!("./code.lisp?");
    let ast = parse(code);

    dbg!(eval_program(&ast));
}

#[derive(Clone, Debug)]
pub struct Scope {
    parent: Option<Rc<Scope>>,
    variables: HashMap<&'static str, Value>,
}

impl Scope {
    fn resolve(&self, name: &str) -> &Value {
        if let Some(value) = self.variables.get(name) {
            value
        } else if let Some(ref parent) = self.parent {
            parent.resolve(name)
        } else {
            Builtin::resolve(name)
        }
    }
}

fn eval_program(content: &Value) -> Value {
    let root_scope = Rc::new(Scope {
        parent: None,
        variables: HashMap::new(),
    });

    if let Value::List(block_content) = content {
        eval_block(root_scope, block_content)
    } else {
        Value::Error
    }
}

fn eval(scope: &Rc<Scope>, input: &Value) -> Value {
    match input {
        v @ (Value::Number(_) | Value::String(_)) => v.clone(),
        Value::List(values) => {
            if let [callable, ..] = values.as_slice() {
                let callable = eval(scope, callable);
                call(scope, &callable, &values[1..])
            } else {
                Value::Error
            }
        }
        Value::Symbol(name) => scope.resolve(name).clone(),
        _ => Value::Error,
    }
}

fn eval_block(scope: Rc<Scope>, content: &[Value]) -> Value {
    let Some((last, statements)) = content.split_last() else {
        return Value::Error;
    };

    let mut scope = Rc::new(Scope {
        parent: Some(scope),
        variables: HashMap::with_capacity(statements.len()),
    });

    for statement in statements {
        if let &Value::List(ref list) = statement
            && let [Value::Symbol("let"), Value::Symbol(name), expr] = list.as_slice()
        {
            let value = eval(&scope, expr);

            // This clones the scope if it was captured by the expression. Maybe it would would be
            // better to start a new scope that just has the other one as its parent in that case.
            // Or each scope could just be a pair.
            Rc::make_mut(&mut scope).variables.insert(name, value);
        } else {
            return Value::Error;
        }
    }

    eval(&scope, last)
}

fn call(scope: &Rc<Scope>, callable: &Value, params: &[Value]) -> Value {
    match callable {
        Value::Builtin(builtin) => builtin.call(scope, params),
        Value::Fn(function) => {
            if function.params.len() != params.len() {
                return Value::Error;
            }

            let mut fn_scope = function.scope.clone();
            let fn_scope_params = &mut Rc::make_mut(&mut fn_scope).variables;

            let mut params_def = function.params.iter();

            for param in params {
                fn_scope_params
                    .insert(
                        match params_def.next() {
                            Some(Value::Symbol(name)) => name,
                            // The lenth of params_def gets checked above and at function definition
                            // only symbols are allowed.
                            _ => unreachable!(),
                        },
                        eval(scope, param),
                    )
                    // There needs to be a previous value in here from when we define the scope.
                    .unwrap();
            }

            let ret = eval_block(fn_scope.clone(), &function.content);

            for param in Rc::make_mut(&mut fn_scope).variables.values_mut() {
                // We don't want to hang on to those values for too long. For all we know they could
                // be huge.
                *param = Value::Error;
            }

            ret
        }
        _ => Value::Error,
    }
}
