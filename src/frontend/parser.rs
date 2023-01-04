use super::{
    expr_tree::{CellOffset, Instruction, Program},
    lexer::Token,
};

pub fn parse(mut src: impl Iterator<Item = Token>) -> Program {
    let (body, closed) = parse_instructions(&mut src);
    assert!(!closed);

    Program(body)
}
fn parse_instructions(src: &mut impl Iterator<Item = Token>) -> (Vec<Instruction>, bool) {
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
fn parse_instruction(src: &mut impl Iterator<Item = Token>) -> (Option<Instruction>, bool) {
    let Some(tok) = src.next() else { return (None, false) };

    let i = match tok {
        Token::Plus => Instruction::Modify(0, 1),
        Token::Minus => Instruction::Modify(0, -1),
        Token::Next => Instruction::Move(1),
        Token::Previous => Instruction::Move(-1),
        Token::Dot => Instruction::Output(0),
        Token::Comma => Instruction::Input(0),
        Token::Close => return (None, true),
        Token::Open => {
            let (body, closed) = parse_instructions(src);
            assert!(closed);
            collapse_loop(body, 0)
        }
    };

    (Some(i), false)
}

fn collapse_loop(body: Vec<Instruction>, counter: CellOffset) -> Instruction {
    if body.len() == 1 {
        match body[0] {
            Instruction::Modify(decr, amount) if decr == counter && amount % 2 != 0 => {
                Instruction::Set(counter, 0)
            }
            _ => Instruction::Loop(counter, body),
        }
    } else {
        Instruction::Loop(counter, body)
    }
}

fn merge(left: Instruction, right: Instruction) -> Merged {
    use Instruction::*;
    Merged::Yes(match (left, right) {
        (Modify(offa, a), Modify(offb, b)) if offa == offb => Modify(offa, a.wrapping_add(b)),
        (Move(a), Move(b)) => Move(a.wrapping_add(b)),
        (Set(c0, v0), Modify(c1, o1)) if c0 == c1 => Set(c0, v0.wrapping_add_signed(o1)),
        (left, right) => return Merged::No(left, right),
    })
}

enum Merged {
    No(Instruction, Instruction),
    Yes(Instruction),
}
