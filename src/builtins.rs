use std::{collections::HashMap, rc::Rc};

use crate::{Fn, Scope, Value, eval, eval_block};

#[derive(Copy, Clone, Debug)]
pub enum Builtin {
    Add,
    Sub,
    Mul,
    Div,
    Block,
    Fn,
}

impl Builtin {
    /// A bit hacky to return a Value here instead of an Option<Self>, but this way we can return a
    /// reference from [Scope::resolve].
    pub fn resolve(name: &str) -> &'static Value {
        match name {
            "+" => &Value::Builtin(Self::Add),
            "-" => &Value::Builtin(Self::Sub),
            "*" => &Value::Builtin(Self::Mul),
            "/" => &Value::Builtin(Self::Div),
            "block" => &Value::Builtin(Self::Block),
            "fn" => &Value::Builtin(Self::Fn),
            _ => &Value::Error,
        }
    }

    pub fn call(self, scope: &Rc<Scope>, params: &[Value]) -> Value {
        match (self, params) {
            (Builtin::Add | Builtin::Sub | Builtin::Mul | Builtin::Div, [a, b]) => {
                let a = eval(scope, a);
                let b = eval(scope, b);

                if let (Value::Number(a), Value::Number(b)) = (a, b) {
                    Value::Number(match self {
                        Builtin::Add => a + b,
                        Builtin::Sub => a - b,
                        Builtin::Mul => a * b,
                        Builtin::Div => a / b,
                        _ => unreachable!(),
                    })
                } else {
                    Value::Error
                }
            }
            (Builtin::Block, content) => eval_block(scope.clone(), content),
            (Builtin::Fn, content) => {
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
            _ => Value::Error,
        }
    }
}
