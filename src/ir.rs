use self::{
    block::{Block, BlockID},
    register::{Register, RegisterID},
    types::Type,
};
use crate::util::add_with_index;
use std::ops::{Index, IndexMut};

pub mod block;
pub mod builder;
pub mod exec;
pub mod instruction;
pub mod printing;
pub mod register;
pub mod types;

pub struct Module {
    entry: Option<BlockID>,
    blocks: Vec<Block>,
    registers: Vec<Register>,
}
impl Module {
    pub fn new() -> Self {
        Self {
            entry: None,
            blocks: Vec::new(),
            registers: Vec::new(),
        }
    }

    pub fn set_entry_block(&mut self, entry: BlockID) {
        self.entry = Some(entry);
    }
    pub fn add_block(&mut self) -> BlockID {
        add_with_index(&mut self.blocks, |id| Block::new(id))
    }
    pub fn block(&self, id: BlockID) -> Option<&Block> {
        self.blocks.get(id.0)
    }
    pub fn block_mut(&mut self, id: BlockID) -> Option<&mut Block> {
        self.blocks.get_mut(id.0)
    }

    pub fn add_register(&mut self, reg_type: Type) -> RegisterID {
        add_with_index(&mut self.registers, |id| Register::new(id, reg_type))
    }
    pub fn reg(&self, id: RegisterID) -> Option<&Register> {
        self.registers.get(id.0)
    }
    pub fn reg_mut(&mut self, id: RegisterID) -> Option<&mut Register> {
        self.registers.get_mut(id.0)
    }

    pub fn add_parameter(&mut self, block: BlockID, param_type: Type) -> RegisterID {
        let id = self.add_register(param_type);
        self[block].add_parameter(id);
        id
    }

    fn entry_block(&self) -> BlockID {
        self.entry.unwrap()
    }
}
impl Index<BlockID> for Module {
    type Output = Block;
    fn index(&self, index: BlockID) -> &Self::Output {
        &self.blocks[index.0]
    }
}
impl IndexMut<BlockID> for Module {
    fn index_mut(&mut self, index: BlockID) -> &mut Self::Output {
        &mut self.blocks[index.0]
    }
}
impl Index<RegisterID> for Module {
    type Output = Register;
    fn index(&self, index: RegisterID) -> &Self::Output {
        &self.registers[index.0]
    }
}
impl IndexMut<RegisterID> for Module {
    fn index_mut(&mut self, index: RegisterID) -> &mut Self::Output {
        &mut self.registers[index.0]
    }
}
