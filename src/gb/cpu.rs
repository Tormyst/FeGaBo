use std::fmt;
use super::mem;
use super::mem::Mem;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Cpu {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    f: u8,
    sp: u16,
    pc: u16,
}

enum ByteRegister {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    F,
}

enum WordRegister {
    AF,
    BC,
    DE,
    HL,
}

enum Flags {
    Z,
    N,
    H,
    C,
}

struct Inst {
    opcode: u8,
    formater: fn(u8, u16) -> String,
    execute: fn(&mut Cpu, &mut Mem, u8, u16) -> usize,
    operand_8: u8,
    operand_16: u16,
}

macro_rules! i {
    ($op:expr, $f:expr, $i:expr) => {
        (Inst {
            opcode: $op,
            formater: |_x, _y|{
                format!($f)
            },
            execute: $i,
            operand_8: 0,
            operand_16: 0,
        }, 1)
    }
}

macro_rules! i_8 {
    ($op:expr, $f:expr, $i:expr, $x:expr) => {
        (Inst {
            opcode: $op,
            formater: |op, _|{
                format!($f, op)
            },
            execute: $i,
            operand_8: $x,
            operand_16: 0,
        }, 2)
    }
}
macro_rules! i_16 {
    ($op:expr, $f:expr, $i:expr, $x:expr) => {
        (Inst {
            opcode: $op,
            formater: |_, op|{
                format!($f, op)
            },
            execute: $i,
            operand_8: 0,
            operand_16: $x,
        }, 3)
    }
}

impl Inst {
    fn load_opcode(addr: u16, mem: &Mem) -> (Self, u16) {
        let o = mem.load_8(addr);
        match o {
            0x31 => {
                i_16!(o,
                      "LD SP, 0x{:04X}",
                      |cpu, mem, _, op| {
                          cpu.sp = op;
                          12
                      },
                      mem.load_16(addr + 1))
            }
            0xAF => {
                i!(o, "XOR A", |cpu, mem, _, _| {
                    cpu.a = cpu.xor(ByteRegister::A, ByteRegister::A);
                    4
                })
            }
            _ => panic!("Unknown instruction {:02X} was not implemented", o),
        }
    }

    fn run(&mut self, cpu: &mut Cpu, mem: &mut Mem) -> usize {
        (self.execute)(cpu, mem, self.operand_8, self.operand_16)
    }
}

impl fmt::Display for Inst {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "0x{:02X}: {}",
               self.opcode,
               (self.formater)(self.operand_8, self.operand_16))
    }
}


impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            f: 0,
            sp: 0,
            pc: 0,
        }
    }

    pub fn cycle(&mut self, mem: &mut Mem) {
        // let opcode = self.load_pc_8(mem);
        let (opcode, inst_size) = Inst::load_opcode(self.pc, mem);
        println!("Executing 0x{:04X}: {}", self.pc, opcode);
        self.pc += inst_size;
    }

    fn load_pc_8(&mut self, mem: &Mem) -> u8 {
        // Memory load
        let ret = mem.load_8(self.pc);
        self.pc += 1;
        ret
    }

    fn load_pc_16(&mut self, mem: &Mem) -> u16 {
        // Memory load
        let ret = mem.load_16(self.pc);
        self.pc += 2;
        ret
    }

    fn read_8(&self, reg: ByteRegister) -> u8 {
        match reg {
            ByteRegister::A => self.a,
            ByteRegister::B => self.b,
            ByteRegister::C => self.c,
            ByteRegister::D => self.d,
            ByteRegister::E => self.e,
            ByteRegister::H => self.h,
            ByteRegister::L => self.l,
            ByteRegister::F => self.f,
        }
    }

    fn read_16(&self, reg: WordRegister) -> u16 {
        let (high, low) = match reg {
            WordRegister::AF => (self.a, self.f),
            WordRegister::BC => (self.b, self.c),
            WordRegister::DE => (self.d, self.e),
            WordRegister::HL => (self.h, self.l),
        };
        ((high as u16) << 8) | low as u16
    }

    fn xor(&mut self, reg1: ByteRegister, reg2: ByteRegister) -> u8 {
        let val = self.read_8(reg1) ^ self.read_8(reg2);
        self.set_flag(Flags::Z, val == 0);
        self.set_flag(Flags::N, false);
        self.set_flag(Flags::H, false);
        self.set_flag(Flags::C, false);
        val
    }


    fn read_flag(&self, flag: Flags) -> bool {
        let mask = match flag {
            Flags::Z => 1 << 7,
            Flags::N => 1 << 6,
            Flags::H => 1 << 5,
            Flags::C => 1 << 4,
        };
        mask & self.f != 0
    }

    fn set_flag(&mut self, flag: Flags, val: bool) {
        let mask = match flag {
            Flags::Z => 1 << 7,
            Flags::N => 1 << 6,
            Flags::H => 1 << 5,
            Flags::C => 1 << 4,
        };
        match val {
            true => self.f |= mask,
            false => self.f &= !mask,

        }
    }
}
