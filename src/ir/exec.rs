use super::{
    block::{Block, BlockID},
    instruction::{BinaryOp, Expr, LeafExpr, TargetBlock, TestOp, UnaryOp},
    register::RegisterID,
    Module,
};
use crate::ir::instruction::Instruction;
use std::{
    io::{self, Read, Write},
    iter::once,
    ops::{Index, IndexMut},
};

pub struct Exec<O, I> {
    cells: Vec<u8>,
    registers: Vec<Value>,
    stdout: O,
    stdin: I,
}
impl<O: Write, I: Read> Exec<O, I> {
    pub fn new(stdout: O, stdin: I) -> Self {
        Self {
            cells: Vec::new(),
            registers: Vec::new(),
            stdout,
            stdin,
        }
    }

    pub fn exec_program(&mut self, module: &Module) -> io::Result<()> {
        let registers = module.registers.len();
        self.registers.clear();
        self.registers
            .extend(once(Value::Uninit).cycle().take(registers));

        let entry = module.entry_block();
        let mut action = Action::Jump(entry, Vec::new());

        loop {
            match action {
                Action::Halt => break,
                Action::Jump(block, args) => action = self.exec_block(&module[block], args)?,
            }
        }

        Ok(())
    }

    fn exec_block(&mut self, block: &Block, args: Vec<Value>) -> io::Result<Action> {
        for (&param, arg) in block.parameters().into_iter().zip(args) {
            self[param] = arg;
        }
        let _id = block.id();
        for (_i, instruction) in block.body().into_iter().enumerate() {
            use Instruction::*;
            match instruction {
                &Nop => (),
                &LoadCell(target, ref index) => self.load_cell(target, index),
                StoreCell(index, value) => self.store_cell(index, value),
                BoundsCheck(start, end) => self.bounds_check(start, end),
                &Assign(target, ref expr) => self.assign(target, expr),
                &Output(ref value) => self.output(value)?,
                &Input(target, ref default) => self.input(target, default)?,
                Jump(target) => return Ok(self.jump(target)),
                Branch(c, then, els) => return Ok(self.branch(c, then, els)),
            }
        }

        Ok(Action::Halt)
    }

    fn load_cell(&mut self, target: RegisterID, index: &LeafExpr) {
        let Value::I64(index) = self.eval_leaf_expr(index) else { panic!("{index} is not of type i64") };
        let cell = self.cells[index as usize];
        self[target] = Value::I8(cell);
    }
    fn store_cell(&mut self, index: &LeafExpr, value: &LeafExpr) {
        let Value::I64(index) = self.eval_leaf_expr(index) else { panic!() };
        let Value::I8(value) = self.eval_leaf_expr(value) else { panic!() };
        self.cells[index as usize] = value;
    }
    fn bounds_check(&mut self, _start: &LeafExpr, end: &LeafExpr) {
        let Value::I64(end) = self.eval_leaf_expr(end) else { panic!() };
        let needs_length = end as usize;
        let has_length = self.cells.len();
        if needs_length > has_length {
            let difference = needs_length - has_length;
            //eprintln!("Padding the cell vector by {} to length {}", difference, needs_length);
            self.cells.extend(once(0).cycle().take(difference));
            //eprintln!("Cells now have length {}", self.cells.len());
        }
    }

    fn assign(&mut self, target: RegisterID, expr: &Expr) {
        let value = self.eval_expr(expr);
        self[target] = value;
    }
    fn eval_expr(&self, expr: &Expr) -> Value {
        match expr {
            Expr::Leaf(e) => self.eval_leaf_expr(e),
            &Expr::Binary(ref a, op, ref b) => self.binary_op(op, a, b),
            &Expr::Unary(ref a, op) => self.unary_op(op, a),
            &Expr::Test(ref a, op, ref b) => self.test_op(op, a, b),
        }
    }

    fn binary_op(&self, op: BinaryOp, a: &LeafExpr, b: &LeafExpr) -> Value {
        let a = self.eval_leaf_expr(a);
        let b = self.eval_leaf_expr(b);

        Value::do_binary_op(a, b, op)
    }

    fn unary_op(&self, op: UnaryOp, a: &LeafExpr) -> Value {
        let a = self.eval_leaf_expr(a);
        Value::do_unary_op(a, op)
    }

    fn test_op(&self, op: TestOp, a: &LeafExpr, b: &LeafExpr) -> Value {
        let a = self.eval_leaf_expr(a);
        let b = self.eval_leaf_expr(b);

        Value::do_test_op(a, b, op)
    }

    fn output(&mut self, value: &LeafExpr) -> io::Result<()> {
        let Value::I8(value) = self.eval_leaf_expr(value) else { panic!() };
        self.stdout.write(&[value])?;
        self.stdout.flush()?;
        Ok(())
    }
    fn input(&mut self, target: RegisterID, default: &LeafExpr) -> io::Result<()> {
        let Value::I8(default) = self.eval_leaf_expr(default) else { panic!() };
        let mut buffer = [0];
        let read = self.stdin.read(&mut buffer)?;
        let result = if read == 0 { default } else { buffer[0] };
        self[target] = Value::I8(result);
        Ok(())
    }

    fn jump(&mut self, target: &TargetBlock) -> Action {
        let id = target.id;
        let args = target.args.iter().map(|a| self.eval_leaf_expr(a)).collect();
        Action::Jump(id, args)
    }
    fn branch(&mut self, c: &LeafExpr, then: &TargetBlock, els: &TargetBlock) -> Action {
        let Value::I1(c) = self.eval_leaf_expr(c) else { panic!() };
        if c {
            self.jump(then)
        } else {
            self.jump(els)
        }
    }

    fn eval_leaf_expr(&self, expr: &LeafExpr) -> Value {
        match expr {
            &LeafExpr::Register(r) => self[r],
            &LeafExpr::Int(_) => expr.eval_const().unwrap(),
        }
    }
}
impl<O, I> Index<RegisterID> for Exec<O, I> {
    type Output = Value;

    fn index(&self, index: RegisterID) -> &Self::Output {
        &self.registers[index.0]
    }
}
impl<O, I> IndexMut<RegisterID> for Exec<O, I> {
    fn index_mut(&mut self, index: RegisterID) -> &mut Self::Output {
        &mut self.registers[index.0]
    }
}

enum Action {
    Halt,
    Jump(BlockID, Vec<Value>),
}

#[derive(Copy, Clone, Debug)]
pub enum Value {
    Uninit,
    I1(bool),
    I8(u8),
    I64(u64),
}
impl Value {
    pub fn do_binary_op(a: Value, b: Value, op: BinaryOp) -> Value {
        use BinaryOp::*;
        match op {
            Add => Self::add(a, b),
            Sub => Self::sub(a, b),
            Mul => Self::mul(a, b),
            UDiv => Self::udiv(a, b),
            IDiv => Self::idiv(a, b),
            UMod => Self::umod(a, b),
            IMod => Self::imod(a, b),

            And => Self::and(a, b),
            Or => Self::or(a, b),
            Xor => Self::xor(a, b),
        }
    }
    fn add(a: Value, b: Value) -> Value {
        use Value::*;
        match (a, b) {
            (I1(a), I1(b)) => I1(a ^ b),
            (I8(a), I8(b)) => I8(a.wrapping_add(b)),
            (I64(a), I64(b)) => I64(a.wrapping_add(b)),
            _ => panic!("Cannot add values {a:?} and {b:?}"),
        }
    }
    fn sub(a: Value, b: Value) -> Value {
        use Value::*;
        match (a, b) {
            (I1(a), I1(b)) => I1(a ^ b),
            (I8(a), I8(b)) => I8(a.wrapping_sub(b)),
            (I64(a), I64(b)) => I64(a.wrapping_sub(b)),
            _ => panic!(),
        }
    }
    fn mul(a: Value, b: Value) -> Value {
        use Value::*;
        match (a, b) {
            (I1(a), I1(b)) => I1(a && b),
            (I8(a), I8(b)) => I8(a.wrapping_mul(b)),
            (I64(a), I64(b)) => I64(a.wrapping_mul(b)),
            _ => panic!(),
        }
    }
    fn udiv(a: Value, b: Value) -> Value {
        use Value::*;
        match (a, b) {
            (I1(a), I1(_)) => I1(a), // b being false is undefined, doesn't happen. a / 1 = a
            (I8(a), I8(b)) => I8(a.wrapping_div(b)),
            (I64(a), I64(b)) => I64(a.wrapping_div(b)),
            _ => panic!(),
        }
    }
    fn idiv(a: Value, b: Value) -> Value {
        use Value::*;
        match (a, b) {
            (I1(_), I1(_)) => panic!(), // I don't wanna work out some logic for this one, it's dumb, undefined
            (I8(a), I8(b)) => {
                let a = i8::from_le_bytes(a.to_le_bytes());
                let b = i8::from_le_bytes(b.to_le_bytes());
                let result = a.wrapping_div(b);
                I8(u8::from_le_bytes(result.to_le_bytes()))
            }
            (I64(a), I64(b)) => {
                let a = i64::from_le_bytes(a.to_le_bytes());
                let b = i64::from_le_bytes(b.to_le_bytes());
                let result = a.wrapping_div(b);
                I64(u64::from_le_bytes(result.to_le_bytes()))
            }
            _ => panic!(),
        }
    }
    fn umod(a: Value, b: Value) -> Value {
        use Value::*;
        match (a, b) {
            (I1(_), I1(_)) => I1(false), // Dividing by 0 is undefined, doesn't happen. a % 1 = 0
            (I8(a), I8(b)) => I8(a.wrapping_rem(b)),
            (I64(a), I64(b)) => I64(a.wrapping_rem(b)),
            _ => panic!(),
        }
    }
    fn imod(a: Value, b: Value) -> Value {
        use Value::*;
        match (a, b) {
            (I1(_), I1(_)) => panic!(), // I don't wanna work out some logic for this one, it's dumb, undefined
            (I8(a), I8(b)) => {
                let a = i8::from_le_bytes(a.to_le_bytes());
                let b = i8::from_le_bytes(b.to_le_bytes());
                let result = a.wrapping_rem(b);
                I8(u8::from_le_bytes(result.to_le_bytes()))
            }
            (I64(a), I64(b)) => {
                let a = i64::from_le_bytes(a.to_le_bytes());
                let b = i64::from_le_bytes(b.to_le_bytes());
                let result = a.wrapping_rem(b);
                I64(u64::from_le_bytes(result.to_le_bytes()))
            }
            _ => panic!(),
        }
    }
    fn and(a: Value, b: Value) -> Value {
        use Value::*;
        match (a, b) {
            (I1(a), I1(b)) => I1(a & b),
            (I8(a), I8(b)) => I8(a & b),
            (I64(a), I64(b)) => I64(a & b),
            _ => panic!(),
        }
    }
    fn or(a: Value, b: Value) -> Value {
        use Value::*;
        match (a, b) {
            (I1(a), I1(b)) => I1(a | b),
            (I8(a), I8(b)) => I8(a | b),
            (I64(a), I64(b)) => I64(a | b),
            _ => panic!(),
        }
    }
    fn xor(a: Value, b: Value) -> Value {
        use Value::*;
        match (a, b) {
            (I1(a), I1(b)) => I1(a ^ b),
            (I8(a), I8(b)) => I8(a ^ b),
            (I64(a), I64(b)) => I64(a ^ b),
            _ => panic!(),
        }
    }

    pub fn do_unary_op(a: Value, op: UnaryOp) -> Value {
        use UnaryOp::*;
        match op {
            Not => Self::not(a),
            Neg => Self::neg(a),
        }
    }
    fn not(a: Value) -> Value {
        use Value::*;
        match a {
            Uninit => panic!(),
            I1(a) => I1(!a),
            I8(a) => I8(!a),
            I64(a) => I64(!a),
        }
    }
    fn neg(a: Value) -> Value {
        use Value::*;
        match a {
            Uninit => panic!(),
            I1(a) => I1(a),
            I8(a) => I8((!a).wrapping_add(1)),
            I64(a) => I64((!a).wrapping_add(1)),
        }
    }

    pub fn do_test_op(a: Value, b: Value, op: TestOp) -> Value {
        use TestOp::*;
        match op {
            Equal => Self::test_equal(a, b),
            NotEqual => Self::test_not_equal(a, b),
        }
    }
    fn test_equal(a: Value, b: Value) -> Value {
        use Value::*;
        match (a, b) {
            (I1(a), I1(b)) => I1(a == b),
            (I8(a), I8(b)) => I1(a == b),
            (I64(a), I64(b)) => I1(a == b),
            _ => panic!(),
        }
    }
    fn test_not_equal(a: Value, b: Value) -> Value {
        use Value::*;
        match (a, b) {
            (I1(a), I1(b)) => I1(a != b),
            (I8(a), I8(b)) => I1(a != b),
            (I64(a), I64(b)) => I1(a != b),
            _ => panic!(),
        }
    }

    pub fn to_leaf_expr(self) -> LeafExpr {
        match self {
            Self::Uninit => panic!(),
            Self::I1(val) => LeafExpr::Int(val.into()),
            Self::I8(val) => LeafExpr::Int(val.into()),
            Self::I64(val) => LeafExpr::Int(val.into()),
        }
    }
}
