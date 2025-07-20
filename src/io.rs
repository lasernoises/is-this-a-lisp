use std::rc::Rc;

use crate::{Fn, Value};

#[derive(Debug)]
pub enum Io {
    ReadLine(Fn),
    PrintLine(Rc<String>, Rc<Io>),
    Done(Value),
}

impl Io {
    pub fn execute(&self) -> Value {
        match self {
            Io::ReadLine(function) => {
                let mut buf = String::new();
                if let Err(_) = std::io::stdin().read_line(&mut buf) {
                    return Value::Error;
                }

                let Value::Io(next) = function.call([Value::String(Rc::new(buf))].into_iter())
                else {
                    return Value::Error;
                };

                next.execute()
            }
            Io::PrintLine(line, io) => {
                println!("{line}");
                io.execute()
            }
            Io::Done(value) => value.clone(),
        }
    }

    pub fn bind(&self, f: &Fn) -> Option<Rc<Io>> {
        match self {
            Io::ReadLine(function) => Some(Rc::new(Io::ReadLine(Fn::Bind(Rc::new((
                f.clone(),
                function.clone(),
            )))))),
            Io::PrintLine(line, io) => Some(Rc::new(Io::PrintLine(line.clone(), io.bind(f)?))),
            Io::Done(value) => {
                let Value::Io(next) = f.call([value.clone()].into_iter()) else {
                    return None;
                };

                Some(next)
            }
        }
    }

    pub fn then(&self, other: Rc<Io>) -> Rc<Io> {
        match self {
            Io::ReadLine(f) => Rc::new(Io::ReadLine(Fn::Then(Rc::new((f.clone(), other))))),
            Io::PrintLine(line, io) => Rc::new(Io::PrintLine(line.clone(), io.then(other))),
            Io::Done(_) => other,
        }
    }
}
