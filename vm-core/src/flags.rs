#[derive(Copy, Clone, Debug, Default)]
pub struct VmFlags {
    pub stack_overflow: bool,
    pub stack_underflow: bool,
    pub invalid_opcode: bool,
    pub memory_oob: bool,
    pub invalid_jump: bool,
    pub halted: bool,
}

impl VmFlags {
    pub const fn new() -> Self {
        VmFlags {
            stack_overflow: false,
            stack_underflow: false,
            invalid_opcode: false,
            memory_oob: false,
            invalid_jump: false,
            halted: false,
        }
    }

    pub fn any_error(&self) -> bool {
        self.stack_overflow
            || self.stack_underflow
            || self.invalid_opcode
            || self.memory_oob
            || self.invalid_jump
    }
}
