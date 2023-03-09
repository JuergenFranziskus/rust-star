use super::{
    block::BlockID,
    instruction::{BinaryOp, Instruction, LeafExpr, TargetBlock, TestOp, UnaryOp},
    register::RegisterID,
    types::Type,
    Module,
};
use crate::ir::instruction::Expr;

pub struct Builder<'a> {
    module: &'a mut Module,
    block: BlockID,
}
impl<'a> Builder<'a> {
    pub fn new(module: &'a mut Module, block: BlockID) -> Self {
        Self { module, block }
    }

    pub fn add_block(&mut self) -> BlockID {
        self.module.add_block()
    }
    pub fn add_register(&mut self, reg_type: Type) -> RegisterID {
        self.module.add_register(reg_type)
    }
    pub fn add_parameter(&mut self, param_type: Type) -> RegisterID {
        self.module.add_parameter(self.block, param_type)
    }

    pub fn select_block(&mut self, block: BlockID) {
        self.block = block;
    }
    fn push_instruction(&mut self, i: Instruction) {
        self.module[self.block].add_instruction(i);
    }

    pub fn nop(&mut self) {
        self.push_instruction(Instruction::Nop);
    }

    pub fn load_cell(&mut self, index: impl Into<LeafExpr>) -> RegisterID {
        let target = self.add_register(Type::I8);
        let index = index.into();
        let index_t = index.expr_type(self.module);
        assert_eq!(index_t, Type::I64);
        self.push_instruction(Instruction::LoadCell(target, index));
        target
    }
    pub fn store_cell(&mut self, index: impl Into<LeafExpr>, value: impl Into<LeafExpr>) {
        let index = index.into();
        let value = value.into();

        let index_t = index.expr_type(self.module);
        let value_t = value.expr_type(self.module);
        assert_eq!(index_t, Type::I64);
        assert_eq!(value_t, Type::I8);

        self.push_instruction(Instruction::StoreCell(index, value));
    }
    pub fn check_bounds(&mut self, start: impl Into<LeafExpr>, end: impl Into<LeafExpr>) {
        self.push_instruction(Instruction::BoundsCheck(start.into(), end.into()));
    }

    pub fn set(&mut self, value: impl Into<LeafExpr>) -> RegisterID {
        let value = value.into();
        let value_type = value.expr_type(self.module);
        let target = self.add_register(value_type);
        self.push_instruction(Instruction::Assign(target, Expr::Leaf(value)));
        target
    }
    pub fn binop(
        &mut self,
        op: BinaryOp,
        a: impl Into<LeafExpr>,
        b: impl Into<LeafExpr>,
    ) -> RegisterID {
        let a = a.into();
        let b = b.into();
        let at = a.expr_type(self.module);
        let bt = b.expr_type(self.module);
        assert_eq!(at, bt);
        let target = self.add_register(at);
        self.push_instruction(Instruction::Assign(target, Expr::Binary(a, op, b)));
        target
    }
    pub fn add(&mut self, a: impl Into<LeafExpr>, b: impl Into<LeafExpr>) -> RegisterID {
        self.binop(BinaryOp::Add, a, b)
    }
    pub fn sub(&mut self, a: impl Into<LeafExpr>, b: impl Into<LeafExpr>) -> RegisterID {
        self.binop(BinaryOp::Sub, a, b)
    }
    pub fn mul(&mut self, a: impl Into<LeafExpr>, b: impl Into<LeafExpr>) -> RegisterID {
        self.binop(BinaryOp::Mul, a, b)
    }
    pub fn udiv(&mut self, a: impl Into<LeafExpr>, b: impl Into<LeafExpr>) -> RegisterID {
        self.binop(BinaryOp::UDiv, a, b)
    }
    pub fn idiv(&mut self, a: impl Into<LeafExpr>, b: impl Into<LeafExpr>) -> RegisterID {
        self.binop(BinaryOp::IDiv, a, b)
    }
    pub fn umod(&mut self, a: impl Into<LeafExpr>, b: impl Into<LeafExpr>) -> RegisterID {
        self.binop(BinaryOp::UMod, a, b)
    }
    pub fn imod(&mut self, a: impl Into<LeafExpr>, b: impl Into<LeafExpr>) -> RegisterID {
        self.binop(BinaryOp::IMod, a, b)
    }
    pub fn and(&mut self, a: impl Into<LeafExpr>, b: impl Into<LeafExpr>) -> RegisterID {
        self.binop(BinaryOp::And, a, b)
    }
    pub fn or(&mut self, a: impl Into<LeafExpr>, b: impl Into<LeafExpr>) -> RegisterID {
        self.binop(BinaryOp::Or, a, b)
    }
    pub fn xor(&mut self, a: impl Into<LeafExpr>, b: impl Into<LeafExpr>) -> RegisterID {
        self.binop(BinaryOp::Xor, a, b)
    }

    pub fn unop(&mut self, op: UnaryOp, a: impl Into<LeafExpr>) -> RegisterID {
        let a = a.into();
        let at = a.expr_type(self.module);
        let target = self.add_register(at);
        self.push_instruction(Instruction::Assign(target, Expr::Unary(a, op)));
        target
    }
    pub fn not(&mut self, a: impl Into<LeafExpr>) -> RegisterID {
        self.unop(UnaryOp::Not, a)
    }
    pub fn neg(&mut self, a: impl Into<LeafExpr>) -> RegisterID {
        self.unop(UnaryOp::Neg, a)
    }

    pub fn test(
        &mut self,
        op: TestOp,
        a: impl Into<LeafExpr>,
        b: impl Into<LeafExpr>,
    ) -> RegisterID {
        let a = a.into();
        let b = b.into();
        let at = a.expr_type(self.module);
        let bt = b.expr_type(self.module);
        assert_eq!(at, bt);
        let target = self.add_register(Type::I1);
        self.push_instruction(Instruction::Assign(target, Expr::Test(a, op, b)));
        target
    }

    pub fn output(&mut self, value: impl Into<LeafExpr>) {
        self.push_instruction(Instruction::Output(value.into()));
    }
    pub fn input(&mut self, default: impl Into<LeafExpr>) -> RegisterID {
        let default = default.into();
        let target = self.add_register(Type::I8);
        self.push_instruction(Instruction::Input(target, default));
        target
    }

    pub fn jump(&mut self, target: impl Into<TargetBlock>) {
        self.push_instruction(Instruction::Jump(target.into()));
    }
    pub fn branch(
        &mut self,
        c: impl Into<LeafExpr>,
        then: impl Into<TargetBlock>,
        els: impl Into<TargetBlock>,
    ) {
        let c = c.into();
        let then = then.into();
        let els = els.into();

        let ct = c.expr_type(self.module);
        assert_eq!(ct, Type::I1);
        self.push_instruction(Instruction::Branch(c, then, els));
    }
}
