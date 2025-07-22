use std::rc::Rc;

use crate::{BadProgram, Function, Result, Value};

#[derive(Debug)]
pub enum Io {
    ReadLine(Function),
    PrintLine(Rc<String>, Rc<Io>),
    Done(Value),
}

impl Io {
    pub fn execute(&self) -> Result<Value> {
        match self {
            Io::ReadLine(function) => {
                let mut buf = String::new();
                std::io::stdin()
                    .read_line(&mut buf)
                    .map_err(|_| BadProgram)?;

                let Value::Io(next) =
                    function.call([Ok(Value::String(Rc::new(buf)))].into_iter())?
                else {
                    return Err(BadProgram);
                };

                next.execute()
            }
            Io::PrintLine(line, io) => {
                println!("{line}");
                io.execute()
            }
            Io::Done(value) => Ok(value.clone()),
        }
    }

    pub fn bind(&self, f: &Function) -> Result<Rc<Io>> {
        match self {
            Io::ReadLine(function) => Ok(Rc::new(Io::ReadLine(Function::Fn(Rc::new({
                let f = f.clone();
                let function = function.clone();
                move |params| {
                    let (Some(val), None) = (params.next(), params.next()) else {
                        return Err(BadProgram);
                    };

                    let Value::Io(io) = f.call([val].into_iter())? else {
                        return Err(BadProgram);
                    };

                    io.bind(&function).map(Value::Io)
                }
            }))))),
            Io::PrintLine(line, io) => Ok(Rc::new(Io::PrintLine(line.clone(), io.bind(f)?))),
            Io::Done(value) => {
                let Value::Io(next) = f.call([Ok(value.clone())].into_iter())? else {
                    return Err(BadProgram);
                };

                Ok(next)
            }
        }
    }

    pub fn then(&self, other: Rc<Io>) -> Rc<Io> {
        match self {
            Io::ReadLine(f) => Rc::new(Io::ReadLine(Function::Fn(Rc::new({
                let f = f.clone();
                let other = other.clone();
                move |params| {
                    let (Some(val), None) = (params.next(), params.next()) else {
                        return Err(BadProgram);
                    };

                    let Value::Io(io) = f.call([val].into_iter())? else {
                        return Err(BadProgram);
                    };

                    Ok(Value::Io(io.then(other.clone())))
                }
            })))),
            Io::PrintLine(line, io) => Rc::new(Io::PrintLine(line.clone(), io.then(other))),
            Io::Done(_) => other,
        }
    }
}
