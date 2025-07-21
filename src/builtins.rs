use std::rc::Rc;

use crate::{Function, Scope, UserFn, Value, eval_block, io::Io};

pub fn resolve(name: &str) -> &'static Value {
    match name {
        "+" => &Value::Fn(Function::Builtin(BuiltinFn::Add)),
        "-" => &Value::Fn(Function::Builtin(BuiltinFn::Sub)),
        "*" => &Value::Fn(Function::Builtin(BuiltinFn::Mul)),
        "/" => &Value::Fn(Function::Builtin(BuiltinFn::Div)),

        "then" => &Value::Fn(Function::Builtin(BuiltinFn::Then)),
        "bind" => &Value::Fn(Function::Builtin(BuiltinFn::Bind)),
        "return" => &Value::Fn(Function::Builtin(BuiltinFn::Return)),

        "read_line" => &Value::Fn(Function::Builtin(BuiltinFn::ReadLine)),
        "print_line" => &Value::Fn(Function::Builtin(BuiltinFn::PrintLine)),

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

    Then,
    Bind,
    Return,

    ReadLine,
    PrintLine,
}

#[derive(Copy, Clone, Debug)]
pub enum BuiltinMacro {
    Block,
    Fn,
}

impl BuiltinFn {
    pub fn call(self, mut params: impl ExactSizeIterator<Item = Value>) -> Value {
        match self {
            BuiltinFn::Add | BuiltinFn::Sub | BuiltinFn::Mul | BuiltinFn::Div => {
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
                        _ => unreachable!(),
                    })
                } else {
                    Value::Error
                }
            }
            BuiltinFn::Then => {
                let (Some(Value::Io(a)), Some(Value::Io(b)), None) =
                    (params.next(), params.next(), params.next())
                else {
                    return Value::Error;
                };

                Value::Io(a.then(b))
            }
            BuiltinFn::Bind => {
                let (Some(Value::Io(a)), Some(Value::Fn(b)), None) =
                    (params.next(), params.next(), params.next())
                else {
                    return Value::Error;
                };

                a.bind(&b).map(Value::Io).unwrap_or(Value::Error)
            }
            BuiltinFn::Return => {
                if params.len() != 1 {
                    return Value::Error;
                }

                Value::Io(Rc::new(Io::Done(params.next().unwrap())))
            }
            BuiltinFn::ReadLine => {
                if params.len() != 0 {
                    return Value::Error;
                }

                Value::Io(Rc::new(Io::ReadLine(Function::Builtin(BuiltinFn::Return))))
            }
            BuiltinFn::PrintLine => {
                let Some(Value::String(line)) = params.next() else {
                    return Value::Error;
                };

                Value::Io(Rc::new(Io::PrintLine(line, Rc::new(Io::Done(Value::Nil)))))
            }
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

                for (i, param) in params.iter().enumerate() {
                    if let &Value::Symbol(name) = param {
                        // Duplicate parameter names are not allowed.
                        if params[..i].iter().any(|other| {
                            if let &Value::Symbol(other) = other {
                                other == name
                            } else {
                                unreachable!()
                            }
                        }) {
                            return Value::Error;
                        }
                    } else {
                        return Value::Error;
                    }
                }

                Value::Fn(Function::User(Rc::new(UserFn {
                    scope: scope.clone(),
                    params: params.clone(),
                    content: content[1..].to_vec(),
                })))
            }
        }
    }
}
