#![no_std]

pub trait Host {
    fn print(&self, value: i32);
}

pub struct VM<'a, H: Host> {
    bytecode: &'a [u8],
    pc: usize,
    stack: [i32; 32],
    sp: usize,
    host: H,
}

impl<'a, H: Host> VM<'a, H> {
    pub const fn new(bytecode: &'a [u8], host: H) -> Self {
        Self {
            bytecode,
            pc: 0,
            stack: [0; 32],
            sp: 0,
            host,
        }
    }

    fn push(&mut self, value: i32) {
        if self.sp < self.stack.len() {
            self.stack[self.sp] = value;
            self.sp += 1;
        }
    }

    fn pop(&mut self) -> i32 {
        if self.sp == 0 {
            0
        } else {
            self.sp -= 1;
            self.stack[self.sp]
        }
    }

    fn fetch(&mut self) -> u8 {
        let b = self.bytecode[self.pc];
        self.pc += 1;
        b
    }

    pub fn run(&mut self) {
        loop {
            match self.fetch() {
                0x01 => {
                    let d = self.fetch() as i32;
                    self.push(d);
                }
                0x02 => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a + b);
                }
                0xFF => break,
                _ => break,
            }
        }

        if self.sp > 0 {
            self.host.print(self.stack[self.sp - 1]);
        }
    }
}
