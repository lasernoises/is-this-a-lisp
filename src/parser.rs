use std::{iter::Peekable, str::Chars};

use crate::ast::Node;

#[derive(PartialEq, Debug)]
enum Token {
    Open,
    Close,
    Number(f64),
    Symbol(String),
    String(String),
    UnterminatedString,
    Unknown(char),
}

struct Scanner<'a> {
    current_position: usize,
    it: Peekable<Chars<'a>>,
}

impl<'a> Scanner<'a> {
    fn new(buf: &str) -> Scanner {
        Scanner {
            current_position: 0,
            it: buf.chars().peekable(),
        }
    }

    fn next(&mut self) -> Option<char> {
        let next = self.it.next();
        if let Some(c) = next {
            self.current_position += c.len_utf8();
        }
        next
    }

    fn peek(&mut self) -> Option<&char> {
        self.it.peek()
    }

    // Consume next char if the next one after matches (so .3 eats . if 3 is numeric, for example)
    fn consume_if_next<F>(&mut self, x: F) -> bool
    where
        F: Fn(char) -> bool,
    {
        let mut it = self.it.clone();
        match it.next() {
            None => return false,
            _ => (),
        }

        if let Some(&ch) = it.peek() {
            if x(ch) {
                self.next().unwrap();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn consume_while<F>(&mut self, x: F) -> Vec<char>
    where
        F: Fn(char) -> bool,
    {
        let mut chars: Vec<char> = Vec::new();
        while let Some(&ch) = self.peek() {
            if x(ch) {
                self.next().unwrap();
                chars.push(ch);
            } else {
                break;
            }
        }
        chars
    }
}

struct Lexer<'a> {
    scanner: Scanner<'a>,
}

impl<'a> Lexer<'a> {
    fn new(buf: &str) -> Lexer {
        Lexer {
            scanner: Scanner::new(buf),
        }
    }

    fn match_token(&mut self, ch: char) -> Option<Token> {
        match ch {
            ' ' => None,
            '\n' => None,
            '\t' => None,
            '\r' => None,
            '[' => Some(Token::Open),
            ']' => Some(Token::Close),
            x if x.is_numeric() => self.number(x),
            x if x.is_ascii_alphabetic() || ['$', '-', '+', '*', '_'].contains(&x) => {
                self.symbol(x)
            }
            '"' => {
                let content: String = self
                    .scanner
                    .consume_while(|x| x != '"')
                    .into_iter()
                    .collect();

                if let Some(next) = self.scanner.next() {
                    assert!(next == '"');

                    Some(Token::String(content))
                } else {
                    Some(Token::UnterminatedString)
                }
            }
            c => Some(Token::Unknown(c)),
        }
    }

    fn number(&mut self, x: char) -> Option<Token> {
        let mut number = String::new();
        number.push(x);
        let num: String = self
            .scanner
            .consume_while(|a| a.is_numeric())
            .into_iter()
            .collect();
        number.push_str(num.as_str());
        if self.scanner.peek() == Some(&'.') && self.scanner.consume_if_next(|ch| ch.is_numeric()) {
            let num2: String = self
                .scanner
                .consume_while(|a| a.is_numeric())
                .into_iter()
                .collect();
            number.push('.');
            number.push_str(num2.as_str());
        }
        Some(Token::Number(number.parse::<f64>().unwrap()))
    }

    fn symbol(&mut self, first: char) -> Option<Token> {
        let mut identifier: String = first.into();
        let rest: String = self
            .scanner
            .consume_while(|a| a.is_ascii_alphanumeric() || ['$', '-', '+', '*', '_'].contains(&a))
            .into_iter()
            .collect();
        identifier.push_str(rest.as_str());
        Some(Token::Symbol(identifier))
    }
}

fn tokenize(buf: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(buf);

    let mut tokens: Vec<Token> = Vec::new();
    loop {
        let ch = match lexer.scanner.next() {
            None => break,
            Some(c) => c,
        };
        if let Some(token) = lexer.match_token(ch) {
            tokens.push(token);
        }
    }
    tokens
}

struct Parser<'a> {
    tokens: &'a [Token],
    cursor: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, cursor: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.cursor)
    }

    fn advance(&mut self) -> Option<&Token> {
        let cursor = self.cursor;
        self.cursor += 1;
        self.tokens.get(cursor)
    }
}

fn parse_node(parser: &mut Parser) -> Option<Node> {
    match parser.peek() {
        Some(Token::Open) => parse_list(parser),
        Some(&Token::Number(n)) => {
            parser.advance();
            Some(Node::Number(n))
        }
        Some(Token::String(s)) => {
            let s = s.clone();
            parser.advance();
            Some(Node::String(s))
        }
        Some(Token::Symbol(s)) => {
            let s = s.clone();
            parser.advance();
            Some(Node::Symbol(s))
        }
        _ => None,
    }
}

fn parse_list(parser: &mut Parser) -> Option<Node> {
    parser.advance();

    let mut content = Vec::new();

    while parser.peek().is_some_and(|t| t != &Token::Close) {
        content.push(parse_node(parser)?);
    }

    if parser.peek().is_some() {
        parser.advance();
        Some(Node::List(content))
    } else {
        None
    }
}

pub fn parse(buf: &str) -> Option<Node> {
    let tokens = tokenize(buf);

    dbg!(&tokens);

    let mut parser = Parser::new(&tokens);
    let node = parse_node(&mut parser);

    if parser.advance().is_some() {
        None
    } else {
        node
    }
}
