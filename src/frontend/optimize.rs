use std::collections::HashSet;

use super::expr_tree::{CellOffset, Instruction, Program};

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

            Instruction::VerifyCell(cell) => *cell += offset,

            Instruction::Loop(_, cell, body) => {
                *cell += offset;
                normalize_pointer_rec(body, offset);
            }
            Instruction::If(_, base, body) => {
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

        VerifyCell(_) => true,

        Loop(_, _, body) => {
            remove_dead_rec(body);
            true
        }
        If(_, _, body) => {
            remove_dead_rec(body);
            body.len() != 0
        }
    });
}

pub fn mark_balanced_blocks(p: &mut Program) {
    mark_bal_blocks_rec(&mut p.0)
}
fn mark_bal_blocks_rec(i: &mut Vec<Instruction>) {
    for i in i {
        use Instruction::*;
        match i {
            If(bal, _, body) | Loop(bal, _, body) => {
                mark_bal_blocks_rec(body);
                if body.iter().all(|i| !i.moves_pointer()) {
                    *bal = true;
                }
            }
            _ => (),
        }
    }
}

pub fn remove_dead_verifications(program: &mut Program) {
    let mut verified = HashSet::new();
    remove_dead_verify_rec(&mut program.0, &mut verified);
}
fn remove_dead_verify_rec(i: &mut Vec<Instruction>, verified: &mut HashSet<CellOffset>) {
    i.retain_mut(|i| {
        if i.moves_pointer() {
            verified.clear();
        }

        use Instruction::*;
        match i {
            VerifyCell(cell) => verified.insert(*cell),
            If(_, _, body) | Loop(_, _, body) => {
                remove_dead_verify_rec(body, verified);
                true
            }
            _ => true,
        }
    })
}

pub fn recog_additions(p: &mut Program) {
    p.0.iter_mut().for_each(recog_additions_rec);
}
fn recog_additions_rec(i: &mut Instruction) {
    if let Instruction::Loop(_, base, body) = i {
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
            *i = Instruction::If(true, *base, body);
        }
    }
}
