use super::{
    ast::{Ast, AstNode},
    lexer::Token,
};

pub fn parse(mut src: impl Iterator<Item = Token>) -> Ast {
    let (body, closed) = parse_instructions(&mut src);
    assert!(!closed);

    Ast(body)
}
fn parse_instructions(src: &mut impl Iterator<Item = Token>) -> (Vec<AstNode>, bool) {
    let mut i = Vec::new();
    let mut previous = None;

    loop {
        let (tok, closed) = parse_instruction(src);
        let Some(tok) = tok else {
            if let Some(prev) = previous.take() {
                i.push(prev);
            }
            return (i, closed);
        };

        if let Some(prev) = previous.take() {
            match merge(prev, tok) {
                Merged::No(prev, tok) => {
                    i.push(prev);
                    previous = Some(tok);
                }
                Merged::Yes(tok) => previous = Some(tok),
            }
        } else {
            previous = Some(tok);
        }
    }
}
fn parse_instruction(src: &mut impl Iterator<Item = Token>) -> (Option<AstNode>, bool) {
    let Some(tok) = src.next() else { return (None, false) };

    let i = match tok {
        Token::Plus => AstNode::Modify(1),
        Token::Minus => AstNode::Modify(-1),
        Token::Next => AstNode::Move(1),
        Token::Previous => AstNode::Move(-1),
        Token::Dot => AstNode::Output,
        Token::Comma => AstNode::Input,
        Token::Close => return (None, true),
        Token::Open => {
            let (body, closed) = parse_instructions(src);
            assert!(closed);
            if loop_is_clear(&body) {
                AstNode::Set(0)
            } else {
                AstNode::Loop(body)
            }
        }
    };

    (Some(i), false)
}

fn loop_is_clear(body: &[AstNode]) -> bool {
    if body.len() == 1 {
        match &body[0] {
            AstNode::Modify(a) if *a % 2 != 0 => true,
            _ => false,
        }
    } else {
        false
    }
}
fn merge(left: AstNode, right: AstNode) -> Merged {
    use AstNode::*;
    Merged::Yes(match (left, right) {
        (Modify(a), Modify(b)) => Modify(a.wrapping_add(b)),
        (Move(a), Move(b)) => Move(a.wrapping_add(b)),
        (Set(a), Modify(b)) => Set(a.wrapping_add_signed(b)),
        (left, right) => return Merged::No(left, right),
    })
}

enum Merged {
    No(AstNode, AstNode),
    Yes(AstNode),
}
