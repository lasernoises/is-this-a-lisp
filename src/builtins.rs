use std::{collections::HashMap, rc::Rc};

use crate::{Fn, Scope, UserFn, Value, eval_block};

pub fn resolve(name: &str) -> &'static Value {
    match name {
        "+" => &Value::Fn(Fn::Builtin(BuiltinFn::Add)),
        "-" => &Value::Fn(Fn::Builtin(BuiltinFn::Sub)),
        "*" => &Value::Fn(Fn::Builtin(BuiltinFn::Mul)),
        "/" => &Value::Fn(Fn::Builtin(BuiltinFn::Div)),

        "block" => &Value::Macro(BuiltinMacro::Block),
        "fn" => &Value::Macro(BuiltinMacro::Fn),
        _ => &Value::Error,
    }
}

#[derive(Copy, Clone, Debug)]
pub enum BuiltinFn {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Copy, Clone, Debug)]
pub enum BuiltinMacro {
    Block,
    Fn,
}

impl BuiltinFn {
    pub fn call(self, mut params: impl ExactSizeIterator<Item = Value>) -> Value {
        if params.len() != 2 {
            return Value::Error;
        }

        let a = params.next().unwrap();
        let b = params.next().unwrap();

        if let (Value::Number(a), Value::Number(b)) = (a, b) {
            Value::Number(match self {
                BuiltinFn::Add => a + b,
                BuiltinFn::Sub => a - b,
                BuiltinFn::Mul => a * b,
                BuiltinFn::Div => a / b,
            })
        } else {
            Value::Error
        }
    }
}

impl BuiltinMacro {
    pub fn call(self, scope: &Rc<Scope>, content: &[Value]) -> Value {
        match self {
            BuiltinMacro::Block => eval_block(scope.clone(), content),
            BuiltinMacro::Fn => {
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

                Value::Fn(Fn::User(Rc::new(UserFn {
                    scope: Rc::new(Scope {
                        parent: Some(scope.clone()),
                        variables: params_map,
                    }),
                    params: params.clone(),
                    content: content[1..].to_vec(),
                })))
            }
        }
    }
}
