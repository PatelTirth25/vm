#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Opcode {
    Push = 0x01,
    Add = 0x02,
    Sub = 0x03,
    Mul = 0x04,
    Div = 0x05,
    ArrAlloc = 0x06,
    ArrLen = 0x07,
    ArrLit = 0x08,

    Load = 0x20,
    Store = 0x21,
    ArrLoad = 0x22,
    ArrStore = 0x23,

    CmpEq = 0x30,
    CmpLt = 0x31,
    CmpGt = 0x32,

    Jmp = 0x40,
    JmpIfFalse = 0x41,

    CallNative = 0x50,
    Print = 0x51,
    PrintStr = 0x52,
    StrCmp = 0x53,
    StrConcat = 0x55,
    Halt = 0xFF,

    CmpLe = 0x34,
    CmpGe = 0x35,

    Call = 0x60,
    Ret = 0x61,
}

impl Opcode {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Opcode::Push),
            0x02 => Some(Opcode::Add),
            0x03 => Some(Opcode::Sub),
            0x04 => Some(Opcode::Mul),
            0x05 => Some(Opcode::Div),
            0x06 => Some(Opcode::ArrAlloc),
            0x07 => Some(Opcode::ArrLen),
            0x08 => Some(Opcode::ArrLit),
            0x20 => Some(Opcode::Load),
            0x21 => Some(Opcode::Store),
            0x22 => Some(Opcode::ArrLoad),
            0x23 => Some(Opcode::ArrStore),
            0x30 => Some(Opcode::CmpEq),
            0x31 => Some(Opcode::CmpLt),
            0x32 => Some(Opcode::CmpGt),
            0x34 => Some(Opcode::CmpLe),
            0x35 => Some(Opcode::CmpGe),
            0x40 => Some(Opcode::Jmp),
            0x41 => Some(Opcode::JmpIfFalse),
            0x50 => Some(Opcode::CallNative),
            0x51 => Some(Opcode::Print),
            0x52 => Some(Opcode::PrintStr),
            0x53 => Some(Opcode::StrCmp),
            0x55 => Some(Opcode::StrConcat),
            0x60 => Some(Opcode::Call),
            0x61 => Some(Opcode::Ret),
            0xFF => Some(Opcode::Halt),
            _ => None,
        }
    }
}
