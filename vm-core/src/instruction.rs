#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Opcode {
    Push        = 0x01,
    Add         = 0x02,
    Sub         = 0x03,
    CallNative  = 0x40,
    Halt        = 0xFF,
}

impl Opcode {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Opcode::Push),
            0x02 => Some(Opcode::Add),
            0x03 => Some(Opcode::Sub),
            0x40 => Some(Opcode::CallNative),
            0xFF => Some(Opcode::Halt),
            _ => None,
        }
    }
}
