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
    SPP(u8),
    PC,
    HLI,
    HLD,
    IMM(u16),
    HighC,
    High(u8),
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
            SPP(data) => write!(f, "SP+{}", data),
            PC => write!(f, "PC"),
            HLI => write!(f, "HL+"),
            HLD => write!(f, "HL-"),
            IMM(data) => write!(f, "{:04X}", data),
            HighC => write!(f, "C"),
            High(op) => write!(f, "FF00+{}", op),
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

#[derive(Debug)]
pub struct OptFlag(pub Option<(Flag,bool)>);

const ofZ: OptFlag = OptFlag(Some((Flag::Z, true)));
const ofNZ: OptFlag = OptFlag(Some((Flag::Z, false)));
const ofC: OptFlag = OptFlag(Some((Flag::C, true)));
const ofNC: OptFlag = OptFlag(Some((Flag::C, false)));
const ofNone: OptFlag = OptFlag(None);

impl fmt::Display for OptFlag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OptFlag(None) => write!(f, ""),
            OptFlag(Some((flag, true))) => write!(f, "{}, ", flag),
            OptFlag(Some((flag, false))) => write!(f, "N{}, ", flag),
        }
    }
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

    RET(OptFlag),
    RETI,
    CALL(OptFlag, WordR),

    PUSH(WordR),
    POP(WordR),

    LD8(ByteR, ByteR),
    LD16(WordR, WordR),
    LDMem(ByteR, WordR),

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

    JR(OptFlag, i8),
    JP(OptFlag, WordR),

    RL(ByteR),
    RLC(ByteR),

    RR(ByteR),
    RRC(ByteR),

    SLA(ByteR),
    SRA(ByteR),

    SWAP(ByteR),
    SRL(ByteR),

    BIT(u8, ByteR),
    RES(u8, ByteR),
    SET(u8, ByteR),

    RLA,
    RLCA,
    RRA,
    RRCA,

    DI,
    EI,
}

macro_rules! fz {
    ($fs:expr) => {match $fs {true => "Z", false => ""}}
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Op::*;
        match self {
            NOP => write!(f, "NOP"),
            STOP => write!(f, "STOP"),
            HALT => write!(f, "HALT"),
            DAA => write!(f, "DAA"),
            CPL => write!(f, "CPL"),
            SCF => write!(f, "SCF"),
            CCF => write!(f, "CCF"),

            LD8(op1, op2) => write!(f, "LD {}, {}", op1, op2),
            LD16(op1, op2) => write!(f, "LD {}, {}", op1, op2),

            AND(op) => write!(f, "AND {}", op),
            OR(op) => write!(f, "OR {}", op),
            XOR(op) => write!(f, "XOR {}", op),
            CP(op) => write!(f, "CP {}", op),

            JR(fl, o) => write!(f, "JR {}{}", fl, o),
            JP(fl, o) => write!(f, "JP {}{}", fl, o),

            INC8(op) => write!(f, "INC {}", op),
            INC16(op) => write!(f, "INC {}", op),

            DEC8(op) => write!(f, "DEC {}", op),
            DEC16(op) => write!(f, "DEC {}", op),

            RL(op) => write!(f, "RL {}", op),
            RLC(op) => write!(f, "RLC {}", op),



            BIT(n, o) => write!(f, "BIT {}, {}", n, o),
            RES(n, o) => write!(f, "RES {}, {}", n, o),
            SET(n, o) => write!(f, "SET {}, {}", n, o),

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
        0x18 => op!(op, op2, JR(ofNone, op2 as i8), 8),
        0x19 => op!(op, ADD16(HL, DE),8),
        0x1A => op!(op, LD8(A, Mem(DE)),8),
        0x1B => op!(op, DEC16(DE),8),
        0x1C => op!(op, INC8(E),4),
        0x1D => op!(op, DEC8(E),4),
        0x1E => op!(op, op2, LD8(E,imm8!(op2)), 8),
        0x1F => op!(op, RRA, 4),

        0x20 => op!(op, op2, JR(ofNZ, op2 as i8), 8),
        0x21 => op!(op, op2, op3, LD16(HL, imm16!(op2, op3)), 12),
        0x22 => op!(op, LD8(Mem(HLI), A), 8),
        0x23 => op!(op, INC16(HL), 8),
        0x24 => op!(op, INC8(H), 4),
        0x25 => op!(op, DEC8(H), 4),
        0x26 => op!(op, op2, LD8(H,imm8!(op2)), 8),
        0x27 => op!(op, DAA, 4),
        0x28 => op!(op, op2, JR(ofZ,  op2 as i8), 12),
        0x29 => op!(op, ADD16(HL, HL),8),
        0x2A => op!(op, LD8(A, Mem(HLI)),8),
        0x2B => op!(op, DEC16(HL),8),
        0x2C => op!(op, INC8(L),4),
        0x2D => op!(op, DEC8(L),4),
        0x2E => op!(op, op2, LD8(L,imm8!(op2)), 8),
        0x2F => op!(op, CPL, 4),

        0x30 => op!(op, op2, JR(ofNC, op2 as i8), 8),
        0x31 => op!(op, op2, op3, LD16(SP, imm16!(op2, op3)), 12),
        0x32 => op!(op, LD8(Mem(HLD), A), 8),
        0x33 => op!(op, INC16(SP), 8),
        0x34 => op!(op, INC8(Mem(HL)), 4),
        0x35 => op!(op, DEC8(Mem(HL)), 4),
        0x36 => op!(op, op2, LD8(Mem(HL),imm8!(op2)), 8),
        0x37 => op!(op, SCF, 4),
        0x38 => op!(op, op2, JR(ofC, op2 as i8), 12),
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

        0xC0 => op!(op, RET(ofNZ), 8),
        0xC1 => op!(op, POP(BC), 12),
        0xC2 => op!(op, op2, op3, JP(ofNZ, imm16!(op2, op3)), 12),
        0xC3 => op!(op, op2, op3, JP(ofNone, imm16!(op2, op3)), 12),
        0xC4 => op!(op, op2, op3, CALL(ofNZ, imm16!(op2, op3)), 12),
        0xC5 => op!(op, PUSH(BC), 16),
        0xC6 => op!(op, op2, ADD8(A,imm8!(op2)), 8),
        0xC7 => op!(op, RST(0x0), 16),
        0xC8 => op!(op, RET(ofZ), 8),
        0xC9 => op!(op, RET(ofNone), 8),
        0xCA => op!(op, op2, op3, JP(ofZ, imm16!(op2, op3)), 12),
        0xCB => cbTable(op2),
        0xCC => op!(op, op2, op3, CALL(ofZ, imm16!(op2, op3)), 12),
        0xCD => op!(op, op2, op3, CALL(ofNone, imm16!(op2, op3)), 12),
        0xCE => op!(op, op2, ADC8(A, imm8!(op2)), 8),
        0xCF => op!(op, RST(0x8), 16),

        0xD0 => op!(op, RET(ofNC), 8),
        0xD1 => op!(op, POP(DE), 12),
        0xD2 => op!(op, op2, op3, JP(ofNC, imm16!(op2, op3)), 12),
        // 0xD3 => No instruction,
        0xD4 => op!(op, op2, op3, CALL(ofNC, imm16!(op2, op3)), 12),
        0xD5 => op!(op, PUSH(DE), 16),
        0xD6 => op!(op, op2, SUB(imm8!(op2)), 8),
        0xD7 => op!(op, RST(0x10), 16),
        0xD8 => op!(op, RET(ofC), 8),
        0xD9 => op!(op, RETI, 8),
        0xDA => op!(op, op2, op3, JP(ofC, imm16!(op2, op3)), 12),
        // 0xDB => No instruction,
        0xDC => op!(op, op2, op3, CALL(ofC, imm16!(op2, op3)), 12),
        // 0xDD => No instruction,
        0xDE => op!(op, op2, SBC(A, imm8!(op2)), 8),
        0xDF => op!(op, RST(0x18), 16),

        0xE0 => op!(op, op2, LD8(Mem(High(op2)),A), 12),
        0xE1 => op!(op, POP(HL), 12),
        0xE2 => op!(op, op2, LD8(Mem(HighC),A),8),
        // 0xE3 => No instruction,
        // 0xE4 => No instruction,
        0xE5 => op!(op, PUSH(HL), 16),
        0xE6 => op!(op, op2, AND(imm8!(op2)), 8),
        0xE7 => op!(op, RST(0x20), 16),
        0xE8 => op!(op, op2, ADD16(SP, imm16!(op2, 0xFF)), 16),
        0xE9 => op!(op, JP(ofNone, HL), 4),
        0xEA => op!(op, op2, op3, LD8(Mem(imm16!(op2, op3)), A), 16),
        // 0xEB => No instruction,
        // 0xEC => No instruction,
        // 0xED => No instruction,
        0xEE => op!(op, op2, XOR(imm8!(op2)), 8),
        0xEF => op!(op, RST(0x28), 16),

        0xF0 => op!(op, op2, LD8(A, Mem(High(op2))), 12),
        0xF1 => op!(op, POP(AF), 12),
        0xF2 => op!(op, op2, LD8(A, Mem(HighC)),8),
        0xF3 => op!(op, DI, 4),
        // 0xF4 => No instruction,
        0xF5 => op!(op, PUSH(AF), 16),
        0xF6 => op!(op, op2, OR(imm8!(op2)), 8),
        0xF7 => op!(op, RST(0x30), 16),
        0xF8 => op!(op, op2, LD16(HL, SPP(op2)), 12),
        0xF9 => op!(op, LD16(SP, HL), 8),
        0xFA => op!(op, op2, op3, LD8(A, Mem(imm16!(op2, op3))), 16),
        0xFB => op!(op, EI, 4),
        // 0xFC => No instruction,
        // 0xFD => No instruction,
        0xFE => op!(op, op2, CP(imm8!(op2)), 8),
        0xFF => op!(op, RST(0x38), 16),

        _ => {
            panic!("Instruction {:02X} from {:04X} cannot be decoded.",
                   op,
                   addr)
        }
    }
}

pub fn cbTable(op:u8) -> (OpCode, Op, u16, usize) {
    use self::Op::*;
    use self::ByteR::*;
    use self::WordR::*;

    match op {
        0x00 => op!(0xCB, op, RLC(B), 8),
        0x01 => op!(0xCB, op, RLC(C), 8),
        0x02 => op!(0xCB, op, RLC(D), 8),
        0x03 => op!(0xCB, op, RLC(E), 8),
        0x04 => op!(0xCB, op, RLC(H), 8),
        0x05 => op!(0xCB, op, RLC(L), 8),
        0x06 => op!(0xCB, op, RLC(Mem(HL)), 16),
        0x07 => op!(0xCB, op, RLC(A), 8),
        0x08 => op!(0xCB, op, RRC(B), 8),
        0x09 => op!(0xCB, op, RRC(C), 8),
        0x0A => op!(0xCB, op, RRC(D), 8),
        0x0B => op!(0xCB, op, RRC(E), 8),
        0x0C => op!(0xCB, op, RRC(H), 8),
        0x0D => op!(0xCB, op, RRC(L), 8),
        0x0E => op!(0xCB, op, RRC(Mem(HL)), 16),
        0x0F => op!(0xCB, op, RRC(A), 8),

        0x10 => op!(0xCB, op, RL(B), 8),
        0x11 => op!(0xCB, op, RL(C), 8),
        0x12 => op!(0xCB, op, RL(D), 8),
        0x13 => op!(0xCB, op, RL(E), 8),
        0x14 => op!(0xCB, op, RL(H), 8),
        0x15 => op!(0xCB, op, RL(L), 8),
        0x16 => op!(0xCB, op, RL(Mem(HL)), 16),
        0x17 => op!(0xCB, op, RL(A), 8),
        0x18 => op!(0xCB, op, RR(B), 8),
        0x19 => op!(0xCB, op, RR(C), 8),
        0x1A => op!(0xCB, op, RR(D), 8),
        0x1B => op!(0xCB, op, RR(E), 8),
        0x1C => op!(0xCB, op, RR(H), 8),
        0x1D => op!(0xCB, op, RR(L), 8),
        0x1E => op!(0xCB, op, RR(Mem(HL)), 16),
        0x1F => op!(0xCB, op, RR(A), 8),

        0x20 => op!(0xCB, op, SLA(B), 8),
        0x21 => op!(0xCB, op, SLA(C), 8),
        0x22 => op!(0xCB, op, SLA(D), 8),
        0x23 => op!(0xCB, op, SLA(E), 8),
        0x24 => op!(0xCB, op, SLA(H), 8),
        0x25 => op!(0xCB, op, SLA(L), 8),
        0x26 => op!(0xCB, op, SLA(Mem(HL)), 16),
        0x27 => op!(0xCB, op, SLA(A), 8),
        0x28 => op!(0xCB, op, SRA(B), 8),
        0x29 => op!(0xCB, op, SRA(C), 8),
        0x2A => op!(0xCB, op, SRA(D), 8),
        0x2B => op!(0xCB, op, SRA(E), 8),
        0x2C => op!(0xCB, op, SRA(H), 8),
        0x2D => op!(0xCB, op, SRA(L), 8),
        0x2E => op!(0xCB, op, SRA(Mem(HL)), 16),
        0x2F => op!(0xCB, op, SRA(A), 8),

        0x30 => op!(0xCB, op, SWAP(B), 8),
        0x31 => op!(0xCB, op, SWAP(C), 8),
        0x32 => op!(0xCB, op, SWAP(D), 8),
        0x33 => op!(0xCB, op, SWAP(E), 8),
        0x34 => op!(0xCB, op, SWAP(H), 8),
        0x35 => op!(0xCB, op, SWAP(L), 8),
        0x36 => op!(0xCB, op, SWAP(Mem(HL)), 16),
        0x37 => op!(0xCB, op, SWAP(A), 8),
        0x38 => op!(0xCB, op, SRL(B), 8),
        0x39 => op!(0xCB, op, SRL(C), 8),
        0x3A => op!(0xCB, op, SRL(D), 8),
        0x3B => op!(0xCB, op, SRL(E), 8),
        0x3C => op!(0xCB, op, SRL(H), 8),
        0x3D => op!(0xCB, op, SRL(L), 8),
        0x3E => op!(0xCB, op, SRL(Mem(HL)), 16),
        0x3F => op!(0xCB, op, SRL(A), 8),

        0x40 => op!(0xCB, op, BIT(0, B), 8),
        0x41 => op!(0xCB, op, BIT(0, C), 8),
        0x42 => op!(0xCB, op, BIT(0, D), 8),
        0x43 => op!(0xCB, op, BIT(0, E), 8),
        0x44 => op!(0xCB, op, BIT(0, H), 8),
        0x45 => op!(0xCB, op, BIT(0, L), 8),
        0x46 => op!(0xCB, op, BIT(0, Mem(HL)), 16),
        0x47 => op!(0xCB, op, BIT(0, A), 8),
        0x48 => op!(0xCB, op, BIT(1, B), 8),
        0x49 => op!(0xCB, op, BIT(1, C), 8),
        0x4A => op!(0xCB, op, BIT(1, D), 8),
        0x4B => op!(0xCB, op, BIT(1, E), 8),
        0x4C => op!(0xCB, op, BIT(1, H), 8),
        0x4D => op!(0xCB, op, BIT(1, L), 8),
        0x4E => op!(0xCB, op, BIT(1, Mem(HL)), 16),
        0x4F => op!(0xCB, op, BIT(1, A), 8),

        0x50 => op!(0xCB, op, BIT(2, B), 8),
        0x51 => op!(0xCB, op, BIT(2, C), 8),
        0x52 => op!(0xCB, op, BIT(2, D), 8),
        0x53 => op!(0xCB, op, BIT(2, E), 8),
        0x54 => op!(0xCB, op, BIT(2, H), 8),
        0x55 => op!(0xCB, op, BIT(2, L), 8),
        0x56 => op!(0xCB, op, BIT(2, Mem(HL)), 16),
        0x57 => op!(0xCB, op, BIT(2, A), 8),
        0x58 => op!(0xCB, op, BIT(3, B), 8),
        0x59 => op!(0xCB, op, BIT(3, C), 8),
        0x5A => op!(0xCB, op, BIT(3, D), 8),
        0x5B => op!(0xCB, op, BIT(3, E), 8),
        0x5C => op!(0xCB, op, BIT(3, H), 8),
        0x5D => op!(0xCB, op, BIT(3, L), 8),
        0x5E => op!(0xCB, op, BIT(3, Mem(HL)), 16),
        0x5F => op!(0xCB, op, BIT(3, A), 8),

        0x60 => op!(0xCB, op, BIT(4, B), 8),
        0x61 => op!(0xCB, op, BIT(4, C), 8),
        0x62 => op!(0xCB, op, BIT(4, D), 8),
        0x63 => op!(0xCB, op, BIT(4, E), 8),
        0x64 => op!(0xCB, op, BIT(4, H), 8),
        0x65 => op!(0xCB, op, BIT(4, L), 8),
        0x66 => op!(0xCB, op, BIT(4, Mem(HL)), 16),
        0x67 => op!(0xCB, op, BIT(4, A), 8),
        0x68 => op!(0xCB, op, BIT(5, B), 8),
        0x69 => op!(0xCB, op, BIT(5, C), 8),
        0x6A => op!(0xCB, op, BIT(5, D), 8),
        0x6B => op!(0xCB, op, BIT(5, E), 8),
        0x6C => op!(0xCB, op, BIT(5, H), 8),
        0x6D => op!(0xCB, op, BIT(5, L), 8),
        0x6E => op!(0xCB, op, BIT(5, Mem(HL)), 16),
        0x6F => op!(0xCB, op, BIT(5, A), 8),

        0x70 => op!(0xCB, op, BIT(6, B), 8),
        0x71 => op!(0xCB, op, BIT(6, C), 8),
        0x72 => op!(0xCB, op, BIT(6, D), 8),
        0x73 => op!(0xCB, op, BIT(6, E), 8),
        0x74 => op!(0xCB, op, BIT(6, H), 8),
        0x75 => op!(0xCB, op, BIT(6, L), 8),
        0x76 => op!(0xCB, op, BIT(6, Mem(HL)), 16),
        0x77 => op!(0xCB, op, BIT(6, A), 8),
        0x78 => op!(0xCB, op, BIT(7, B), 8),
        0x79 => op!(0xCB, op, BIT(7, C), 8),
        0x7A => op!(0xCB, op, BIT(7, D), 8),
        0x7B => op!(0xCB, op, BIT(7, E), 8),
        0x7C => op!(0xCB, op, BIT(7, H), 8),
        0x7D => op!(0xCB, op, BIT(7, L), 8),
        0x7E => op!(0xCB, op, BIT(7, Mem(HL)), 16),
        0x7F => op!(0xCB, op, BIT(7, A), 8),

        0x80 => op!(0xCB, op, RES(0, B), 8),
        0x81 => op!(0xCB, op, RES(0, C), 8),
        0x82 => op!(0xCB, op, RES(0, D), 8),
        0x83 => op!(0xCB, op, RES(0, E), 8),
        0x84 => op!(0xCB, op, RES(0, H), 8),
        0x85 => op!(0xCB, op, RES(0, L), 8),
        0x86 => op!(0xCB, op, RES(0, Mem(HL)), 16),
        0x87 => op!(0xCB, op, RES(0, A), 8),
        0x88 => op!(0xCB, op, RES(1, B), 8),
        0x89 => op!(0xCB, op, RES(1, C), 8),
        0x8A => op!(0xCB, op, RES(1, D), 8),
        0x8B => op!(0xCB, op, RES(1, E), 8),
        0x8C => op!(0xCB, op, RES(1, H), 8),
        0x8D => op!(0xCB, op, RES(1, L), 8),
        0x8E => op!(0xCB, op, RES(1, Mem(HL)), 16),
        0x8F => op!(0xCB, op, RES(1, A), 8),

        0x90 => op!(0xCB, op, RES(2, B), 8),
        0x91 => op!(0xCB, op, RES(2, C), 8),
        0x92 => op!(0xCB, op, RES(2, D), 8),
        0x93 => op!(0xCB, op, RES(2, E), 8),
        0x94 => op!(0xCB, op, RES(2, H), 8),
        0x95 => op!(0xCB, op, RES(2, L), 8),
        0x96 => op!(0xCB, op, RES(2, Mem(HL)), 16),
        0x97 => op!(0xCB, op, RES(2, A), 8),
        0x98 => op!(0xCB, op, RES(3, B), 8),
        0x99 => op!(0xCB, op, RES(3, C), 8),
        0x9A => op!(0xCB, op, RES(3, D), 8),
        0x9B => op!(0xCB, op, RES(3, E), 8),
        0x9C => op!(0xCB, op, RES(3, H), 8),
        0x9D => op!(0xCB, op, RES(3, L), 8),
        0x9E => op!(0xCB, op, RES(3, Mem(HL)), 16),
        0x9F => op!(0xCB, op, RES(3, A), 8),

        0xA0 => op!(0xCB, op, RES(4, B), 8),
        0xA1 => op!(0xCB, op, RES(4, C), 8),
        0xA2 => op!(0xCB, op, RES(4, D), 8),
        0xA3 => op!(0xCB, op, RES(4, E), 8),
        0xA4 => op!(0xCB, op, RES(4, H), 8),
        0xA5 => op!(0xCB, op, RES(4, L), 8),
        0xA6 => op!(0xCB, op, RES(4, Mem(HL)), 16),
        0xA7 => op!(0xCB, op, RES(4, A), 8),
        0xA8 => op!(0xCB, op, RES(5, B), 8),
        0xA9 => op!(0xCB, op, RES(5, C), 8),
        0xAA => op!(0xCB, op, RES(5, D), 8),
        0xAB => op!(0xCB, op, RES(5, E), 8),
        0xAC => op!(0xCB, op, RES(5, H), 8),
        0xAD => op!(0xCB, op, RES(5, L), 8),
        0xAE => op!(0xCB, op, RES(5, Mem(HL)), 16),
        0xAF => op!(0xCB, op, RES(5, A), 8),

        0xB0 => op!(0xCB, op, RES(6, B), 8),
        0xB1 => op!(0xCB, op, RES(6, C), 8),
        0xB2 => op!(0xCB, op, RES(6, D), 8),
        0xB3 => op!(0xCB, op, RES(6, E), 8),
        0xB4 => op!(0xCB, op, RES(6, H), 8),
        0xB5 => op!(0xCB, op, RES(6, L), 8),
        0xB6 => op!(0xCB, op, RES(6, Mem(HL)), 16),
        0xB7 => op!(0xCB, op, RES(6, A), 8),
        0xB8 => op!(0xCB, op, RES(7, B), 8),
        0xB9 => op!(0xCB, op, RES(7, C), 8),
        0xBA => op!(0xCB, op, RES(7, D), 8),
        0xBB => op!(0xCB, op, RES(7, E), 8),
        0xBC => op!(0xCB, op, RES(7, H), 8),
        0xBD => op!(0xCB, op, RES(7, L), 8),
        0xBE => op!(0xCB, op, RES(7, Mem(HL)), 16),
        0xBF => op!(0xCB, op, RES(7, A), 8),

        0xC0 => op!(0xCB, op, SET(0, B), 8),
        0xC1 => op!(0xCB, op, SET(0, C), 8),
        0xC2 => op!(0xCB, op, SET(0, D), 8),
        0xC3 => op!(0xCB, op, SET(0, E), 8),
        0xC4 => op!(0xCB, op, SET(0, H), 8),
        0xC5 => op!(0xCB, op, SET(0, L), 8),
        0xC6 => op!(0xCB, op, SET(0, Mem(HL)), 16),
        0xC7 => op!(0xCB, op, SET(0, A), 8),
        0xC8 => op!(0xCB, op, SET(1, B), 8),
        0xC9 => op!(0xCB, op, SET(1, C), 8),
        0xCA => op!(0xCB, op, SET(1, D), 8),
        0xCB => op!(0xCB, op, SET(1, E), 8),
        0xCC => op!(0xCB, op, SET(1, H), 8),
        0xCD => op!(0xCB, op, SET(1, L), 8),
        0xCE => op!(0xCB, op, SET(1, Mem(HL)), 16),
        0xCF => op!(0xCB, op, SET(1, A), 8),

        0xD0 => op!(0xCB, op, SET(2, B), 8),
        0xD1 => op!(0xCB, op, SET(2, C), 8),
        0xD2 => op!(0xCB, op, SET(2, D), 8),
        0xD3 => op!(0xCB, op, SET(2, E), 8),
        0xD4 => op!(0xCB, op, SET(2, H), 8),
        0xD5 => op!(0xCB, op, SET(2, L), 8),
        0xD6 => op!(0xCB, op, SET(2, Mem(HL)), 16),
        0xD7 => op!(0xCB, op, SET(2, A), 8),
        0xD8 => op!(0xCB, op, SET(3, B), 8),
        0xD9 => op!(0xCB, op, SET(3, C), 8),
        0xDA => op!(0xCB, op, SET(3, D), 8),
        0xDB => op!(0xCB, op, SET(3, E), 8),
        0xDC => op!(0xCB, op, SET(3, H), 8),
        0xDD => op!(0xCB, op, SET(3, L), 8),
        0xDE => op!(0xCB, op, SET(3, Mem(HL)), 16),
        0xDF => op!(0xCB, op, SET(3, A), 8),

        0xE0 => op!(0xCB, op, SET(4, B), 8),
        0xE1 => op!(0xCB, op, SET(4, C), 8),
        0xE2 => op!(0xCB, op, SET(4, D), 8),
        0xE3 => op!(0xCB, op, SET(4, E), 8),
        0xE4 => op!(0xCB, op, SET(4, H), 8),
        0xE5 => op!(0xCB, op, SET(4, L), 8),
        0xE6 => op!(0xCB, op, SET(4, Mem(HL)), 16),
        0xE7 => op!(0xCB, op, SET(4, A), 8),
        0xE8 => op!(0xCB, op, SET(5, B), 8),
        0xE9 => op!(0xCB, op, SET(5, C), 8),
        0xEA => op!(0xCB, op, SET(5, D), 8),
        0xEB => op!(0xCB, op, SET(5, E), 8),
        0xEC => op!(0xCB, op, SET(5, H), 8),
        0xED => op!(0xCB, op, SET(5, L), 8),
        0xEE => op!(0xCB, op, SET(5, Mem(HL)), 16),
        0xEF => op!(0xCB, op, SET(5, A), 8),

        0xF0 => op!(0xCB, op, SET(6, B), 8),
        0xF1 => op!(0xCB, op, SET(6, C), 8),
        0xF2 => op!(0xCB, op, SET(6, D), 8),
        0xF3 => op!(0xCB, op, SET(6, E), 8),
        0xF4 => op!(0xCB, op, SET(6, H), 8),
        0xF5 => op!(0xCB, op, SET(6, L), 8),
        0xF6 => op!(0xCB, op, SET(6, Mem(HL)), 16),
        0xF7 => op!(0xCB, op, SET(6, A), 8),
        0xF8 => op!(0xCB, op, SET(7, B), 8),
        0xF9 => op!(0xCB, op, SET(7, C), 8),
        0xFA => op!(0xCB, op, SET(7, D), 8),
        0xFB => op!(0xCB, op, SET(7, E), 8),
        0xFC => op!(0xCB, op, SET(7, H), 8),
        0xFD => op!(0xCB, op, SET(7, L), 8),
        0xFE => op!(0xCB, op, SET(7, Mem(HL)), 16),
        0xFF => op!(0xCB, op, SET(7, A), 8),

        _ => {
            panic!("Instruction CB{:02X} cannot be decoded.",
                   op)
        }
    }
}
