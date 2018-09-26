use super::mem::Mem;
use std::fmt;

#[derive(Debug)]
pub enum ByteR {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    F,
    Mem(WordR),
    IMM(u8),
}

impl fmt::Display for ByteR {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ByteR::*;
        match self {
            A => write!(f, "A"),
            B => write!(f, "B"),
            C => write!(f, "C"),
            D => write!(f, "D"),
            E => write!(f, "E"),
            H => write!(f, "H"),
            L => write!(f, "L"),
            F => write!(f, "F"),
            Mem(data) => write!(f, "({})", data),
            IMM(data) => write!(f, "{:02X}", data),
        }
    }
}

#[derive(Debug)]
pub enum WordR {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
    HLI,
    HLD,
    IMM(u16),
}

impl fmt::Display for WordR {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::WordR::*;
        match self {
            AF => write!(f, "AF"),
            BC => write!(f, "BC"),
            DE => write!(f, "DE"),
            HL => write!(f, "HL"),
            SP => write!(f, "SP"),
            PC => write!(f, "PC"),
            HLI => write!(f, "HL+"),
            HLD => write!(f, "HL-"),
            IMM(data) => write!(f, "{:04X}", data),
        }
    }
}

#[derive(Debug)]
pub enum Flag {
    Z,
    N,
    H,
    C,
}

impl fmt::Display for Flag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Flag::*;
        match self {
            Z => write!(f, "Z"),
            N => write!(f, "N"),
            H => write!(f, "H"),
            C => write!(f, "C"),
        }
    }
}

#[derive(Debug)]
pub enum OpCode {
    One(u8),
    Two(u16),
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::OpCode::*;
        match self {
            One(data) => write!(f, "{:02X}", data),
            Two(data) => write!(f, "{:04X}", data),
        }
    }
}

#[derive(Debug)]
pub enum Op {
    NOP,
    LD8(ByteR, ByteR),
    LD16(WordR, WordR),
    XOR(ByteR),
    INC8(ByteR),
    INC16(WordR),
    DEC8(ByteR),
    DEC16(WordR),
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Op::*;
        match self {
            LD8(op1, op2) => write!(f, "LD {}, {}", op1, op2),
            LD16(op1, op2) => write!(f, "LD {}, {}", op1, op2),
            XOR(op) => write!(f, "XOR {}", op),
            INC8(op) => write!(f, "INC {}", op),
            INC16(op) => write!(f, "INC {}", op),
            DEC8(op) => write!(f, "DEC {}", op),
            DEC16(op) => write!(f, "DEC {}", op),
            _ => write!(f, "{:?}", self),
        }
    }
}

macro_rules! op {
    ($opcode:expr, $op:expr, $size:expr, $time:expr) => {
        (OpCode::One($opcode), $op, $size, $time)
    }
}

macro_rules! cb {
    ($opcode:expr, $op:expr, $time:expr) => {
        (OpCode::Two(0xCB | $opcode as u16), $op, 2, $time)
    }
}

macro_rules! imm8 {
    ($mem:expr, $addr:expr) => {ByteR::IMM($mem.load_8($addr + 1))}
}

macro_rules! imm16 {
    ($mem:expr, $addr:expr) => {WordR::IMM($mem.load_16($addr + 1))}
}

// Returns Opcode decoded, the number of bytes the instruction was
// and the number of cycles the instruction takes.
pub fn decode(addr: u16, mem: &mut Mem) -> (OpCode, Op, u16, usize) {
    use self::Op::*;
    use self::ByteR::*;
    use self::WordR::*;
    let op = mem.load_8(addr);

    match op {
        0x00 => op!(op, Op::NOP, 1, 4),
        0x01 => op!(op, LD16(BC, imm16!(mem, addr)), 3, 12),
        0x02 => op!(op, LD8(Mem(BC), A), 1, 8),
        0x03 => op!(op, INC16(BC), 1, 8),
        0x04 => op!(op, INC8(B), 1, 4),
        0x05 => op!(op, DEC8(B), 1, 4),
        0x06 => op!(op, LD8(B,imm8!(mem, addr)), 2, 8),

        0x11 => op!(op, LD16(DE, imm16!(mem, addr)), 3, 12),

        0x21 => op!(op, LD16(HL, imm16!(mem, addr)), 3, 12),

        0x31 => op!(op, LD16(SP, imm16!(mem, addr)), 3, 12),

        0xA8 => op!(op, XOR(B), 1, 4),
        0xA9 => op!(op, XOR(C), 1, 4),
        0xAA => op!(op, XOR(D), 1, 4),
        0xAB => op!(op, XOR(E), 1, 4),
        0xAC => op!(op, XOR(H), 1, 4),
        0xAD => op!(op, XOR(L), 1, 4),
        0xAE => op!(op, XOR(Mem(HL)), 1, 8),
        0xAF => op!(op, XOR(A), 1, 4),

        _ => {
            panic!("Instruction {:02X} from {:04X} cannot be decoded.",
                   op,
                   addr)
        }
    }
}
