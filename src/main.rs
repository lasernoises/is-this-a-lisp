use std::{collections::HashMap, rc::Rc};

use parser::parse;

mod parser;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Number(f64),
    String(Rc<String>),
    Symbol(&'static str), // TODO: interning
    List(Rc<Vec<Value>>),
    Error,
    BuiltinAdd,
    BuiltinSub,
    BuiltinMul,
    BuiltinDiv,
}

fn main() {
    let code = include_str!("./code.lisp?");
    let ast = parse(code);

    dbg!(eval_program(&ast));
}

#[derive(Clone)]
struct Scope {
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
            match name {
                "+" => &Value::BuiltinAdd,
                "-" => &Value::BuiltinSub,
                "*" => &Value::BuiltinMul,
                "/" => &Value::BuiltinDiv,
                _ => &Value::Error,
            }
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
    match (callable, params) {
        (Value::BuiltinAdd | Value::BuiltinSub | Value::BuiltinMul | Value::BuiltinDiv, [a, b]) => {
            let a = eval(scope, a);
            let b = eval(scope, b);

            if let (Value::Number(a), Value::Number(b)) = (a, b) {
                Value::Number(match callable {
                    Value::BuiltinAdd => a + b,
                    Value::BuiltinSub => a - b,
                    Value::BuiltinMul => a * b,
                    Value::BuiltinDiv => a / b,
                    _ => unreachable!(),
                })
            } else {
                Value::Error
            }
        }
        _ => Value::Error,
    }
}
