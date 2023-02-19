use super::{block::BlockID, register::RegisterID, types::Type, Module};
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Instruction {
    Nop,

    LoadCell(RegisterID, Expr),
    StoreCell(Expr, Expr),
    BoundsCheck(Expr, Expr),

    Set(RegisterID, Expr),
    Binary(BinaryOp, RegisterID, Expr, Expr),
    Unary(UnaryOp, RegisterID, Expr),
    Test(TestOp, RegisterID, Expr, Expr),

    Output(Expr),
    Input(RegisterID, Expr),

    Jump(TargetBlock),
    Branch(Expr, TargetBlock, TargetBlock),
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
    pub args: Vec<Expr>,
}
impl TargetBlock {
    pub fn new(id: BlockID, args: Vec<Expr>) -> Self {
        Self { id, args }
    }
}
impl From<BlockID> for TargetBlock {
    fn from(value: BlockID) -> Self {
        Self::new(value, Vec::new())
    }
}
impl<T> From<(BlockID, T)> for TargetBlock
where
    T: Into<Expr>,
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
    Register(RegisterID),
    Int(ConstInt),
}
impl Expr {
    pub fn expr_type(&self, module: &Module) -> Type {
        match self {
            &Self::Int(c) => c.int_type(),
            &Self::Register(r) => module[r].register_type(),
        }
    }
}
impl From<RegisterID> for Expr {
    fn from(value: RegisterID) -> Self {
        Self::Register(value)
    }
}
impl From<bool> for Expr {
    fn from(value: bool) -> Self {
        Self::Int(ConstInt::from(value))
    }
}
impl From<i8> for Expr {
    fn from(value: i8) -> Self {
        Self::Int(ConstInt::from(value))
    }
}
impl From<u8> for Expr {
    fn from(value: u8) -> Self {
        Self::Int(ConstInt::from(value))
    }
}
impl From<i64> for Expr {
    fn from(value: i64) -> Self {
        Self::Int(ConstInt::from(value))
    }
}
impl From<u64> for Expr {
    fn from(value: u64) -> Self {
        Self::Int(ConstInt::from(value))
    }
}
impl From<ConstInt> for Expr {
    fn from(value: ConstInt) -> Self {
        Self::Int(value)
    }
}
impl Display for Expr {
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
