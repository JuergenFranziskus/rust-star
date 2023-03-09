use super::{block::BlockID, exec::Value, register::RegisterID, types::Type, Module};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Instruction {
    Nop,

    LoadCell(RegisterID, LeafExpr),
    StoreCell(LeafExpr, LeafExpr),
    BoundsCheck(LeafExpr, LeafExpr),

    Assign(RegisterID, Expr),

    Output(LeafExpr),
    Input(RegisterID, LeafExpr),

    Jump(TargetBlock),
    Branch(LeafExpr, TargetBlock, TargetBlock),
}
impl Instruction {
    pub fn replace_usages(&mut self, map: &HashMap<RegisterID, LeafExpr>) -> bool {
        use Instruction::*;
        match self {
            Nop => false,
            LoadCell(_, e) => e.replace_usage(map),
            StoreCell(d, e) => d.replace_usage(map) | e.replace_usage(map),
            BoundsCheck(l, h) => l.replace_usage(map) | h.replace_usage(map),
            Assign(_, e) => e.replace_usages(map),
            Output(e) => e.replace_usage(map),
            Input(_, e) => e.replace_usage(map),
            Jump(target) => target.replace_usages(map),
            Branch(c, t, e) => c.replace_usage(map) | t.replace_usages(map) | e.replace_usages(map),
        }
    }

    pub(super) fn populate_used(&self, used: &mut HashSet<RegisterID>) {
        use Instruction::*;
        match self {
            Nop => (),
            LoadCell(_, e) => e.populate_used(used),
            StoreCell(d, e) => {
                d.populate_used(used);
                e.populate_used(used);
            }
            BoundsCheck(l, h) => {
                l.populate_used(used);
                h.populate_used(used);
            }
            Assign(_, e) => e.populate_used(used),
            Output(e) => e.populate_used(used),
            Input(_, e) => e.populate_used(used),
            Jump(t) => t.populate_used(used),
            Branch(c, t, e) => {
                c.populate_used(used);
                t.populate_used(used);
                e.populate_used(used)
            }
        }
    }

    pub fn uses(&self, reg: RegisterID) -> bool {
        match self {
            Self::Nop => false,
            Self::LoadCell(_, e) => e.contains(reg),
            Self::StoreCell(d, e) => d.contains(reg) || e.contains(reg),
            Self::BoundsCheck(l, h) => l.contains(reg) || h.contains(reg),
            Self::Assign(_, e) => e.contains(reg),
            Self::Output(e) => e.contains(reg),
            Self::Input(_, e) => e.contains(reg),
            Self::Jump(t) => t.uses(reg),
            Self::Branch(c, t, e) => c.contains(reg) || t.uses(reg) || e.uses(reg),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    UDiv,
    IDiv,
    UMod,
    IMod,

    And,
    Or,
    Xor,
}
impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use BinaryOp::*;
        match *self {
            Add => write!(f, "add"),
            Sub => write!(f, "sub"),
            Mul => write!(f, "mul"),
            UDiv => write!(f, "udiv"),
            IDiv => write!(f, "idiv"),
            UMod => write!(f, "umod"),
            IMod => write!(f, "imod"),
            And => write!(f, "and"),
            Or => write!(f, "or"),
            Xor => write!(f, "xor"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Not,
    Neg,
}
impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use UnaryOp::*;
        match *self {
            Not => write!(f, "not"),
            Neg => write!(f, "neg"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TestOp {
    Equal,
    NotEqual,
}
impl Display for TestOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TestOp::*;
        match *self {
            Equal => write!(f, "teq"),
            NotEqual => write!(f, "tne"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TargetBlock {
    pub id: BlockID,
    pub args: Vec<LeafExpr>,
}
impl TargetBlock {
    pub fn new(id: BlockID, args: Vec<LeafExpr>) -> Self {
        Self { id, args }
    }

    pub(super) fn populate_used(&self, used: &mut HashSet<RegisterID>) {
        for arg in &self.args {
            arg.populate_used(used);
        }
    }

    pub fn uses(&self, reg: RegisterID) -> bool {
        self.args.iter().any(|a| a.contains(reg))
    }

    pub fn replace_usages(&mut self, map: &HashMap<RegisterID, LeafExpr>) -> bool {
        let mut changed = false;
        for arg in &mut self.args {
            changed |= arg.replace_usage(map);
        }
        changed
    }
}
impl From<BlockID> for TargetBlock {
    fn from(value: BlockID) -> Self {
        Self::new(value, Vec::new())
    }
}
impl<T> From<(BlockID, T)> for TargetBlock
where
    T: Into<LeafExpr>,
{
    fn from(value: (BlockID, T)) -> Self {
        Self::new(value.0, vec![value.1.into()])
    }
}
impl Display for TargetBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)?;
        if let Some((last, others)) = self.args.split_last() {
            write!(f, "(")?;
            for other in others {
                write!(f, "{other}")?;
                write!(f, ", ")?;
            }
            write!(f, "{last}")?;
            write!(f, ")")?;
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Expr {
    Leaf(LeafExpr),
    Binary(LeafExpr, BinaryOp, LeafExpr),
    Unary(LeafExpr, UnaryOp),
    Test(LeafExpr, TestOp, LeafExpr),
}
impl Expr {
    pub fn contains(self, reg: RegisterID) -> bool {
        match self {
            Self::Leaf(l) => l.contains(reg),
            Self::Binary(a, _, b) => a.contains(reg) || b.contains(reg),
            Self::Unary(a, _) => a.contains(reg),
            Self::Test(a, _, b) => a.contains(reg) || b.contains(reg),
        }
    }

    pub(super) fn populate_used(self, used: &mut HashSet<RegisterID>) {
        match self {
            Self::Leaf(l) => l.populate_used(used),
            Self::Binary(a, _, b) => {
                a.populate_used(used);
                b.populate_used(used);
            }
            Self::Unary(a, _) => a.populate_used(used),
            Self::Test(a, _, b) => {
                a.populate_used(used);
                b.populate_used(used);
            }
        }
    }

    pub fn is_leaf(self) -> bool {
        matches!(self, Self::Leaf(_))
    }

    pub fn eval_const(self) -> Option<Value> {
        match self {
            Self::Leaf(l) => l.eval_const(),
            Self::Binary(a, op, b) => {
                Some(Value::do_binary_op(a.eval_const()?, b.eval_const()?, op))
            }
            Self::Test(a, op, b) => Some(Value::do_test_op(a.eval_const()?, b.eval_const()?, op)),
            Self::Unary(a, op) => Some(Value::do_unary_op(a.eval_const()?, op)),
        }
    }
    pub fn replace_usages(&mut self, map: &HashMap<RegisterID, LeafExpr>) -> bool {
        use Expr::*;
        match self {
            Leaf(l) => l.replace_usage(map),
            Binary(a, _, b) => a.replace_usage(map) | b.replace_usage(map),
            Unary(a, _) => a.replace_usage(map),
            Test(a, _, b) => a.replace_usage(map) | b.replace_usage(map),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum LeafExpr {
    Register(RegisterID),
    Int(ConstInt),
}
impl LeafExpr {
    pub fn contains(self, reg: RegisterID) -> bool {
        self == LeafExpr::Register(reg)
    }

    pub fn as_register(self) -> Option<RegisterID> {
        let Self::Register(r) = self else { return None };
        Some(r)
    }
    pub(super) fn populate_used(self, used: &mut HashSet<RegisterID>) {
        if let Some(reg) = self.as_register() {
            used.insert(reg);
        }
    }

    pub fn eval_const(self) -> Option<Value> {
        Some(match self {
            LeafExpr::Register(_) => return None,
            LeafExpr::Int(i) => match i {
                ConstInt::Bool(b) => Value::I1(b),
                ConstInt::U8(v) => Value::I8(v),
                ConstInt::I8(v) => Value::I8(u8::from_le_bytes(v.to_le_bytes())),
                ConstInt::U64(v) => Value::I64(v),
                ConstInt::I64(v) => Value::I64(u64::from_le_bytes(v.to_le_bytes())),
            },
        })
    }

    pub fn replace_usage(&mut self, map: &HashMap<RegisterID, LeafExpr>) -> bool {
        if let &mut Self::Register(reg) = self {
            if let Some(&new) = map.get(&reg) {
                *self = new;
            }
        }
        false
    }

    pub fn is_constant_multiplicative_negation(self) -> bool {
        match self {
            Self::Register(_) => false,
            Self::Int(c) => c.is_multiplicative_negation(),
        }
    }
    pub fn is_constant_multiplicative_identity(self) -> bool {
        match self {
            Self::Register(_) => false,
            Self::Int(c) => c.is_multiplicative_identity(),
        }
    }

    pub fn expr_type(&self, module: &Module) -> Type {
        match self {
            &Self::Int(c) => c.int_type(),
            &Self::Register(r) => module[r].register_type(),
        }
    }
}
impl From<RegisterID> for LeafExpr {
    fn from(value: RegisterID) -> Self {
        Self::Register(value)
    }
}
impl From<bool> for LeafExpr {
    fn from(value: bool) -> Self {
        Self::Int(ConstInt::from(value))
    }
}
impl From<i8> for LeafExpr {
    fn from(value: i8) -> Self {
        Self::Int(ConstInt::from(value))
    }
}
impl From<u8> for LeafExpr {
    fn from(value: u8) -> Self {
        Self::Int(ConstInt::from(value))
    }
}
impl From<i64> for LeafExpr {
    fn from(value: i64) -> Self {
        Self::Int(ConstInt::from(value))
    }
}
impl From<u64> for LeafExpr {
    fn from(value: u64) -> Self {
        Self::Int(ConstInt::from(value))
    }
}
impl From<ConstInt> for LeafExpr {
    fn from(value: ConstInt) -> Self {
        Self::Int(value)
    }
}
impl Display for LeafExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &Self::Register(r) => write!(f, "{r}"),
            Self::Int(c) => write!(f, "{c}"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ConstInt {
    Bool(bool),
    I8(i8),
    U8(u8),
    U64(u64),
    I64(i64),
}
impl ConstInt {
    pub fn is_multiplicative_negation(self) -> bool {
        match self {
            Self::Bool(_) => false,
            Self::I8(val) => val == -1,
            Self::U8(val) => val == 255,
            Self::I64(val) => val == -1,
            Self::U64(val) => val == u64::MAX,
        }
    }
    pub fn is_multiplicative_identity(self) -> bool {
        match self {
            Self::Bool(b) => b,
            Self::I8(val) => val == 1,
            Self::U8(val) => val == 1,
            Self::I64(val) => val == 1,
            Self::U64(val) => val == 1,
        }
    }
    pub fn int_type(self) -> Type {
        match self {
            Self::Bool(_) => Type::I1,
            Self::U8(_) | Self::I8(_) => Type::I8,
            Self::U64(_) | Self::I64(_) => Type::I64,
        }
    }
}
impl From<bool> for ConstInt {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}
impl From<i8> for ConstInt {
    fn from(value: i8) -> Self {
        Self::I8(value)
    }
}
impl From<u8> for ConstInt {
    fn from(value: u8) -> Self {
        Self::U8(value)
    }
}
impl From<i64> for ConstInt {
    fn from(value: i64) -> Self {
        Self::I64(value)
    }
}
impl From<u64> for ConstInt {
    fn from(value: u64) -> Self {
        Self::U64(value)
    }
}
impl Display for ConstInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &Self::Bool(v) => write!(f, "{v}"),
            &Self::U8(v) => write!(f, "{v}"),
            &Self::I8(v) => write!(f, "{v}"),
            &Self::U64(v) => write!(f, "{v}"),
            &Self::I64(v) => write!(f, "{v}"),
        }
    }
}
