use super::expr_tree::{Instruction, Program};

pub fn normalize_pointer_movement(program: &mut Program) {
    normalize_pointer_rec(&mut program.0, 0)
}
fn normalize_pointer_rec(i: &mut Vec<Instruction>, mut offset: isize) {
    let initial = offset;

    for i in i.iter_mut() {
        match i {
            Instruction::Move(movement) => {
                offset += *movement;
                *movement = 0;
            }
            Instruction::Modify(cell, _) => *cell += offset,
            Instruction::Output(cell) => *cell += offset,
            Instruction::Input(cell) => *cell += offset,
            Instruction::Set(cell, _) => *cell += offset,
            Instruction::AddMultiple {
                base, target: cell, ..
            } => {
                *base += offset;
                *cell += offset;
            }

            Instruction::Loop(cell, body) => {
                *cell += offset;
                normalize_pointer_rec(body, offset);
            }
            Instruction::If(base, body) => {
                *base += offset;
                normalize_pointer_rec(body, offset);
            }
        }
    }

    if offset != initial {
        let difference = offset - initial;
        i.push(Instruction::Move(difference));
    }
}

pub fn remove_dead(program: &mut Program) {
    remove_dead_rec(&mut program.0);
}
fn remove_dead_rec(i: &mut Vec<Instruction>) {
    use Instruction::*;
    i.retain_mut(|i| match i {
        Modify(_, amount) => *amount != 0,
        Move(amount) => *amount != 0,
        Output(_) => true,
        Input(_) => true,
        Set(_, _) => true,
        AddMultiple { .. } => true,

        Loop(_, body) => {
            remove_dead_rec(body);
            true
        }
        If(_, body) => {
            remove_dead_rec(body);
            body.len() != 0
        }
    });
}

pub fn recog_additions(p: &mut Program) {
    p.0.iter_mut().for_each(recog_additions_rec);
}
fn recog_additions_rec(i: &mut Instruction) {
    if let Instruction::Loop(base, body) = i {
        body.iter_mut().for_each(recog_additions_rec);

        let mut args = Vec::new();
        let mut decremented = false;
        for i in body {
            if let Instruction::Modify(cell, amount) = i {
                if decremented && cell == base {
                    return;
                } else if cell == base && *amount == -1 {
                    decremented = true;
                } else {
                    args.push((*cell, *amount));
                }
            } else {
                return;
            }
        }

        if decremented {
            let mut body: Vec<_> = args
                .into_iter()
                .map(|(c, a)| Instruction::AddMultiple {
                    base: *base,
                    target: c,
                    factor: a,
                })
                .collect();
            body.push(Instruction::Set(*base, 0));
            *i = Instruction::If(*base, body);
        }
    }
}
