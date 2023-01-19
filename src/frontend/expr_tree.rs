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

    Seek(CellOffset, isize),

    BoundsCheck(BoundsRange),

    Loop(BlockBalanced, CellOffset, Vec<Instruction>),
    If(BlockBalanced, CellOffset, Vec<Instruction>),
}
impl Instruction {
    pub fn moves_pointer(&self) -> bool {
        match self {
            Self::Move(_) => true,
            Self::Seek(_, _) => true,
            Self::If(bal, _, _) | Self::Loop(bal, _, _) => !bal,
            _ => false,
        }
    }
}

pub type CellOffset = isize;
pub type BlockBalanced = bool;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct BoundsRange {
    pub start: isize,
    pub length: usize,
}
impl BoundsRange {
    pub fn merge(self, other: Self) -> Self {
        let start = self.start.min(other.start);
        let end1 = self.start.checked_add_unsigned(self.length).unwrap();
        let end2 = other.start.checked_add_unsigned(other.length).unwrap();
        let end = end1.max(end2);
        let length = end - start;
        Self {
            start,
            length: length as usize,
        }
    }

    pub fn includes(&self, other: &Self) -> bool {
        let self_end = self.start.checked_add_unsigned(self.length).unwrap();
        let other_end = other.start.checked_add_unsigned(other.length).unwrap();
        self.start <= other.start && self_end >= other_end
    }
}
