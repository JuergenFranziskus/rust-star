use super::{block::Block, instruction::Instruction, register::RegisterID, Module};
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
            &Set(target, value) => writeln!(self.out, "{target} = {value}")?,
            &Binary(op, target, a, b) => {
                let target_type = m[target].register_type();
                writeln!(self.out, "{target} = {op} {target_type} {a}, {b}")?;
            }
            &Unary(op, target, a) => {
                let target_type = m[target].register_type();
                writeln!(self.out, "{target} = {op} {target_type} {a}")?;
            }
            &Test(op, target, a, b) => {
                let a_type = a.expr_type(m);
                writeln!(self.out, "{target} = {op} {a_type} {a}, {b}")?;
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

    fn print_reg_with_type(&mut self, reg: RegisterID, m: &Module) -> io::Result<()> {
        let rt = m[reg].register_type();
        write!(self.out, "{rt} {reg}")
    }
}
