use super::{types::Type, Block, Function, Module, Register, Variable};
use crate::util::arena::ID;

pub enum Instruction {
    AddressOf(ID<Register>, ID<Variable>),
    GetArrayElementPointer {
        target: ID<Register>,
        pointer: Expr,
        index: Expr,
        element_type: Type,
    },

    Call(ID<Function>, Vec<Expr>),
    Return(Option<Expr>),

    Set {
        target: ID<Register>,
        value: Expr,
    },
    Load {
        target: ID<Register>,
        pointer: Expr,
    },
    Store {
        pointer: Expr,
        value: Expr,
    },

    Add(ID<Register>, Expr, Expr),
    Sub(ID<Register>, Expr, Expr),
    Mul(ID<Register>, Expr, Expr),

    Jump(ID<Block>),
    Branch(Expr, ID<Block>, ID<Block>),

    TestNotZero(ID<Register>, Expr),
    TestLessThan(ID<Register>, Expr, Expr),

    Linux64Syscall {
        target: ID<Register>,
        syscall_number: Expr,
        args: SyscallArgs,
    },
}

pub enum Expr {
    Register(ID<Register>),
    I64(i64),
    U64(u64),
    I32(i32),
    U32(u32),
    I16(i16),
    U16(u16),
    I8(i8),
    U8(u8),
    I1(bool),
}
impl Expr {
    pub fn expr_type(&self, m: &Module) -> Type {
        match self {
            Self::I1(_) => Type::I1,
            Self::U8(_) | Self::I8(_) => Type::I8,
            Self::U16(_) | Self::I16(_) => Type::I16,
            Self::U32(_) | Self::I32(_) => Type::I32,
            Self::U64(_) | Self::I64(_) => Type::I64,
            Self::Register(reg) => m[*reg].reg_type(),
        }
    }
}
impl From<ID<Register>> for Expr {
    fn from(value: ID<Register>) -> Self {
        Self::Register(value)
    }
}

pub enum SyscallArgs {
    One(Expr),
    Two(Expr, Expr),
    Three(Expr, Expr, Expr),
    Four(Expr, Expr, Expr, Expr),
    Five(Expr, Expr, Expr, Expr, Expr),
    Six(Expr, Expr, Expr, Expr, Expr, Expr),
}
