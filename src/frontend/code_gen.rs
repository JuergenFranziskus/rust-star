use super::expr_tree::{CellOffset, Instruction, Program};
use crate::ir::{
    block::BlockID,
    builder::Builder,
    instruction::{Expr, TestOp},
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
}
impl<'a> CodeGen<'a> {
    fn new(module: &'a mut Module) -> Self {
        let entry = module.add_block();
        module.set_entry_block(entry);
        let mut builder = Builder::new(module, entry);
        let index = builder.set(0u64);
        Self { builder, index }
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
    }
    fn gen_if(&mut self, unbalanced: bool, condition: CellOffset, instructions: &[Instruction]) {
        let body = self.builder.add_block();
        let end = self.builder.add_block();

        let cell_val = self.get_cell(condition);
        let not_zero = self.builder.test(TestOp::NotEqual, cell_val, 0i8);
        self.branch_to(not_zero, body, end, unbalanced);

        self.enter_branch(body, false);
        for i in instructions {
            self.gen_instruction(i);
        }
        self.jump_to(end, unbalanced);

        self.enter_branch(end, unbalanced);
    }

    fn branch_to(&mut self, c: impl Into<Expr>, then: BlockID, els: BlockID, unbalanced: bool) {
        if unbalanced {
            self.builder.branch(c, then, (els, self.index));
        } else {
            self.builder.branch(c, then, els);
        }
    }
    fn jump_to(&mut self, block: BlockID, unbalanced: bool) {
        if unbalanced {
            self.builder.jump((block, self.index));
        } else {
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
    }
    fn get_cell(&mut self, offset: CellOffset) -> RegisterID {
        let cell_index = self.builder.add(self.index, offset as i64);
        let cell_val = self.builder.load_cell(cell_index);
        cell_val
    }
    fn set_cell(&mut self, offset: CellOffset, value: impl Into<Expr>) {
        let cell_index = self.builder.add(self.index, offset as i64);
        self.builder.store_cell(cell_index, value)
    }
}
