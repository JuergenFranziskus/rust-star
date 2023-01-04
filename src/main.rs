use rustfck::frontend::{
    expr_tree::{Instruction, Program},
    lexer::lex,
    optimize::{normalize_pointer_movement, recog_additions, remove_dead},
    parser::parse,
    printing::pretty_print,
};
use std::io::{stderr, stdin, stdout, Cursor, Read, Write};

fn main() {
    let src = std::fs::read_to_string("./programs/mandelbrot.b").unwrap();
    let tokens = lex(Cursor::new(src));
    let mut program = parse(tokens);

    normalize_pointer_movement(&mut program);
    remove_dead(&mut program);
    recog_additions(&mut program);

    pretty_print(&program, stderr()).unwrap();
    interpret(&program);
}

fn interpret(p: &Program) {
    let mut mem = [0; 30000];
    let mut ptr = 0;
    for i in &p.0 {
        exec_i(i, &mut mem, &mut ptr);
    }
}
fn exec_i(i: &Instruction, memory: &mut [u8; 30000], ptr: &mut usize) {
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

        Loop(cell, body) => {
            while memory[ptr.wrapping_add_signed(*cell)] != 0 {
                for i in body {
                    exec_i(i, memory, ptr);
                }
            }
        }
        If(cell, body) => {
            if memory[ptr.wrapping_add_signed(*cell)] != 0 {
                for i in body {
                    exec_i(i, memory, ptr);
                }
            }
        }
    }
}
