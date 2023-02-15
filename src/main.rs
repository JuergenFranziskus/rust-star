use rustfck::frontend::{
    expr_tree::{BoundsRange, CellOffset, Instruction, Program},
    lexer::lex,
    parser::parse,
    printing::pretty_print, optimize::apply_optimizations,
};
use std::{
    io::{stderr, stdin, Cursor, Read, Write},
    iter::once,
};

fn main() {
    let program_name = "mandelbrot";

    let src = std::fs::read_to_string(format!("./programs/{}.b", program_name)).unwrap();
    let tokens = lex(Cursor::new(src));
    let ast = parse(tokens);
    let mut program = ast.gen_expr_tree();

    apply_optimizations(&mut program);

    pretty_print(&program, stderr()).unwrap();
    interpret(&program);
}

#[allow(dead_code)]
fn interpret<'a>(p: &'a Program) {
    let mut ctx = Ctx::new();
    for i in &p.0 {
        exec_i(i, &mut ctx);
    }
}
fn exec_i<'a>(i: &'a Instruction, ctx: &mut Ctx) {
    use Instruction::*;
    match i {
        &Modify(cell, amount) => {
            let old = ctx.read_cell(cell);
            let new = old.wrapping_add_signed(amount);
            ctx.write_cell(cell, new);
        }
        Move(offset) => ctx.move_pointer(*offset),
        &Output(cell) => {
            let val = ctx.read_cell(cell);
            stderr().write(&[val]).unwrap();
        }
        &Input(cell) => {
            let mut buff = [0];
            let read = stdin().read(&mut buff).unwrap();
            if read != 0 {
                ctx.write_cell(cell, buff[0]);
            }
        }
        Set(cell, value) => ctx.write_cell(*cell, *value),
        &AddMultiple {
            target,
            base,
            factor,
        } => {
            let old = ctx.read_cell(target);

            let base = ctx.read_cell(base);
            let addend = base.wrapping_mul(factor.wrapping_abs() as u8);

            let new = if factor.is_negative() {
                old.wrapping_sub(addend)
            } else {
                old.wrapping_add(addend)
            };
            ctx.write_cell(target, new);
        }

        &BoundsCheck(BoundsRange { start, length }) => {
            let offset = start.wrapping_add_unsigned(length);
            ctx.guarantee_cell(offset);
        }

        &Seek(cell, movement) => {
            while ctx.read_cell(cell) != 0 {
                ctx.move_pointer(movement);
                ctx.guarantee_cell(cell);
            }
        }
        Loop(_, cell, body) => {
            while ctx.read_cell(*cell) != 0 {
                for i in body {
                    exec_i(i, ctx);
                }
            }
        }
        If(_, cell, body) => {
            if ctx.read_cell(*cell) != 0 {
                for i in body {
                    exec_i(i, ctx);
                }
            }
        }
    }
}

struct Ctx {
    index: usize,
    memory: Vec<u8>,
}
impl Ctx {
    pub fn new() -> Self {
        Self {
            index: 0,
            memory: Vec::with_capacity(30000),
        }
    }

    pub fn read_cell(&self, cell: CellOffset) -> u8 {
        let i = self.index.wrapping_add_signed(cell);
        self.memory[i]
    }
    pub fn write_cell(&mut self, cell: CellOffset, value: u8) {
        let i = self.index.wrapping_add_signed(cell);
        self.memory[i] = value;
    }

    pub fn move_pointer(&mut self, movement: isize) {
        self.index = self.index.wrapping_add_signed(movement);
    }
    pub fn resize_array(&mut self, at_least_index: usize) {
        let req_len = at_least_index + 1;
        if req_len > self.memory.len() {
            let diff = req_len - self.memory.len();
            self.memory.extend(once(0).cycle().take(diff + 1));
        }
    }
    pub fn guarantee_cell(&mut self, cell: CellOffset) {
        let i = self.index.wrapping_add_signed(cell);
        self.resize_array(i);
    }
}
