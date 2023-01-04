use std::io::{BufRead, BufReader, Read};

pub fn lex<R: BufRead>(src: R) -> impl Iterator<Item = Token> {
    let reader = BufReader::new(src);
    reader
        .bytes()
        .map(Result::unwrap)
        .filter_map(|c| char::from_u32(c as u32))
        .filter_map(|c| match c {
            '+' => Some(Token::Plus),
            '-' => Some(Token::Minus),
            '>' => Some(Token::Next),
            '<' => Some(Token::Previous),
            '.' => Some(Token::Dot),
            ',' => Some(Token::Comma),
            '[' => Some(Token::Open),
            ']' => Some(Token::Close),
            _ => None,
        })
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Plus,
    Minus,
    Next,
    Previous,
    Dot,
    Comma,
    Open,
    Close,
}
