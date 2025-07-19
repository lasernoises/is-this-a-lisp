mod ast;
mod parser;

fn main() {
    dbg!(parser::parse("[]"));
    dbg!(parser::parse("[1 \"abc\"]"));
    dbg!(parser::parse("[1 [\"abc\"]]"));
    dbg!(parser::parse("[the [\"abc\"]]"));
    dbg!(parser::parse("[- 5 8]"));
}
