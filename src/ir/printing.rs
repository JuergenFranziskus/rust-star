use super::{
    block::Block,
    instruction::{BinaryOp, Instruction, LeafExpr, TestOp, UnaryOp},
    register::RegisterID,
    Module,
};
use crate::ir::instruction::Expr;
use std::io::{self, Write};

pub struct Printer<O> {
    out: O,
}
impl<O: Write> Printer<O> {
    pub fn new(out: O) -> Self {
        Self { out }
    }

    pub fn print_module(&mut self, m: &Module) -> io::Result<()> {
        for block in &m.blocks {
            self.print_block(block, m)?;
        }
        Ok(())
    }
    fn print_block(&mut self, b: &Block, m: &Module) -> io::Result<()> {
        write!(self.out, "{}", b.id())?;
        if let Some((&last, others)) = b.parameters().split_last() {
            write!(self.out, "(")?;
            for &other in others {
                self.print_reg_with_type(other, m)?;
                write!(self.out, ", ")?;
            }
            self.print_reg_with_type(last, m)?;
            write!(self.out, ")")?;
        }
        writeln!(self.out, ":")?;

        for instruction in b.body() {
            self.print_instruction(instruction, m)?;
        }

        Ok(())
    }
    fn print_instruction(&mut self, i: &Instruction, m: &Module) -> io::Result<()> {
        write!(self.out, "\t")?;

        use Instruction::*;
        match i {
            &Nop => writeln!(self.out, "nop")?,
            &LoadCell(target, index) => writeln!(self.out, "{target} = load({index})")?,
            &StoreCell(index, value) => writeln!(self.out, "store({index}, {value})")?,
            &BoundsCheck(index, bounds) => writeln!(self.out, "boundscheck({index}, {bounds})")?,
            &Assign(target, value) => {
                write!(self.out, "{target} = ")?;
                match value {
                    Expr::Leaf(val) => self.print_expr_leaf(val, m)?,
                    Expr::Binary(a, op, b) => self.print_bin_op(a, op, b, m)?,
                    Expr::Unary(a, op) => self.print_un_op(a, op, m)?,
                    Expr::Test(a, op, b) => self.print_test_op(a, op, b, m)?,
                };
                writeln!(self.out)?;
            }
            &Output(value) => writeln!(self.out, "stdout << {value}")?,
            &Input(target, default) => writeln!(self.out, "{target} = eof ? {default} : stdin")?,
            Jump(target) => writeln!(self.out, "jump {target}")?,
            Branch(condition, then, els) => {
                writeln!(self.out, "branch {condition}\n\t  {then}\n\t  {els}")?
            }
        }

        Ok(())
    }
    fn print_expr_leaf(&mut self, leaf: LeafExpr, m: &Module) -> io::Result<()> {
        let leaf_type = leaf.expr_type(m);
        write!(self.out, "{leaf_type} {leaf}")
    }
    fn print_bin_op(
        &mut self,
        a: LeafExpr,
        op: BinaryOp,
        b: LeafExpr,
        m: &Module,
    ) -> io::Result<()> {
        let a_type = a.expr_type(m);
        write!(self.out, "{op} {a_type} {a}, {b}")
    }
    fn print_un_op(&mut self, a: LeafExpr, op: UnaryOp, m: &Module) -> io::Result<()> {
        let a_type = a.expr_type(m);
        write!(self.out, "{op} {a_type} {a}")
    }
    fn print_test_op(
        &mut self,
        a: LeafExpr,
        op: TestOp,
        b: LeafExpr,
        m: &Module,
    ) -> io::Result<()> {
        let a_type = a.expr_type(m);
        write!(self.out, "{op} {a_type} {a}, {b}")
    }

    fn print_reg_with_type(&mut self, reg: RegisterID, m: &Module) -> io::Result<()> {
        let rt = m[reg].register_type();
        write!(self.out, "{rt} {reg}")
    }
}
