#![no_std]

pub mod flags;
pub mod instruction;

pub use flags::VmFlags;
use instruction::Opcode;

pub trait Host {
    fn print(&self, value: i32);
    fn report_flags(&self, flags: VmFlags);
}

pub struct VM<'a, H: Host> {
    bytecode: &'a [u8],
    pc: usize,

    stack: [i32; 32],
    sp: usize,

    memory: [i32; 32],

    flags: VmFlags,
    host: H,
}

impl<'a, H: Host> VM<'a, H> {
    pub const fn new(bytecode: &'a [u8], host: H) -> Self {
        Self {
            bytecode,
            pc: 0,
            stack: [0; 32],
            sp: 0,
            memory: [0; 32],
            flags: VmFlags::new(),
            host,
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
                    let id = self.fetch();
                    let arg = self.pop();
                    let _ = (id, arg); // not implemented yet
                }

                Opcode::Print => {
                    let v = self.pop();
                    self.host.print(v);
                }

                Opcode::Halt => {
                    self.flags.halted = true;
                }
            }
        }

        self.host.report_flags(self.flags);
    }
}
