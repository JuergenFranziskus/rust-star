use super::expr_tree::{BoundsRange, CellOffset, Instruction, Program};

pub fn apply_optimizations(program: &mut Program) {
    normalize_pointer_movement(program);
    remove_dead(program);
    mark_balanced_blocks(program);
    merge_verifications(program);
    remove_dead_verifications(program);
    recog_additions(program);
    remove_dead_if_statements(program);
    merge_verifications(program);
    remove_dead_verifications(program);
}

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

            Instruction::BoundsCheck(cell) => cell.start += offset,

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

        BoundsCheck(_) => true,

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

pub fn merge_verifications(p: &mut Program) {
    merge_verif_rec(&mut p.0)
}
fn merge_verif_rec(instructions: &mut Vec<Instruction>) {
    let mut insertions = Vec::new();
    let mut insert_index = 0;
    let mut insert_value: Option<BoundsRange> = None;

    for (i, instruction) in instructions.iter_mut().enumerate() {
        use Instruction::*;
        match instruction {
            &mut BoundsCheck(cell) => {
                if let Some(val) = insert_value {
                    insert_value = Some(val.merge(cell));
                } else {
                    insert_value = Some(cell);
                }
            }
            Loop(bal, _, body) | If(bal, _, body) => {
                if !*bal {
                    if let Some(val) = insert_value.take() {
                        insertions.push((insert_index, val));
                    }
                    insert_index = i + 1;
                }
                merge_verif_rec(body);
            }
            _ => (),
        }
    }

    if let Some(val) = insert_value.take() {
        insertions.push((insert_index, val));
    }

    for (i, val) in insertions.into_iter().rev() {
        instructions.insert(i, Instruction::BoundsCheck(val));
    }
}

pub fn remove_dead_verifications(program: &mut Program) {
    let mut verified = None;
    remove_dead_verify_rec(&mut program.0, &mut verified);
}
fn remove_dead_verify_rec(i: &mut Vec<Instruction>, verified: &mut Option<BoundsRange>) {
    i.retain_mut(|i| {
        if i.moves_pointer() {
            *verified = None;
        }

        use Instruction::*;
        let ret = match i {
            BoundsCheck(cell) => {
                if let Some(highest) = verified {
                    let larger = !highest.includes(cell);
                    if larger {
                        *highest = (*highest).merge(*cell);
                    }
                    larger
                } else {
                    *verified = Some(*cell);
                    true
                }
            }
            If(_, _, body) | Loop(_, _, body) => {
                remove_dead_verify_rec(body, verified);
                true
            }
            _ => true,
        };
        if i.moves_pointer() {
            *verified = None;
        }

        ret
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

pub fn remove_dead_if_statements(p: &mut Program) {
    remove_dead_if_rec(&mut p.0)
}
fn remove_dead_if_rec(instructions: &mut Vec<Instruction>) {
    let mut changes = Vec::new();

    for (i, instruction) in instructions.iter_mut().enumerate() {
        use Instruction::*;
        match instruction {
            Loop(_, _, body) => remove_dead_if_rec(body),
            If(_, con, body) => {
                let can_inline = if_is_dead(body, *con);
                if can_inline {
                    changes.push((i, body.clone()));
                } else {
                    remove_dead_if_rec(body);
                }
            }
            _ => (),
        }
    }

    for (i, body) in changes.into_iter().rev() {
        instructions.remove(i);

        for instruction in body.into_iter().rev() {
            instructions.insert(i, instruction);
        }
    }
}
fn if_is_dead(i: &[Instruction], con: CellOffset) -> bool {
    for i in i {
        use Instruction::*;
        match i {
            &AddMultiple { base, .. } if base == con => (),
            &Set(cell, val) if cell == con && val == 0 => (),
            _ => return false,
        }
    }

    true
}
