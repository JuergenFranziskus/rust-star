use super::{
    instruction::{BinaryOp, Expr, Instruction, LeafExpr, UnaryOp},
    register::RegisterID,
    Module,
};
use std::collections::{HashMap, HashSet};

pub fn optimize_module(module: &mut Module) {
    let mut changed = true;
    while changed {
        changed = false;
        changed |= local_cse(module);
        changed |= remove_identity_muls(module);
        changed |= remove_negating_muls(module);
        changed |= do_constant_operations(module);
        changed |= propagate_leaf_assigns(module);
        changed |= remove_dead_assignments(module);

        remove_nops(module);
    }
}

pub fn local_cse(module: &mut Module) -> bool {
    let mut exprs = HashMap::new();
    let mut changed = false;
    for block in module.blocks.iter_mut() {
        exprs.clear();
        for i in block.body.iter_mut() {
            if let Instruction::Assign(target, val) = i {
                if !val.is_leaf() && exprs.contains_key(val) {
                    changed = true;
                    let new_val = exprs[val];
                    *val = Expr::Leaf(LeafExpr::Register(new_val));
                } else {
                    exprs.insert(*val, *target);
                }
            }
        }
    }

    changed
}

pub fn remove_identity_muls(module: &mut Module) -> bool {
    let mut changed = false;
    for block in &mut module.blocks {
        for i in &mut block.body {
            if let &mut Instruction::Assign(target, Expr::Binary(a, BinaryOp::Mul, b)) = i {
                if a.is_constant_multiplicative_identity() {
                    changed = true;
                    *i = Instruction::Assign(target, Expr::Leaf(b));
                } else if b.is_constant_multiplicative_identity() {
                    changed = true;
                    *i = Instruction::Assign(target, Expr::Leaf(a));
                }
            }
        }
    }

    changed
}
pub fn remove_negating_muls(module: &mut Module) -> bool {
    let mut changed = false;
    for block in &mut module.blocks {
        for i in &mut block.body {
            if let &mut Instruction::Assign(target, Expr::Binary(a, BinaryOp::Mul, b)) = i {
                if a.is_constant_multiplicative_negation() {
                    changed = true;
                    *i = Instruction::Assign(target, Expr::Unary(b, UnaryOp::Neg));
                } else if b.is_constant_multiplicative_negation() {
                    changed = true;
                    *i = Instruction::Assign(target, Expr::Unary(a, UnaryOp::Neg));
                }
            }
        }
    }

    changed
}
pub fn do_constant_operations(module: &mut Module) -> bool {
    let mut changed = false;

    for block in &mut module.blocks {
        for i in &mut block.body {
            if let Instruction::Assign(_, e) = i {
                if matches!(e, Expr::Leaf(_)) {
                    continue;
                }
                if let Some(value) = e.eval_const() {
                    *e = Expr::Leaf(value.to_leaf_expr());
                    changed = true;
                }
            }
        }
    }

    changed
}

pub fn propagate_leaf_assigns(module: &mut Module) -> bool {
    let mut replacements: HashMap<RegisterID, LeafExpr> =
        HashMap::with_capacity(module.registers.len());
    for block in &mut module.blocks {
        for i in &mut block.body {
            if let &mut Instruction::Assign(target, Expr::Leaf(l)) = i {
                replacements.insert(target, l);
            }
        }
    }

    let mut changed = false;
    for block in &mut module.blocks {
        for i in &mut block.body {
            changed |= i.replace_usages(&replacements);
        }
    }

    changed
}

pub fn remove_dead_assignments(module: &mut Module) -> bool {
    let mut not_dead = HashSet::new();
    let instructions = module.blocks.iter().map(|b| b.body.iter()).flatten();
    instructions.for_each(|i| i.populate_used(&mut not_dead));

    let mut changed = false;

    for block in &mut module.blocks {
        for i in &mut block.body {
            if let &mut Instruction::Assign(target, _) = i {
                if !not_dead.contains(&target) {
                    *i = Instruction::Nop;
                    changed = true;
                }
            }
        }
    }

    changed
}

pub fn remove_nops(module: &mut Module) {
    for block in &mut module.blocks {
        block.body.retain(|i| i != &Instruction::Nop);
    }
}
