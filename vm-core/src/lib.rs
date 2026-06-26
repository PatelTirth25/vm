#![no_std]

pub mod flags;
pub mod instruction;

pub use flags::VmFlags;
use instruction::Opcode;

pub trait Host {
    fn print(&self, value: i32);
    fn print_char(&self, _c: u8) {}
    fn native_call(&self, id: u8, args: &[i32]) -> i32;
    fn report_flags(&self, flags: VmFlags);
}

pub const HEAP_SIZE: usize = 256;

pub struct VM<'a, H: Host> {
    bytecode: &'a [u8],
    pc: usize,

    stack: [i32; 32],
    sp: usize,

    memory: [i32; 32],

    heap: [i32; HEAP_SIZE],
    hp: usize,

    flags: VmFlags,
    host: H,

    call_stack: [usize; 32],
    csp: usize,
}

impl<'a, H: Host> VM<'a, H> {
    pub const fn new(bytecode: &'a [u8], host: H) -> Self {
        Self {
            bytecode,
            pc: 0,
            stack: [0; 32],
            sp: 0,
            memory: [0; 32],
            heap: [0; HEAP_SIZE],
            hp: 0,
            flags: VmFlags::new(),
            host,
            call_stack: [0; 32],
            csp: 0,
        }
    }

    fn push(&mut self, value: i32) {
        if self.sp >= self.stack.len() {
            self.flags.stack_overflow = true;
            self.flags.halted = true;
            return;
        }
        self.stack[self.sp] = value;
        self.sp += 1;
    }

    fn pop(&mut self) -> i32 {
        if self.sp == 0 {
            self.flags.stack_underflow = true;
            self.flags.halted = true;
            return 0;
        }
        self.sp -= 1;
        self.stack[self.sp]
    }

    fn fetch(&mut self) -> u8 {
        if self.pc >= self.bytecode.len() {
            self.flags.invalid_opcode = true;
            self.flags.halted = true;
            return 0;
        }

        let b = self.bytecode[self.pc];
        self.pc += 1;
        b
    }

    fn jump_to(&mut self, addr: usize) {
        if addr >= self.bytecode.len() {
            self.flags.invalid_jump = true;
            self.flags.halted = true;
            return;
        }
        self.pc = addr;
    }

    pub fn run(&mut self) {
        while !self.flags.halted {
            let byte = self.fetch();
            let Some(opcode) = Opcode::from_byte(byte) else {
                self.flags.invalid_opcode = true;
                break;
            };

            match opcode {
                Opcode::Push => {
                    let value = self.fetch() as i32;
                    self.push(value);
                }

                Opcode::Add => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a + b);
                }

                Opcode::Sub => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a - b);
                }

                Opcode::Mul => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a * b);
                }

                Opcode::Div => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a / b);
                }

                Opcode::ArrAlloc => {
                    let size = self.pop() as usize;
                    if self.hp + size + 1 > HEAP_SIZE {
                        self.flags.heap_oob = true;
                        self.flags.halted = true;
                        break;
                    }
                    let handle = self.hp as i32;
                    self.heap[self.hp] = size as i32;
                    self.hp += 1;
                    for i in 0..size {
                        self.heap[self.hp + i] = 0;
                    }
                    self.hp += size;
                    self.push(handle);
                }

                Opcode::ArrLen => {
                    let handle = self.pop() as usize;
                    if handle >= HEAP_SIZE {
                        self.flags.heap_oob = true;
                        self.flags.halted = true;
                        break;
                    }
                    self.push(self.heap[handle]);
                }

                Opcode::ArrLit => {
                    let count = self.fetch() as usize;
                    if self.hp + count + 1 > HEAP_SIZE {
                        self.flags.heap_oob = true;
                        self.flags.halted = true;
                        break;
                    }
                    let handle = self.hp as i32;
                    self.heap[self.hp] = count as i32;
                    self.hp += 1;
                    // Pop values from stack into heap[handle+1..]
                    // Elements pushed in reverse (eN..e1), so popping gives e1..eN
                    for i in 0..count {
                        let val = self.pop();
                        self.heap[self.hp + i] = val;
                    }
                    self.hp += count;
                    self.push(handle);
                }

                Opcode::Load => {
                    let idx = self.fetch() as usize;

                    if idx >= self.memory.len() {
                        self.flags.memory_oob = true;
                        self.flags.halted = true;
                        break;
                    }

                    let value = self.memory[idx];
                    self.push(value);
                }

                Opcode::Store => {
                    let idx = self.fetch() as usize;

                    if idx >= self.memory.len() {
                        self.flags.memory_oob = true;
                        self.flags.halted = true;
                        break;
                    }

                    let value = self.pop();
                    self.memory[idx] = value;
                }

                Opcode::ArrLoad => {
                    let index = self.pop() as usize;
                    let handle = self.pop() as usize;
                    if handle >= HEAP_SIZE {
                        self.flags.heap_oob = true;
                        self.flags.halted = true;
                        break;
                    }
                    let len = self.heap[handle] as usize;
                    if index >= len {
                        self.flags.heap_oob = true;
                        self.flags.halted = true;
                        break;
                    }
                    let value = self.heap[handle + 1 + index];
                    self.push(value);
                }

                Opcode::ArrStore => {
                    let value = self.pop();
                    let index = self.pop() as usize;
                    let handle = self.pop() as usize;
                    if handle >= HEAP_SIZE {
                        self.flags.heap_oob = true;
                        self.flags.halted = true;
                        break;
                    }
                    let len = self.heap[handle] as usize;
                    if index >= len {
                        self.flags.heap_oob = true;
                        self.flags.halted = true;
                        break;
                    }
                    self.heap[handle + 1 + index] = value;
                }

                Opcode::CmpEq => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push((a == b) as i32);
                }

                Opcode::CmpLt => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push((a < b) as i32);
                }

                Opcode::CmpGt => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push((a > b) as i32);
                }

                Opcode::CmpLe => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push((a <= b) as i32);
                }

                Opcode::CmpGe => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push((a >= b) as i32);
                }

                Opcode::Jmp => {
                    let addr = self.fetch() as usize;
                    self.jump_to(addr);
                }

                Opcode::JmpIfFalse => {
                    let addr = self.fetch() as usize;
                    let cond = self.pop();

                    if cond == 0 {
                        self.jump_to(addr);
                    }
                }

                Opcode::CallNative => {
                    let count = self.fetch();
                    let id = self.fetch();
                    let mut args = [0i32; 8];
                    for i in 0..count {
                        args[(count - 1 - i) as usize] = self.pop();
                    }
                    let result = if id == 4 && count >= 3 {
                        // substring(handle, start, length) — built-in
                        let handle = args[0] as usize;
                        let start = args[1] as usize;
                        let length = args[2] as usize;
                        if handle >= HEAP_SIZE || start + length > self.heap[handle] as usize {
                            self.flags.heap_oob = true;
                            self.flags.halted = true;
                            break;
                        }
                        if self.hp + length + 1 > HEAP_SIZE {
                            self.flags.heap_oob = true;
                            self.flags.halted = true;
                            break;
                        }
                        let new_handle = self.hp as i32;
                        self.heap[self.hp] = length as i32;
                        self.hp += 1;
                        for i in 0..length {
                            self.heap[self.hp + i] = self.heap[handle + 1 + start + i];
                        }
                        self.hp += length;
                        new_handle
                    } else {
                        self.host.native_call(id, &args[..count as usize])
                    };
                    self.push(result);
                }

                Opcode::Print => {
                    let v = self.pop();
                    self.host.print(v);
                }

                Opcode::PrintStr => {
                    let handle = self.pop() as usize;
                    if handle >= HEAP_SIZE {
                        self.flags.heap_oob = true;
                        self.flags.halted = true;
                        break;
                    }
                    let len = self.heap[handle] as usize;
                    for i in 0..len {
                        let ch = self.heap[handle + 1 + i] as u8;
                        self.host.print_char(ch);
                    }
                }

                Opcode::StrCmp => {
                    let right_addr = self.pop() as usize;
                    let left_addr = self.pop() as usize;
                    if left_addr >= HEAP_SIZE || right_addr >= HEAP_SIZE {
                        self.flags.heap_oob = true;
                        self.flags.halted = true;
                        break;
                    }
                    let left_len = self.heap[left_addr] as usize;
                    let right_len = self.heap[right_addr] as usize;
                    let min_len = if left_len < right_len { left_len } else { right_len };
                    let mut result = 0i32;
                    for i in 0..min_len {
                        let lc = self.heap[left_addr + 1 + i];
                        let rc = self.heap[right_addr + 1 + i];
                        if lc < rc { result = -1; break; }
                        if lc > rc { result = 1; break; }
                    }
                    if result == 0 {
                        if left_len < right_len { result = -1; }
                        else if left_len > right_len { result = 1; }
                    }
                    self.push(result);
                }

                Opcode::StrConcat => {
                    let right_addr = self.pop() as usize;
                    let left_addr = self.pop() as usize;
                    if left_addr >= HEAP_SIZE || right_addr >= HEAP_SIZE {
                        self.flags.heap_oob = true;
                        self.flags.halted = true;
                        break;
                    }
                    let left_len = self.heap[left_addr] as usize;
                    let right_len = self.heap[right_addr] as usize;
                    let new_len = left_len + right_len;
                    if self.hp + new_len + 1 > HEAP_SIZE {
                        self.flags.heap_oob = true;
                        self.flags.halted = true;
                        break;
                    }
                    let new_addr = self.hp;
                    self.heap[new_addr] = new_len as i32;
                    self.hp += 1;
                    for i in 0..left_len {
                        self.heap[self.hp + i] = self.heap[left_addr + 1 + i];
                    }
                    for i in 0..right_len {
                        self.heap[self.hp + left_len + i] = self.heap[right_addr + 1 + i];
                    }
                    self.hp += new_len;
                    self.push(new_addr as i32);
                }

                Opcode::Call => {
                    let addr = self.fetch() as usize;

                    if addr >= self.bytecode.len() {
                        self.flags.invalid_jump = true;
                        self.flags.halted = true;
                        break;
                    }

                    if self.csp >= self.call_stack.len() {
                        self.flags.stack_overflow = true;
                        self.flags.halted = true;
                        break;
                    }

                    // Save return address
                    self.call_stack[self.csp] = self.pc;
                    self.csp += 1;

                    // Jump to function
                    self.pc = addr;
                }

                Opcode::Ret => {
                    if self.csp == 0 {
                        self.flags.stack_underflow = true;
                        self.flags.halted = true;
                        break;
                    }

                    self.csp -= 1;

                    // Restore return address
                    self.pc = self.call_stack[self.csp];
                }

                Opcode::Halt => {
                    self.flags.halted = true;
                }
            }
        }

        self.host.report_flags(self.flags);
    }
}
