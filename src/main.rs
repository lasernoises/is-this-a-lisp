use std::{collections::HashMap, rc::Rc};

use parser::parse;

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
    BuiltinAdd,
    BuiltinSub,
    BuiltinMul,
    BuiltinDiv,
    BuiltinBlock,
    BuiltinFn,
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
            match name {
                "+" => &Value::BuiltinAdd,
                "-" => &Value::BuiltinSub,
                "*" => &Value::BuiltinMul,
                "/" => &Value::BuiltinDiv,
                "block" => &Value::BuiltinBlock,
                "fn" => &Value::BuiltinFn,
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
        (Value::BuiltinBlock, content) => eval_block(scope.clone(), content),
        (Value::BuiltinFn, content) => {
            if content.len() < 2 {
                return Value::Error;
            }

            let Value::List(ref params) = content[0] else {
                return Value::Error;
            };

            // We just put errors in here for now. The point is to not need to allocate a new
            // hashmap every time we call the function. That only happens if the function gets
            // called in a reentrant way because of Rc::make_mut.
            let mut params_map = HashMap::with_capacity(params.len());

            for param in params.iter() {
                if let &Value::Symbol(name) = param {
                    if params_map.insert(name, Value::Error).is_some() {
                        // No duplicate paramter names.
                        return Value::Error;
                    }
                } else {
                    return Value::Error;
                }
            }

            Value::Fn(Rc::new(Fn {
                scope: Rc::new(Scope {
                    parent: Some(scope.clone()),
                    variables: params_map,
                }),
                params: params.clone(),
                content: content[1..].to_vec(),
            }))
        }
        (Value::Fn(function), params) => {
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
