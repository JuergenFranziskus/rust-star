use std::io::{self, Write};

pub fn add_with_index<T, I>(vec: &mut Vec<T>, f: impl FnOnce(I) -> T) -> I
where
    I: From<usize> + Clone,
{
    let i = I::from(vec.len());
    let item = f(i.clone());
    vec.push(item);
    i
}

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
