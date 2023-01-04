#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Program(pub Vec<Instruction>);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Instruction {
    Modify(CellOffset, i8),
    Move(isize),
    Output(CellOffset),
    Input(CellOffset),
    Set(CellOffset, u8),

    AddMultiple {
        target: CellOffset,
        base: CellOffset,
        factor: i8,
    },

    Loop(CellOffset, Vec<Instruction>),
    If(CellOffset, Vec<Instruction>),
}

pub type CellOffset = isize;
