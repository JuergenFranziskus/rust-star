use self::{
    instruction::{Expr, Instruction},
    types::{Type, Types},
};
use crate::util::arena::{Arena, ID};
use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
};

pub mod instruction;
pub mod types;

pub struct Module {
    types: Types,
    main_function: Option<ID<Function>>,
    functions: Arena<Function>,
    registers: Arena<Register>,
    variables: Arena<Variable>,
    blocks: Arena<Block>,
}
impl Module {
    pub fn new() -> Module {
        Module {
            types: Types::new(),
            main_function: None,
            functions: Arena::new(),
            registers: Arena::new(),
            variables: Arena::new(),
            blocks: Arena::new(),
        }
    }

    pub fn types(&self) -> &Types {
        &self.types
    }
    pub fn types_mut(&mut self) -> &mut Types {
        &mut self.types
    }
    pub fn set_main_function(&mut self, fid: ID<Function>) {
        self.main_function = Some(fid);
    }

    pub fn add_function(&mut self, return_type: impl Into<Type>) -> ID<Function> {
        let id = self.functions.next_id();
        self.functions.push(Function {
            id,
            return_type: return_type.into(),
            variables: Vec::new(),
            registers: Vec::new(),
            parameters: Vec::new(),
            blocks: Vec::new(),
            entry_block: None,
        })
    }
    pub fn func(&self, fid: ID<Function>) -> Option<&Function> {
        self.functions.get(fid)
    }
    pub fn func_mut(&mut self, fid: ID<Function>) -> Option<&mut Function> {
        self.functions.get_mut(fid)
    }

    pub fn add_register(&mut self, fid: ID<Function>, reg_type: impl Into<Type>) -> ID<Register> {
        let id = self.registers.next_id();
        self.registers.push(Register {
            id,
            function: fid,
            reg_type: reg_type.into(),
        });

        self[fid].add_register(id);
        id
    }
    pub fn reg(&self, rid: ID<Register>) -> Option<&Register> {
        self.registers.get(rid)
    }
    pub fn reg_mut(&mut self, rid: ID<Register>) -> Option<&mut Register> {
        self.registers.get_mut(rid)
    }

    pub fn add_parameter(
        &mut self,
        fid: ID<Function>,
        param_type: impl Into<Type>,
    ) -> ID<Register> {
        let rid = self.add_register(fid, param_type);
        self[fid].add_parameter(rid);
        rid
    }

    pub fn add_variable(&mut self, fid: ID<Function>, var_type: impl Into<Type>) -> ID<Variable> {
        let id = self.variables.next_id();
        self.variables.push(Variable {
            id,
            function: fid,
            var_type: var_type.into(),
        });
        self[fid].add_variable(id);
        id
    }
    pub fn var(&self, vid: ID<Variable>) -> Option<&Variable> {
        self.variables.get(vid)
    }
    pub fn var_mut(&mut self, vid: ID<Variable>) -> Option<&mut Variable> {
        self.variables.get_mut(vid)
    }

    pub fn add_block(&mut self, fid: ID<Function>) -> ID<Block> {
        let id = self.blocks.next_id();
        self.blocks.push(Block {
            id,
            function: fid,
            phi_nodes: HashMap::new(),
            body: Vec::new(),
        });
        self[fid].add_block(id);
        id
    }
    pub fn block(&self, bid: ID<Block>) -> Option<&Block> {
        self.blocks.get(bid)
    }
    pub fn block_mut(&mut self, bid: ID<Block>) -> Option<&mut Block> {
        self.blocks.get_mut(bid)
    }
}
impl Index<ID<Function>> for Module {
    type Output = Function;
    fn index(&self, index: ID<Function>) -> &Self::Output {
        self.func(index).unwrap()
    }
}
impl IndexMut<ID<Function>> for Module {
    fn index_mut(&mut self, index: ID<Function>) -> &mut Self::Output {
        self.func_mut(index).unwrap()
    }
}
impl Index<ID<Register>> for Module {
    type Output = Register;
    fn index(&self, index: ID<Register>) -> &Self::Output {
        self.reg(index).unwrap()
    }
}
impl IndexMut<ID<Register>> for Module {
    fn index_mut(&mut self, index: ID<Register>) -> &mut Self::Output {
        self.reg_mut(index).unwrap()
    }
}
impl Index<ID<Variable>> for Module {
    type Output = Variable;

    fn index(&self, index: ID<Variable>) -> &Self::Output {
        self.var(index).unwrap()
    }
}
impl IndexMut<ID<Variable>> for Module {
    fn index_mut(&mut self, index: ID<Variable>) -> &mut Self::Output {
        self.var_mut(index).unwrap()
    }
}
impl Index<ID<Block>> for Module {
    type Output = Block;

    fn index(&self, index: ID<Block>) -> &Self::Output {
        self.block(index).unwrap()
    }
}
impl IndexMut<ID<Block>> for Module {
    fn index_mut(&mut self, index: ID<Block>) -> &mut Self::Output {
        self.block_mut(index).unwrap()
    }
}

pub struct Function {
    id: ID<Function>,
    return_type: Type,
    variables: Vec<ID<Variable>>,
    registers: Vec<ID<Register>>,
    parameters: Vec<ID<Register>>,
    blocks: Vec<ID<Block>>,
    entry_block: Option<ID<Block>>,
}
impl Function {
    pub fn id(&self) -> ID<Function> {
        self.id
    }
    pub fn return_type(&self) -> Type {
        self.return_type
    }
    pub fn variables(&self) -> &[ID<Variable>] {
        &self.variables
    }
    pub fn registers(&self) -> &[ID<Register>] {
        &self.registers
    }
    pub fn parameters(&self) -> &[ID<Register>] {
        &self.parameters
    }
    pub fn blocks(&self) -> &[ID<Block>] {
        &self.blocks
    }
    pub fn entry_block(&self) -> Option<ID<Block>> {
        self.entry_block
    }

    fn add_register(&mut self, rid: ID<Register>) {
        self.registers.push(rid)
    }
    fn add_parameter(&mut self, rid: ID<Register>) {
        self.registers.push(rid);
    }
    fn add_variable(&mut self, vid: ID<Variable>) {
        self.variables.push(vid);
    }
    pub fn set_entry_block(&mut self, bid: ID<Block>) {
        self.entry_block = Some(bid)
    }

    fn add_block(&mut self, id: ID<Block>) {
        self.blocks.push(id)
    }
}

pub struct Block {
    id: ID<Block>,
    function: ID<Function>,
    phi_nodes: HashMap<ID<Register>, PhiNode>,
    body: Vec<Instruction>,
}
impl Block {
    pub fn id(&self) -> ID<Block> {
        self.id
    }
    pub fn function(&self) -> ID<Function> {
        self.function
    }
    pub fn phi_nodes(&self) -> &HashMap<ID<Register>, PhiNode> {
        &self.phi_nodes
    }
    pub fn body(&self) -> &[Instruction] {
        &self.body
    }

    pub fn add_phi_node(&mut self, target: ID<Register>, values: HashMap<ID<Block>, Expr>) {
        self.phi_nodes.insert(target, PhiNode { values });
    }
    pub fn append_instruction(&mut self, i: Instruction) {
        self.body.push(i);
    }
}

pub struct PhiNode {
    values: HashMap<ID<Block>, Expr>,
}
impl PhiNode {
    pub fn values(&self) -> &HashMap<ID<Block>, Expr> {
        &self.values
    }
}

pub struct Register {
    id: ID<Register>,
    function: ID<Function>,
    reg_type: Type,
}
impl Register {
    pub fn id(&self) -> ID<Register> {
        self.id
    }
    pub fn function(&self) -> ID<Function> {
        self.function
    }
    pub fn reg_type(&self) -> Type {
        self.reg_type
    }
}

pub struct Variable {
    id: ID<Variable>,
    function: ID<Function>,
    var_type: Type,
}
impl Variable {
    pub fn id(&self) -> ID<Variable> {
        self.id
    }
    pub fn function(&self) -> ID<Function> {
        self.function
    }
    pub fn var_type(&self) -> Type {
        self.var_type
    }
}
