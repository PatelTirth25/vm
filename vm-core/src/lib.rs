#![no_std]

mod instruction;
pub mod flags;

use instruction::Opcode;
pub use flags::VmFlags;

pub trait Host {
    fn print(&self, value: i32);
    fn report_flags(&self, flags: VmFlags);
}

pub struct VM<'a, H: Host> {
    bytecode: &'a [u8],
    pc: usize,
    stack: [i32; 32],
    sp: usize,
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

                Opcode::CallNative => {
                    let id = self.fetch();
                    let arg = self.pop();
                    // not implemented yet
                    let _ = (id, arg);
                }

                Opcode::Halt => {
                    self.flags.halted = true;
                }
            }
        }

        if self.sp > 0 {
            self.host.print(self.stack[self.sp - 1]);
        }

        self.host.report_flags(self.flags);
    }
}
