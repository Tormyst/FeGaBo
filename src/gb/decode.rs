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
    Two(u8, u8),
    Three(u8, u8, u8),
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::OpCode::*;
        match self {
            One(d) => write!(f, "{:02X}      ", d),
            Two(d, d2) => write!(f, "{:02X} {:02X}   ", d, d2),
            Three(d, d2, d3) => write!(f, "{:02X} {:02X} {:02X}", d, d2, d3),
        }
    }
}

#[derive(Debug)]
pub enum Op {
    NOP,
    STOP,
    HALT,
    DAA,
    CPL,
    SCF,
    CCF,

    RST(u8),

    RET(Option<(Flag, bool)>),
    RETI,
    CALL(Option<(Flag, bool)>, WordR),

    PUSH(WordR),
    POP(WordR),

    LD8(ByteR, ByteR),
    LD16(WordR, WordR),
    LDMem(ByteR, WordR),
    LDH(ByteR, ByteR),

    INC8(ByteR),
    INC16(WordR),

    DEC8(ByteR),
    DEC16(WordR),

    ADD8(ByteR, ByteR),
    ADD16(WordR, WordR),
    
    ADC8(ByteR, ByteR),
    ADC16(WordR, WordR),

    SUB(ByteR),
    
    SBC(ByteR, ByteR),

    AND(ByteR),
    OR(ByteR),
    XOR(ByteR),
    CP(ByteR),

    JR(Option<(Flag, bool)>, ByteR),
    JP(Option<(Flag, bool)>, WordR),

    RL(ByteR),
    RLC(ByteR),

    RR(ByteR),
    RRC(ByteR),

    RLA,
    RLCA,
    RRA,
    RRCA,
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
            RL(op) => write!(f, "RL {}", op),
            RLC(op) => write!(f, "RLC {}", op),

            _ => write!(f, "{:?}", self),
        }
    }
}

macro_rules! op {
    ($o:expr, $op:expr, $time:expr) => {
        (OpCode::One($o), $op, 1, $time)
    };

    ($o:expr, $o2:expr, $op:expr, $time:expr) => {
        (OpCode::Two($o, $o2), $op, 2, $time)
    };

    ($o:expr, $o2:expr, $o3:expr, $op:expr, $time:expr) => {
        (OpCode::Three($o, $o2, $o3), $op, 3, $time)
    };
}

macro_rules! imm8 {
    ($op:expr) => {ByteR::IMM($op)}
}

macro_rules! imm16 {
    ($low:expr, $high:expr) => {WordR::IMM(($high as u16) << 8|$low as u16)}
}

// Returns Opcode decoded, the number of bytes the instruction was
// and the number of cycles the instruction takes.
pub fn decode(addr: u16, mem: &mut Mem) -> (OpCode, Op, u16, usize) {
    use self::Op::*;
    use self::ByteR::*;
    use self::WordR::*;
    let op = mem.load_8(addr);
    // Figure out a way to avoid extra loads that will be pointless for
    // instructions that are not 3 bytes long.
    let op2 = mem.load_8(addr + 1);
    let op3 = mem.load_8(addr + 2);

    match op {
        0x00 => op!(op, NOP, 4),
        0x01 => op!(op, op2, op3, LD16(BC, imm16!(op2, op3)), 12),
        0x02 => op!(op, LD8(Mem(BC), A), 8),
        0x03 => op!(op, INC16(BC), 8),
        0x04 => op!(op, INC8(B), 4),
        0x05 => op!(op, DEC8(B), 4),
        0x06 => op!(op, op2, LD8(B,imm8!(op2)), 8),
        0x07 => op!(op, RLCA, 4),
        0x08 => op!(op, op2, op3, LDMem(Mem(imm16!(op2, op3)), SP), 20),
        0x09 => op!(op, ADD16(HL, BC),8),
        0x0A => op!(op, LD8(A, Mem(BC)),8),
        0x0B => op!(op, DEC16(BC),8),
        0x0C => op!(op, INC8(C),4),
        0x0D => op!(op, DEC8(C),4),
        0x0E => op!(op, op2, LD8(C,imm8!(op2)), 8),
        0x0F => op!(op, RRCA, 4),

        0x10 => op!(op, STOP, 4),
        0x11 => op!(op, op2, op3, LD16(DE, imm16!(op2, op3)), 12),
        0x12 => op!(op, LD8(Mem(DE), A), 8),
        0x13 => op!(op, INC16(DE), 8),
        0x14 => op!(op, INC8(D), 4),
        0x15 => op!(op, DEC8(D), 4),
        0x16 => op!(op, op2, LD8(D,imm8!(op2)), 8),
        0x17 => op!(op, RLA, 4),
        0x18 => op!(op, op2, JR(None, imm8!(op2)), 8),
        0x19 => op!(op, ADD16(HL, DE),8),
        0x1A => op!(op, LD8(A, Mem(DE)),8),
        0x1B => op!(op, DEC16(DE),8),
        0x1C => op!(op, INC8(E),4),
        0x1D => op!(op, DEC8(E),4),
        0x1E => op!(op, op2, LD8(E,imm8!(op2)), 8),
        0x1F => op!(op, RRA, 4),

        0x20 => op!(op, op2, JR(Some((Flag::Z, false)), imm8!(op2)), 8),
        0x21 => op!(op, op2, op3, LD16(HL, imm16!(op2, op3)), 12),
        0x22 => op!(op, LD8(Mem(HLI), A), 8),
        0x23 => op!(op, INC16(HL), 8),
        0x24 => op!(op, INC8(H), 4),
        0x25 => op!(op, DEC8(H), 4),
        0x26 => op!(op, op2, LD8(H,imm8!(op2)), 8),
        0x27 => op!(op, DAA, 4),
        0x28 => op!(op, op2, JR(Some((Flag::Z, true)), imm8!(op2)), 12),
        0x29 => op!(op, ADD16(HL, HL),8),
        0x2A => op!(op, LD8(A, Mem(HLI)),8),
        0x2B => op!(op, DEC16(HL),8),
        0x2C => op!(op, INC8(L),4),
        0x2D => op!(op, DEC8(L),4),
        0x2E => op!(op, op2, LD8(L,imm8!(op2)), 8),
        0x2F => op!(op, CPL, 4),

        0x30 => op!(op, op2, JR(Some((Flag::C, false)), imm8!(op2)), 8),
        0x31 => op!(op, op2, op3, LD16(SP, imm16!(op2, op3)), 12),
        0x32 => op!(op, LD8(Mem(HLD), A), 8),
        0x33 => op!(op, INC16(SP), 8),
        0x34 => op!(op, INC8(Mem(HL)), 4),
        0x35 => op!(op, DEC8(Mem(HL)), 4),
        0x36 => op!(op, op2, LD8(Mem(HL),imm8!(op2)), 8),
        0x37 => op!(op, SCF, 4),
        0x38 => op!(op, op2, JR(Some((Flag::C, true)), imm8!(op2)), 12),
        0x39 => op!(op, ADD16(HL, SP),8),
        0x3A => op!(op, LD8(A, Mem(HLD)),8),
        0x3B => op!(op, DEC16(SP),8),
        0x3C => op!(op, INC8(A),4),
        0x3D => op!(op, DEC8(A),4),
        0x3E => op!(op, op2, LD8(A,imm8!(op2)), 8),
        0x3F => op!(op, CCF, 4),

        0x40 => op!(op, LD8(B,B),4),
        0x41 => op!(op, LD8(B,C),4),
        0x42 => op!(op, LD8(B,D),4),
        0x43 => op!(op, LD8(B,E),4),
        0x44 => op!(op, LD8(B,H),4),
        0x45 => op!(op, LD8(B,L),4),
        0x46 => op!(op, LD8(B,Mem(HL)),8),
        0x47 => op!(op, LD8(B,A),4),
        0x48 => op!(op, LD8(C,B),4),
        0x49 => op!(op, LD8(C,C),4),
        0x4A => op!(op, LD8(C,D),4),
        0x4B => op!(op, LD8(C,E),4),
        0x4C => op!(op, LD8(C,H),4),
        0x4D => op!(op, LD8(C,L),4),
        0x4E => op!(op, LD8(C,Mem(HL)),8),
        0x4F => op!(op, LD8(C,A),4),

        0x50 => op!(op, LD8(D,B),4),
        0x51 => op!(op, LD8(D,C),4),
        0x52 => op!(op, LD8(D,D),4),
        0x53 => op!(op, LD8(D,E),4),
        0x54 => op!(op, LD8(D,H),4),
        0x55 => op!(op, LD8(D,L),4),
        0x56 => op!(op, LD8(D,Mem(HL)),8),
        0x57 => op!(op, LD8(D,A),4),
        0x58 => op!(op, LD8(E,B),4),
        0x59 => op!(op, LD8(E,C),4),
        0x5A => op!(op, LD8(E,D),4),
        0x5B => op!(op, LD8(E,E),4),
        0x5C => op!(op, LD8(E,H),4),
        0x5D => op!(op, LD8(E,L),4),
        0x5E => op!(op, LD8(E,Mem(HL)),8),
        0x5F => op!(op, LD8(E,A),4),

        0x60 => op!(op, LD8(H,B),4),
        0x61 => op!(op, LD8(H,C),4),
        0x62 => op!(op, LD8(H,D),4),
        0x63 => op!(op, LD8(H,E),4),
        0x64 => op!(op, LD8(H,H),4),
        0x65 => op!(op, LD8(H,L),4),
        0x66 => op!(op, LD8(H,Mem(HL)),8),
        0x67 => op!(op, LD8(H,A),4),
        0x68 => op!(op, LD8(L,B),4),
        0x69 => op!(op, LD8(L,C),4),
        0x6A => op!(op, LD8(L,D),4),
        0x6B => op!(op, LD8(L,E),4),
        0x6C => op!(op, LD8(L,H),4),
        0x6D => op!(op, LD8(L,L),4),
        0x6E => op!(op, LD8(L,Mem(HL)),8),
        0x6F => op!(op, LD8(L,A),4),

        0x70 => op!(op, LD8(Mem(HL),B),4),
        0x71 => op!(op, LD8(Mem(HL),C),4),
        0x72 => op!(op, LD8(Mem(HL),D),4),
        0x73 => op!(op, LD8(Mem(HL),E),4),
        0x74 => op!(op, LD8(Mem(HL),H),4),
        0x75 => op!(op, LD8(Mem(HL),L),4),
        0x76 => op!(op, HALT, 4),
        0x77 => op!(op, LD8(Mem(HL),A),4),
        0x78 => op!(op, LD8(A,B),4),
        0x79 => op!(op, LD8(A,C),4),
        0x7A => op!(op, LD8(A,D),4),
        0x7B => op!(op, LD8(A,E),4),
        0x7C => op!(op, LD8(A,H),4),
        0x7D => op!(op, LD8(A,L),4),
        0x7E => op!(op, LD8(A,Mem(HL)),8),
        0x7F => op!(op, LD8(A,A),4),

        0x80 => op!(op, ADD8(A,B),4),
        0x81 => op!(op, ADD8(A,C),4),
        0x82 => op!(op, ADD8(A,D),4),
        0x83 => op!(op, ADD8(A,E),4),
        0x84 => op!(op, ADD8(A,H),4),
        0x85 => op!(op, ADD8(A,L),4),
        0x86 => op!(op, ADD8(A,Mem(HL)),4),
        0x87 => op!(op, ADD8(A,A),4),
        0x88 => op!(op, ADC8(A,B),4),
        0x89 => op!(op, ADC8(A,C),4),
        0x8A => op!(op, ADC8(A,D),4),
        0x8B => op!(op, ADC8(A,E),4),
        0x8C => op!(op, ADC8(A,H),4),
        0x8D => op!(op, ADC8(A,L),4),
        0x8E => op!(op, ADC8(A,Mem(HL)),8),
        0x8F => op!(op, ADC8(A,A),4),

        0x90 => op!(op, SUB(B),4),
        0x91 => op!(op, SUB(C),4),
        0x92 => op!(op, SUB(D),4),
        0x93 => op!(op, SUB(E),4),
        0x94 => op!(op, SUB(H),4),
        0x95 => op!(op, SUB(L),4),
        0x96 => op!(op, SUB(Mem(HL)),4),
        0x97 => op!(op, SUB(A),4),
        0x98 => op!(op, SBC(A,B),4),
        0x99 => op!(op, SBC(A,C),4),
        0x9A => op!(op, SBC(A,D),4),
        0x9B => op!(op, SBC(A,E),4),
        0x9C => op!(op, SBC(A,H),4),
        0x9D => op!(op, SBC(A,L),4),
        0x9E => op!(op, SBC(A,Mem(HL)),8),
        0x9F => op!(op, SBC(A,A),4),

        0xA0 => op!(op, AND(B), 4),
        0xA1 => op!(op, AND(C), 4),
        0xA2 => op!(op, AND(D), 4),
        0xA3 => op!(op, AND(E), 4),
        0xA4 => op!(op, AND(H), 4),
        0xA5 => op!(op, AND(L), 4),
        0xA6 => op!(op, AND(Mem(HL)), 8),
        0xA7 => op!(op, AND(A), 4),
        0xA8 => op!(op, XOR(B), 4),
        0xA9 => op!(op, XOR(C), 4),
        0xAA => op!(op, XOR(D), 4),
        0xAB => op!(op, XOR(E), 4),
        0xAC => op!(op, XOR(H), 4),
        0xAD => op!(op, XOR(L), 4),
        0xAE => op!(op, XOR(Mem(HL)), 8),
        0xAF => op!(op, XOR(A), 4),

        0xB0 => op!(op, OR(B), 4),
        0xB1 => op!(op, OR(C), 4),
        0xB2 => op!(op, OR(D), 4),
        0xB3 => op!(op, OR(E), 4),
        0xB4 => op!(op, OR(H), 4),
        0xB5 => op!(op, OR(L), 4),
        0xB6 => op!(op, OR(Mem(HL)), 8),
        0xB7 => op!(op, OR(A), 4),
        0xB8 => op!(op, CP(B), 4),
        0xB9 => op!(op, CP(C), 4),
        0xBA => op!(op, CP(D), 4),
        0xBB => op!(op, CP(E), 4),
        0xBC => op!(op, CP(H), 4),
        0xBD => op!(op, CP(L), 4),
        0xBE => op!(op, CP(Mem(HL)), 8),
        0xBF => op!(op, CP(A), 4),

        0xC0 => op!(op, RET(Some((Flag::Z,false))), 8),
        0xC1 => op!(op, POP(BC), 12),
        0xC2 => op!(op, op2, op3, JP(Some((Flag::Z,false)), imm16!(op2, op3)), 12),
        0xC3 => op!(op, op2, op3, JP(None, imm16!(op2, op3)), 12),
        0xC4 => op!(op, op2, op3, CALL(Some((Flag::Z,false)), imm16!(op2, op3)), 12),
        0xC5 => op!(op, PUSH(BC), 16),
        0xC6 => op!(op, op2, ADD8(A,imm8!(op2)), 8),
        0xC7 => op!(op, RST(0x0), 16),
        0xC8 => op!(op, RET(Some((Flag::Z,true))), 8),
        0xC9 => op!(op, RET(None), 8),
        0xCA => op!(op, op2, op3, JP(Some((Flag::Z,true)), imm16!(op2, op3)), 12),
        0xCB => cbTable(op2),
        0xCC => op!(op, op2, op3, CALL(Some((Flag::Z,true)), imm16!(op2, op3)), 12),
        0xCD => op!(op, op2, op3, CALL(None, imm16!(op2, op3)), 12),
        0xCE => op!(op, op2, ADC8(A, imm8!(op2)), 8),
        0xCF => op!(op, RST(0x8), 16),

        0xD0 => op!(op, RET(Some((Flag::C,false))), 8),
        0xD1 => op!(op, POP(DE), 12),
        0xD2 => op!(op, op2, op3, JP(Some((Flag::C,false)), imm16!(op2, op3)), 12),
        // 0xD3 => No instruction,
        0xD4 => op!(op, op2, op3, CALL(Some((Flag::C,false)), imm16!(op2, op3)), 12),
        0xD5 => op!(op, PUSH(DE), 16),
        0xD6 => op!(op, op2, SUB(imm8!(op2)), 8),
        0xD7 => op!(op, RST(0x10), 16),
        0xD8 => op!(op, RET(Some((Flag::C,true))), 8),
        0xD9 => op!(op, RETI, 8),
        0xDA => op!(op, op2, op3, JP(Some((Flag::C,true)), imm16!(op2, op3)), 12),
        // 0xDB => No instruction,
        0xDC => op!(op, op2, op3, CALL(Some((Flag::C,true)), imm16!(op2, op3)), 12),
        // 0xDD => No instruction,
        0xDE => op!(op, op2, SBC(A, imm8!(op2)), 8),
        0xDF => op!(op, RST(0x18), 16),

        0xE0 => op!(op, op2, LDH(Mem(WordR::IMM(0xFF00|(op2 as u16))),A), 12),
        0xE1 => op!(op, POP(HL), 12),

        _ => {
            panic!("Instruction {:02X} from {:04X} cannot be decoded.",
                   op,
                   addr)
        }
    }
}

pub fn cbTable(op:u8) -> (OpCode, Op, u16, usize) {
    match op {
        _ => {
            panic!("Instruction CB{:02X} cannot be decoded.",
                   op)
        }
    }
}
