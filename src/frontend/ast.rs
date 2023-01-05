use super::expr_tree::{Instruction, Program};

pub struct Ast(pub Vec<AstNode>);
impl Ast {
    pub fn gen_expr_tree(&self) -> Program {
        let mut body = Vec::new();
        self.0.iter().for_each(|i| i.gen_expr_tree(&mut body));
        Program(body)
    }
}

pub enum AstNode {
    Modify(i8),
    Move(isize),
    Output,
    Input,
    Set(u8),
    Loop(Vec<AstNode>),
}
impl AstNode {
    fn gen_expr_tree(&self, instructions: &mut Vec<Instruction>) {
        if self.accesses_cell() {
            instructions.push(Instruction::VerifyCell(0));
        }

        instructions.push(match self {
            Self::Modify(amount) => Instruction::Modify(0, *amount),
            Self::Move(amount) => Instruction::Move(*amount),
            Self::Output => Instruction::Output(0),
            Self::Input => Instruction::Input(0),
            Self::Set(val) => Instruction::Set(0, *val),
            Self::Loop(body) => {
                let mut new_body = Vec::new();
                body.iter().for_each(|i| i.gen_expr_tree(&mut new_body));
                new_body.push(Instruction::VerifyCell(0));
                Instruction::Loop(false, 0, new_body)
            }
        });
    }

    fn accesses_cell(&self) -> bool {
        use AstNode::*;
        match self {
            Modify(_) => true,
            Move(_) => false,
            Output => true,
            Input => true,
            Set(_) => true,
            Loop(_) => true,
        }
    }
}
