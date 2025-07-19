#[derive(Debug)]
pub enum Node {
    Number(f64),
    String(String),
    Symbol(String),
    List(Vec<Node>),
}
