#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Opcode {
    Push        = 0x01,
    Add         = 0x02,
    Sub         = 0x03,

    Load        = 0x20,
    Store       = 0x21,

    CmpEq       = 0x30,
    CmpLt       = 0x31,
    CmpGt       = 0x32,

    Jmp         = 0x40,
    JmpIfFalse  = 0x41,

    CallNative  = 0x50,
    Halt        = 0xFF,
}

impl Opcode {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Opcode::Push),
            0x02 => Some(Opcode::Add),
            0x03 => Some(Opcode::Sub),
            0x20 => Some(Opcode::Load),
            0x21 => Some(Opcode::Store),
            0x30 => Some(Opcode::CmpEq),
            0x31 => Some(Opcode::CmpLt),
            0x32 => Some(Opcode::CmpGt),
            0x40 => Some(Opcode::Jmp),
            0x41 => Some(Opcode::JmpIfFalse),
            0x50 => Some(Opcode::CallNative),
            0xFF => Some(Opcode::Halt),
            _ => None,
        }
    }
}
