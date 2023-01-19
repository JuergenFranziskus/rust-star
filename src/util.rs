use std::io::{self, Write};

pub mod arena;

pub fn print_indent<O: Write>(indent: &str, last: bool, out: &mut O) -> io::Result<String> {
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
