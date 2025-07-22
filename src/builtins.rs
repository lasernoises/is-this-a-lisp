use std::rc::Rc;

use crate::{
    BadProgram, Function, Result, Scope, UserFn, Value, eval_block, eval_do_block, io::Io,
};

pub fn resolve(name: &str) -> Result<&'static Value> {
    Ok(match name {
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
        "do" => &Value::Macro(BuiltinMacro::Do),
        "fn" => &Value::Macro(BuiltinMacro::Fn),
        _ => return Err(BadProgram),
    })
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
    Do,
}

impl BuiltinFn {
    pub fn call(self, mut params: impl ExactSizeIterator<Item = Result<Value>>) -> Result<Value> {
        match self {
            BuiltinFn::Add | BuiltinFn::Sub | BuiltinFn::Mul | BuiltinFn::Div => {
                if params.len() != 2 {
                    return Err(BadProgram);
                }

                let a = params.next().unwrap()?;
                let b = params.next().unwrap()?;

                if let (Value::Number(a), Value::Number(b)) = (a, b) {
                    Ok(Value::Number(match self {
                        BuiltinFn::Add => a + b,
                        BuiltinFn::Sub => a - b,
                        BuiltinFn::Mul => a * b,
                        BuiltinFn::Div => a / b,
                        _ => unreachable!(),
                    }))
                } else {
                    Err(BadProgram)
                }
            }
            BuiltinFn::Then => {
                let (Some(Ok(Value::Io(a))), Some(Ok(Value::Io(b))), None) =
                    (params.next(), params.next(), params.next())
                else {
                    return Err(BadProgram);
                };

                Ok(Value::Io(a.then(b)))
            }
            BuiltinFn::Bind => {
                let (Some(Ok(Value::Io(a))), Some(Ok(Value::Fn(b))), None) =
                    (params.next(), params.next(), params.next())
                else {
                    return Err(BadProgram);
                };

                a.bind(&b).map(Value::Io)
            }
            BuiltinFn::Return => {
                if params.len() != 1 {
                    return Err(BadProgram);
                }

                Ok(Value::Io(Rc::new(Io::Done(params.next().unwrap()?))))
            }
            BuiltinFn::ReadLine => {
                if params.len() != 0 {
                    return Err(BadProgram);
                }

                Ok(Value::Io(Rc::new(Io::ReadLine(Function::Builtin(
                    BuiltinFn::Return,
                )))))
            }
            BuiltinFn::PrintLine => {
                let Some(Ok(Value::String(line))) = params.next() else {
                    return Err(BadProgram);
                };

                Ok(Value::Io(Rc::new(Io::PrintLine(
                    line,
                    Rc::new(Io::Done(Value::Nil)),
                ))))
            }
        }
    }
}

impl BuiltinMacro {
    pub fn call(self, scope: &Rc<Scope>, content: &[Value]) -> Result<Value> {
        match self {
            BuiltinMacro::Block => eval_block(scope.clone(), content),
            BuiltinMacro::Do => eval_do_block(scope, content).map(Value::Io),
            BuiltinMacro::Fn => {
                if content.len() < 2 {
                    return Err(BadProgram);
                }

                let Value::List(ref params) = content[0] else {
                    return Err(BadProgram);
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
                            return Err(BadProgram);
                        }
                    } else {
                        return Err(BadProgram);
                    }
                }

                Ok(Value::Fn(Function::User(Rc::new(UserFn {
                    scope: scope.clone(),
                    params: params.clone(),
                    content: content[1..].to_vec(),
                }))))
            }
        }
    }
}
