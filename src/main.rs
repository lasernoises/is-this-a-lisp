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
}

fn main() {
    let code = include_str!("./code.lisp?");
    let ast = parse(code);

    dbg!(eval_program(&ast));
}

// struct Scope {
//     variables: HashMap<>
// }

fn eval_program(content: &Value) -> Value {
    if let Value::List(block_content) = content {
        eval_block(block_content)
    } else {
        Value::Error
    }
}

fn eval(input: &Value) -> Value {
    match input {
        v @ (Value::Number(_) | Value::String(_)) => v.clone(),
        Value::List(values) if values.len() >= 1 => todo!(),
        _ => Value::Error,
    }
}

fn eval_block(content: &[Value]) -> Value {
    let Some((last, statements)) = content.split_last() else {
        return Value::Error;
    };

    let mut scope: HashMap<&'static str, Value> = HashMap::with_capacity(statements.len());

    for statement in statements {
        if let &Value::List(ref list) = statement
            && let [Value::Symbol("let"), Value::Symbol(name), expr] = list.as_slice()
        {
            scope.insert(name, eval(expr));
        } else {
            return Value::Error;
        }
    }

    eval(last)
}
