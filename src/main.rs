use rustfck::{
    frontend::{code_gen::gen_program, lexer::lex, optimize::apply_optimizations, parser::parse},
    ir::{exec::Exec, printing::Printer},
};
use std::io::{stdin, stdout, Cursor};

fn main() {
    let program_name = "bitwidth";

    let src = std::fs::read_to_string(format!("./programs/{}.b", program_name)).unwrap();
    let tokens = lex(Cursor::new(src));
    let ast = parse(tokens);
    let mut program = ast.gen_expr_tree();

    apply_optimizations(&mut program);

    let module = gen_program(&program);
    Printer::new(stdout()).print_module(&module).unwrap();
    let mut exec = Exec::new(stdout(), stdin());
    exec.exec_program(&module).unwrap();
}
