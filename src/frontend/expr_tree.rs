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

    VerifyCell(CellOffset),

    Loop(BlockBalanced, CellOffset, Vec<Instruction>),
    If(BlockBalanced, CellOffset, Vec<Instruction>),
}
impl Instruction {
    pub fn moves_pointer(&self) -> bool {
        match self {
            Self::Move(_) => true,
            Self::If(bal, _, _) | Self::Loop(bal, _, _) => !bal,
            _ => false,
        }
    }
}

pub type CellOffset = isize;
pub type BlockBalanced = bool;
