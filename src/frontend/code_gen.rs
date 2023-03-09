use std::{
    collections::{HashMap, HashSet},
    mem::take,
};

use super::expr_tree::{CellOffset, Instruction, Program};
use crate::ir::{
    block::BlockID,
    builder::Builder,
    instruction::{LeafExpr, TestOp},
    register::RegisterID,
    types::Type,
    Module,
};

pub fn gen_program(program: &Program) -> Module {
    let mut module = Module::new();
    let code_gen = CodeGen::new(&mut module);
    code_gen.gen_program(program);

    module
}

pub struct CodeGen<'a> {
    builder: Builder<'a>,
    index: RegisterID,

    indices: HashMap<CellOffset, RegisterID>,
    cells: HashMap<CellOffset, RegisterID>,
    written: HashSet<CellOffset>,
}
impl<'a> CodeGen<'a> {
    fn new(module: &'a mut Module) -> Self {
        let entry = module.add_block();
        module.set_entry_block(entry);
        let mut builder = Builder::new(module, entry);
        let index = builder.set(0u64);
        Self {
            builder,
            index,
            indices: HashMap::new(),
            cells: HashMap::new(),
            written: HashSet::new(),
        }
    }

    fn gen_program(mut self, program: &Program) {
        for i in &program.0 {
            self.gen_instruction(i);
        }
    }
    fn gen_instruction(&mut self, instruction: &Instruction) {
        use Instruction::*;
        match instruction {
            &Modify(offset, amount) => {
                let old = self.get_cell(offset);
                let new = self.builder.add(old, amount);
                self.set_cell(offset, new);
            }
            &Move(amount) => self.move_index(amount),
            &Output(cell) => {
                let val = self.get_cell(cell);
                self.builder.output(val);
            }
            &Input(cell) => {
                let old = self.get_cell(cell);
                let read = self.builder.input(old);
                self.set_cell(cell, read);
            }
            &Set(cell, val) => self.set_cell(cell, val),
            &AddMultiple {
                target,
                base,
                factor,
            } => {
                let target_val = self.get_cell(target);
                let base_val = self.get_cell(base);
                let addend = self.builder.mul(base_val, factor);
                let total = self.builder.add(target_val, addend);
                self.set_cell(target, total);
            }
            &BoundsCheck(bounds) => {
                let start = self.builder.add(self.index, bounds.start as i64);
                let end = self.builder.add(start, bounds.length as u64);
                self.builder.check_bounds(start, end);
            }
            &Loop(balanced, condition, ref body) => self.gen_loop(!balanced, condition, body),
            &If(balanced, condition, ref body) => self.gen_if(!balanced, condition, body),
        }
    }

    fn gen_loop(&mut self, unbalanced: bool, condition: CellOffset, instructions: &[Instruction]) {
        let header = self.builder.add_block();
        let body = self.builder.add_block();
        let end = self.builder.add_block();

        self.jump_to(header, unbalanced);
        let context = self.save_context();

        self.enter_branch(header, unbalanced);
        let cell_val = self.get_cell(condition);
        let not_zero = self.builder.test(TestOp::NotEqual, cell_val, 0i8);
        self.branch_to(not_zero, body, end, unbalanced);

        self.enter_branch(body, false);
        for i in instructions {
            self.gen_instruction(i);
        }
        self.jump_to(header, unbalanced);

        self.enter_branch(end, unbalanced);
        self.restore_context(context);
    }
    fn gen_if(&mut self, unbalanced: bool, condition: CellOffset, instructions: &[Instruction]) {
        let body = self.builder.add_block();
        let end = self.builder.add_block();

        let cell_val = self.get_cell(condition);
        let not_zero = self.builder.test(TestOp::NotEqual, cell_val, 0i8);
        self.branch_to(not_zero, body, end, unbalanced);
        let context = self.save_context();

        self.enter_branch(body, false);
        for i in instructions {
            self.gen_instruction(i);
        }
        self.jump_to(end, unbalanced);

        self.enter_branch(end, unbalanced);
        self.restore_context(context);
    }

    fn branch_to(&mut self, c: impl Into<LeafExpr>, then: BlockID, els: BlockID, unbalanced: bool) {
        if unbalanced {
            self.spill_values();
            self.spill_indices();
            self.builder.branch(c, then, (els, self.index));
        } else {
            self.spill_values();
            self.builder.branch(c, then, els);
        }
    }
    fn jump_to(&mut self, block: BlockID, unbalanced: bool) {
        if unbalanced {
            self.spill_values();
            self.spill_indices();
            self.builder.jump((block, self.index));
        } else {
            self.spill_values();
            self.builder.jump(block);
        }
    }
    fn enter_branch(&mut self, block: BlockID, unbalanced: bool) {
        self.builder.select_block(block);
        if unbalanced {
            self.index = self.builder.add_parameter(Type::I64);
        }
    }
    fn move_index(&mut self, by: isize) {
        self.index = self.builder.add(self.index, by as i64);
        self.indices = self.indices.drain().map(|(k, v)| (k - by, v)).collect();
        self.cells = self.cells.drain().map(|(k, v)| (k - by, v)).collect();
        self.written = self.written.drain().map(|k| k - by).collect();
    }

    fn spill_indices(&mut self) {
        self.indices.clear();
    }
    fn spill_values(&mut self) {
        let mut values = take(&mut self.cells);
        let mut written = take(&mut self.written);

        values
            .drain()
            .filter(|(k, _)| written.contains(k))
            .for_each(|(offset, value)| {
                let index = self.get_cell_index(offset);
                self.builder.store_cell(index, value);
            });

        written.clear();
        self.cells = values;
        self.written = written;
    }
    fn get_cell_index(&mut self, offset: CellOffset) -> RegisterID {
        if let Some(&index) = self.indices.get(&offset) {
            index
        } else {
            let cell_index = self.builder.add(self.index, offset as i64);
            self.indices.insert(offset, cell_index);
            cell_index
        }
    }
    fn get_cell(&mut self, offset: CellOffset) -> RegisterID {
        if let Some(&value) = self.cells.get(&offset) {
            value
        } else {
            let index = self.get_cell_index(offset);
            let value = self.builder.load_cell(index);
            self.cells.insert(offset, value);
            value
        }
    }
    fn set_cell(&mut self, offset: CellOffset, value: impl Into<LeafExpr>) {
        let value: LeafExpr = value.into();

        match value {
            LeafExpr::Int(c) => self.cells.insert(offset, self.builder.set(c)),
            LeafExpr::Register(r) => self.cells.insert(offset, r),
        };
        self.written.insert(offset);
    }

    fn save_context(
        &self,
    ) -> (
        HashMap<CellOffset, RegisterID>,
        HashMap<CellOffset, RegisterID>,
        HashSet<CellOffset>,
    ) {
        let cells = self.cells.clone();
        let indices = self.indices.clone();
        let written = self.written.clone();
        (cells, indices, written)
    }
    fn restore_context(
        &mut self,
        context: (
            HashMap<CellOffset, RegisterID>,
            HashMap<CellOffset, RegisterID>,
            HashSet<CellOffset>,
        ),
    ) {
        self.cells = context.0;
        self.indices = context.1;
        self.written = context.2;
    }
}
