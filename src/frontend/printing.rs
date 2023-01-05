use super::expr_tree::{Instruction, Program};
use std::{
    fmt::Display,
    io::{self, Write},
};

pub fn pretty_print<O: Write>(program: &Program, mut out: O) -> io::Result<()> {
    let indent = print_indent("", true, &mut out)?;
    writeln!(out, "Program:")?;

    if let Some((last, nodes)) = program.0.split_last() {
        for node in nodes {
            print_instruction(node, &indent, false, &mut out)?;
        }
        print_instruction(last, &indent, true, &mut out)?;
    }

    Ok(())
}

fn print_instruction<O: Write>(
    node: &Instruction,
    indent: &str,
    last: bool,
    out: &mut O,
) -> io::Result<()> {
    let indent = &print_indent(indent, last, out)?;

    use Instruction::*;
    match node {
        Modify(cell, amount) => writeln!(out, "{} += {amount}", Cell(*cell))?,
        Move(amount) => writeln!(out, "ptr += {amount}")?,
        Output(cell) => writeln!(out, "write(stdout, {})", Cell(*cell))?,
        Input(cell) => writeln!(out, "{} = read(stdin)", Cell(*cell))?,
        Set(cell, value) => writeln!(out, "{} = {value}", Cell(*cell))?,
        AddMultiple {
            base,
            target: cell,
            factor,
        } => writeln!(out, "{} += {} * {}", Cell(*cell), Cell(*base), factor)?,

        VerifyCell(cell) => writeln!(out, "verify({})", Cell(*cell))?,

        Loop(_, cell, body) => {
            writeln!(out, "while {} != 0", Cell(*cell))?;
            if let Some((last, body)) = body.split_last() {
                for i in body {
                    print_instruction(i, indent, false, out)?;
                }
                print_instruction(last, indent, true, out)?;
            }
        }
        If(_, cell, body) => {
            writeln!(out, "if {} != 0", Cell(*cell))?;
            if let Some((last, body)) = body.split_last() {
                for i in body {
                    print_instruction(i, indent, false, out)?;
                }
                print_instruction(last, indent, true, out)?;
            }
        }
    }

    Ok(())
}

struct Cell(isize);
impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cell = self.0;
        write!(f, "[ptr")?;
        if cell > 0 {
            write!(f, " + {cell}]")?;
        } else if cell < 0 {
            write!(f, " - {}]", -(cell as i16))?;
        } else {
            write!(f, "]")?;
        }

        Ok(())
    }
}

fn print_indent<O: Write>(indent: &str, last: bool, out: &mut O) -> io::Result<String> {
    write!(out, "{}", indent)?;

    if last {
        write!(out, "└── ")?;
    } else {
        write!(out, "├── ")?;
    }

    let new = if last {
        format!("{}   ", indent)
    } else {
        format!("{}│   ", indent)
    };

    Ok(new)
}
