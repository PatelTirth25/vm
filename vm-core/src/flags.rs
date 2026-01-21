#[derive(Copy, Clone, Debug, Default)]
pub struct VmFlags {
    pub stack_overflow: bool,
    pub stack_underflow: bool,
    pub invalid_opcode: bool,
    pub halted: bool,
}

impl VmFlags {
    pub const fn new() -> Self {
        VmFlags {
            stack_overflow: false,
            stack_underflow: false,
            invalid_opcode: false,
            halted: false,
        }
    }

    pub fn any_error(&self) -> bool {
        self.stack_overflow || self.stack_underflow || self.invalid_opcode
    }
}
