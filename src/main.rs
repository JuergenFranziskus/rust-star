use rustfck::frontend::{
    expr_tree::{Instruction, Program},
    lexer::lex,
    optimize::{
        mark_balanced_blocks, normalize_pointer_movement, recog_additions, remove_dead,
        remove_dead_verifications,
    },
    parser::parse,
    printing::pretty_print,
};
use std::{
    io::{stderr, stdin, stdout, Cursor, Read, Write},
    iter::once,
};

fn main() {
    let src = std::fs::read_to_string("./programs/mandelbrot.b").unwrap();
    let tokens = lex(Cursor::new(src));
    let ast = parse(tokens);
    let mut program = ast.gen_expr_tree();

    normalize_pointer_movement(&mut program);
    remove_dead(&mut program);
    mark_balanced_blocks(&mut program);
    remove_dead_verifications(&mut program);
    recog_additions(&mut program);

    pretty_print(&program, stderr()).unwrap();
    interpret(&program);
}

fn interpret(p: &Program) {
    let mut mem = Vec::with_capacity(30000);
    let mut ptr = 0;
    for i in &p.0 {
        exec_i(i, &mut mem, &mut ptr);
    }
}
fn exec_i(i: &Instruction, memory: &mut Vec<u8>, ptr: &mut usize) {
    use Instruction::*;
    match i {
        Modify(cell, amount) => {
            let index = ptr.wrapping_add_signed(*cell);
            let old = memory[index];
            let new = old.wrapping_add_signed(*amount);
            memory[index] = new;
        }
        Move(offset) => *ptr = ptr.wrapping_add_signed(*offset),
        Output(cell) => {
            let val = memory[ptr.wrapping_add_signed(*cell)];
            stdout().write(&[val]).unwrap();
        }
        Input(cell) => {
            let mut buff = [0];
            let read = stdin().read(&mut buff).unwrap();
            if read != 0 {
                memory[ptr.wrapping_add_signed(*cell)] = buff[0];
            }
        }
        Set(cell, value) => memory[ptr.wrapping_add_signed(*cell)] = *value,
        AddMultiple {
            target,
            base,
            factor,
        } => {
            let old = memory[ptr.wrapping_add_signed(*target)];

            let base = memory[ptr.wrapping_add_signed(*base)];
            let addend = base.wrapping_mul(factor.wrapping_abs() as u8);

            let new = if factor.is_negative() {
                old.wrapping_sub(addend)
            } else {
                old.wrapping_add(addend)
            };
            memory[ptr.wrapping_add_signed(*target)] = new;
        }

        VerifyCell(cell) => {
            let i = ptr.wrapping_add_signed(*cell);
            let req_len = i + 1;

            if req_len > memory.len() {
                let diff = req_len - memory.len();
                memory.extend(once(0).cycle().take(diff + 1));
            }
        }

        Loop(_, cell, body) => {
            while memory[ptr.wrapping_add_signed(*cell)] != 0 {
                for i in body {
                    exec_i(i, memory, ptr);
                }
            }
        }
        If(_, cell, body) => {
            if memory[ptr.wrapping_add_signed(*cell)] != 0 {
                for i in body {
                    exec_i(i, memory, ptr);
                }
            }
        }
    }
}
