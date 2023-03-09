use std::fmt::Display;

use super::{instruction::Instruction, register::RegisterID};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Block {
    pub(super) id: BlockID,
    pub(super) body: Vec<Instruction>,
    pub(super) parameters: Vec<RegisterID>,
}
impl Block {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            body: Vec::new(),
            parameters: Vec::new(),
        }
    }

    pub fn add_instruction(&mut self, i: Instruction) {
        self.body.push(i);
    }
    pub fn add_parameter(&mut self, reg: RegisterID) {
        self.parameters.push(reg);
    }

    pub fn id(&self) -> BlockID {
        self.id
    }
    pub fn parameters(&self) -> &[RegisterID] {
        &self.parameters
    }
    pub fn parameter(&self, index: usize) -> RegisterID {
        self.parameters[index]
    }
    pub fn body(&self) -> &[Instruction] {
        &self.body
    }
    pub fn instruction(&self, i: usize) -> &Instruction {
        &self.body[i]
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct BlockID(pub(super) usize);
impl From<usize> for BlockID {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
impl Display for BlockID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}", self.0)
    }
}
