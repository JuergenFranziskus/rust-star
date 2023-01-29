use std::collections::{HashMap, HashSet};
use cir::{Module, FunctionID, types::Type, builder::Builder, RegisterID, instruction::Expr};
use super::{expr_tree::{Program, CellOffset, Instruction}, optimize::{normalize_pointer_movement, remove_dead, mark_balanced_blocks, merge_verifications, remove_dead_verifications, recog_additions, remove_dead_if_statements}};

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

pub fn gen_code(p: &Program, m: &mut Module) {
    let write = gen_write(m);
    let read = gen_read(m);
    let memset = gen_memset(m);

    let array_type = m.types_mut().make_array(Type::i(8), 30000u64);

    let main = m.add_function("main", Type::i(32));
    let entry = m.add_block(main);
    m[main].set_entry_block(entry);

    let mut b = Builder::new(m, entry);
    let array_var = b.add_variable(array_type);
    let array_ptr = b.build_address_of(array_var);
    let index = b.build_set(0u64);

    b.build_call((memset, array_ptr, 30000u64, 0u8));

    let mut ctx = GenCtx::new(index, array_ptr, write, read);

    for i in &p.0 {
        gen_instruction(i, &mut b, &mut ctx);
    }

    b.build_return(0i32);
}
fn gen_instruction(instruction: &Instruction, b: &mut Builder, ctx: &mut GenCtx) {
    use Instruction::*;
    match instruction {
        &Modify(cell, change) => {
            let old_val = ctx.get_cell_value(b, cell);
            let new_val = b.build_add(old_val, change);
            ctx.set_cell_value(b, cell, new_val);
        }
        &Move(offset) => ctx.move_index(b, offset),
        &Output(cell) => {
            let val = ctx.get_cell_value(b, cell);
            b.build_call((ctx.write, [val.into()]));
        }
        &Input(cell) => {
            ctx.spill_cell(b, cell);
            let ptr = ctx.get_cell_ptr(b, cell);
            b.build_call((ctx.read, [ptr.into()]));
        }
        &Set(cell, val) => ctx.set_cell_value(b, cell, val),
        &AddMultiple { target, base, factor } => {
            let old_target_val = ctx.get_cell_value(b, target);
            let base_val = ctx.get_cell_value(b, base);
            let addend = b.build_mul(base_val, factor);
            let new_target_val = b.build_add(old_target_val, addend);
            ctx.set_cell_value(b, target, new_target_val);
        }
        &Seek(cell, movement) => gen_seek(cell, movement, b, ctx),
        &BoundsCheck(_range) => (),
        &Loop(balanced, base, ref body) => gen_loop(balanced, base, body, b, ctx),
        &If(balanced, base, ref body) => gen_if(balanced, base, body, b, ctx),
    }
}

fn gen_seek(cell: CellOffset, movement: isize, b: &mut Builder, ctx: &mut GenCtx) {
    let loop_body = b.add_block();
    let loop_end = b.add_block();

    ctx.spill_all(b);
    b.build_jump((loop_body, [ctx.index.into()]));

    b.select_block(loop_body);
    ctx.index = b.add_parameter(Type::i(64));
    let cell_val = ctx.get_cell_value(b, cell);
    let found = b.build_test_eq(cell_val, 0u8);
    let next_index = b.build_add(ctx.index, movement as i64);
    ctx.spill_all(b);
    b.build_break(found, loop_end, (loop_body, [next_index.into()]));

    b.select_block(loop_end);
}
fn gen_loop(balanced: bool, base: CellOffset, body: &[Instruction], b: &mut Builder, ctx: &mut GenCtx) {
    if balanced {
        gen_balanced_loop(base, body, b, ctx);
    }
    else {
        gen_unbalanced_loop(base, body, b, ctx);
    }
}
fn gen_balanced_loop(base: CellOffset, body: &[Instruction], b: &mut Builder, ctx: &mut GenCtx) {
    let loop_header = b.add_block();
    let loop_body = b.add_block();
    let loop_end = b.add_block();

    ctx.spill_values(b);
    b.build_jump(loop_header);

    b.select_block(loop_header);
    let val = ctx.get_cell_value(b, base);
    let should_loop = b.build_test_ne(val, 0u8);
    b.build_break(should_loop, loop_body, loop_end);

    b.select_block(loop_body);
    for i in body {
        gen_instruction(i, b, ctx);
    }
    ctx.spill_values(b);
    b.build_jump(loop_header);

    b.select_block(loop_end);
}
fn gen_unbalanced_loop(base: CellOffset, body: &[Instruction], b: &mut Builder, ctx: &mut GenCtx) {
    let loop_header = b.add_block();
    let loop_body = b.add_block();
    let loop_end = b.add_block();

    ctx.spill_all(b);
    b.build_jump((loop_header, [ctx.index.into()]));

    b.select_block(loop_header);
    ctx.index = b.add_parameter(Type::i(64));
    let val = ctx.get_cell_value(b, base);
    let should_loop = b.build_test_ne(val, 0u8);
    b.build_break(should_loop, loop_body, loop_end);

    b.select_block(loop_body);
    for i in body {
        gen_instruction(i, b, ctx);
    }
    ctx.spill_all(b);
    b.build_jump((loop_header, [ctx.index.into()]));

    b.select_block(loop_end);
}
fn gen_if(balanced: bool, base: CellOffset, body: &[Instruction], b: &mut Builder, ctx: &mut GenCtx) {
    if balanced {
        gen_balanced_if(base, body, b, ctx);
    }
    else {
        gen_unbalanced_if(base, body, b, ctx);
    }
}
fn gen_balanced_if(base: CellOffset, body: &[Instruction], b: &mut Builder, ctx: &mut GenCtx) {
    let then_block = b.add_block();
    let end_block = b.add_block();

    let val = ctx.get_cell_value(b, base);
    let should_take = b.build_test_ne(val, 0u8);
    ctx.spill_values(b);
    b.build_break(should_take, then_block, end_block);

    b.select_block(then_block);
    for i in body {
        gen_instruction(i, b, ctx);
    }
    ctx.spill_values(b);
    b.build_jump(end_block);

    b.select_block(end_block);
}
fn gen_unbalanced_if(base: CellOffset, body: &[Instruction], b: &mut Builder, ctx: &mut GenCtx) {
    let then_block = b.add_block();
    let end_block = b.add_block();

    let val = ctx.get_cell_value(b, base);
    let should_take = b.build_test_ne(val, 0u8);
    ctx.spill_all(b);
    b.build_break(should_take, then_block, (end_block, [ctx.index.into()]));

    b.select_block(then_block);
    for i in body {
        gen_instruction(i, b, ctx);
    }
    ctx.spill_all(b);
    b.build_jump((end_block, [ctx.index.into()]));

    b.select_block(end_block);
    ctx.index = b.add_parameter(Type::i(64));
}


struct GenCtx {
    index: RegisterID,
    array_ptr: RegisterID,
    write: FunctionID,
    read: FunctionID,

    cached_pointers: HashMap<CellOffset, RegisterID>,
    cached_values: HashMap<CellOffset, RegisterID>,
    written: HashSet<CellOffset>,
}
impl GenCtx {
    fn new(index: RegisterID, array_ptr: RegisterID, write: FunctionID, read: FunctionID) -> Self {
        Self {
            index,
            array_ptr,
            write,
            read,
            cached_pointers: HashMap::new(),
            cached_values: HashMap::new(),
            written: HashSet::new(),
        }
    }


    fn move_index(&mut self, b: &mut Builder, offset: isize) {
        self.index = b.build_add(self.index, offset as i64);
        self.cached_pointers = self.cached_pointers.drain()
            .map(|(k, v)| (k.wrapping_sub(offset), v))
            .collect();
        self.cached_values = self.cached_values.drain()
            .map(|(k, v)| (k.wrapping_sub(offset), v))
            .collect();
    }
    fn get_cell_ptr(&mut self, b: &mut Builder, cell: CellOffset) -> RegisterID {
        if let Some(&reg) = self.cached_pointers.get(&cell) {
            reg
        }
        else {
            let cell_index = b.build_add(self.index, cell as i64);
            let cell_ptr = b.build_single_gep(self.array_ptr, cell_index, Type::i(8));
            self.cached_pointers.insert(cell, cell_ptr);
            cell_ptr
        }
    }
    fn get_cell_value(&mut self, b: &mut Builder, cell: CellOffset) -> RegisterID {
        if let Some(&val) = self.cached_values.get(&cell) {
            val
        }
        else {
            let ptr = self.get_cell_ptr(b, cell);
            let val = b.build_load(ptr, Type::i(8));
            self.cached_values.insert(cell, val);
            val
        }
    }
    fn set_cell_value(&mut self, b: &mut Builder, cell: CellOffset, val: impl Into<Expr>) {
        let val = val.into();
        let reg = if let Expr::Register(reg) = val {
            reg
        }
        else {
            b.build_set(val)
        };
        self.cached_values.insert(cell, reg);
        self.written.insert(cell);
    }
    fn spill_values(&mut self, b: &mut Builder) {
        // We have to remove the hashmap from the struct so we can borrow it mutably.
        let mut value_map = std::mem::replace(&mut self.cached_values, Default::default());
        for (k, v) in value_map.drain() {
            if self.written.contains(&k) {
                let ptr = self.get_cell_ptr(b, k);
                b.build_store(ptr, v);
            }
        }
        // This way, the old capacity can be reused and we don't need to allocate anything.
        self.cached_values = value_map;
        self.written.clear();
    }
    fn spill_cell(&mut self, b: &mut Builder, cell: CellOffset) {
        if let Some(reg) = self.cached_values.remove(&cell) {
            if self.written.contains(&cell) {
                let ptr = self.get_cell_ptr(b, cell);
                b.build_store(ptr, reg);
                self.written.remove(&cell);
            }
        }
    }
    fn clear_pointers(&mut self) {
        self.cached_pointers.clear();
    }
    fn spill_all(&mut self, b: &mut Builder) {
        self.spill_values(b);
        self.clear_pointers();
    }
}


const STDIN: u64 = 0;
const STDOUT: u64 = 1;
const SYS_READ: u64 = 0;
const SYS_WRITE: u64 = 1;


fn gen_write(m: &mut Module) -> FunctionID {
    let fid = m.add_function("write_cell", Type::Void);
    let entry = m.add_block(fid);
    m[fid].set_entry_block(entry);

    let mut b = Builder::new(m, entry);
    let param_value = b.add_parameter(Type::i(8));
    let buffer_var = b.add_variable(Type::i(8));
    let buffer_ptr = b.build_address_of(buffer_var);
    b.build_store(buffer_ptr, param_value);
    b.build_linux_64_syscall((SYS_WRITE.into(), STDOUT, buffer_ptr, 1u64));
    b.build_return_void();

    fid
}
fn gen_read(m: &mut Module) -> FunctionID {
    let fid = m.add_function("read_cell", Type::i(8));
    let entry = m.add_block(fid);
    m[fid].set_entry_block(entry);

    let mut b = Builder::new(m, entry);
    let param_cell_ptr = b.add_parameter(Type::Pointer);
    b.build_linux_64_syscall((SYS_READ.into(), STDIN, param_cell_ptr, 1u64));
    b.build_return_void();

    fid
}
fn gen_memset(m: &mut Module) -> FunctionID {
    let fid = m.add_function("memset", Type::Void);
    let entry = m.add_block(fid);
    m[fid].set_entry_block(entry);

    let loop_header = m.add_block(fid);
    let loop_body = m.add_block(fid);
    let loop_end = m.add_block(fid);

    let mut b = Builder::new(m, entry);
    let param_ptr = b.add_parameter(Type::Pointer);
    let param_size = b.add_parameter(Type::i(64));
    let param_value = b.add_parameter(Type::i(8));
    b.build_jump((loop_header, [0u64.into()]));
    
    b.select_block(loop_header);
    let index = b.add_parameter(Type::i(64));
    let should_loop = b.build_test_ult(index, param_size);
    b.build_break(should_loop, loop_body, loop_end);

    b.select_block(loop_body);
    let val_ptr = b.build_single_gep(param_ptr, index, Type::i(8));
    b.build_store(val_ptr, param_value);
    let new_index = b.build_unsigned_add(index, 0u64);
    b.build_jump((loop_header, [new_index.into()]));

    b.select_block(loop_end);
    b.build_return_void();


    fid
}
