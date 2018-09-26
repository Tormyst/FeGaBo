use super::mem;
use std::fmt::Debug;
use super::disassemble;

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
    HLI,
    HLD,
}

enum Flags {
    Z,
    N,
    H,
    C,
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

    fn execute(&mut self, opcode: u8, mem: &mut mem::Mem) -> usize {
        match opcode {
            0x01 => {
                self.c = self.load_pc_8(mem);
                self.b = self.load_pc_8(mem);
                12
            }
            0x0A => {
                self.a = mem.load_8(self.read_16(WordRegister::BC));
                8
            }
            0x11 => {
                self.e = self.load_pc_8(mem);
                self.d = self.load_pc_8(mem);
                12
            }
            0x1A => {
                self.a = mem.load_8(self.read_16(WordRegister::DE));
                8
            }
            0x21 => {
                self.l = self.load_pc_8(mem);
                self.h = self.load_pc_8(mem);
                12
            }
            0x2A => {
                self.a = mem.load_8(self.read_16(WordRegister::HLI));
                8
            }
            0x31 => {
                self.sp = self.load_pc_16(mem);
                12
            }
            0x3A => {
                self.a = mem.load_8(self.read_16(WordRegister::HLD));
                8
            }
            0xAF => {
                self.a = self.xor(ByteRegister::A, ByteRegister::A);
                4
            }
            _ => {
                panic!("Unknown instruction {:02X} was not implemented, dump of cpu: {:X?}",
                       opcode,
                       self)
            }
        }
    }

    pub fn cycle(&mut self, mem: &mut mem::Mem) {
        // Load opcode
        let opcode = self.load_pc_8(mem);
        // Print disassemble
        print!("Executing 0x{:04X}: 0x{:02X} ", self.pc - 1, opcode);
        disassemble::daInst(self.pc - 1, mem);
        // Execute
        let time = self.execute(opcode, mem);
    }

    fn load_pc_8(&mut self, mem: &mem::Mem) -> u8 {
        // Memory load
        let ret = mem.load_8(self.pc);
        self.pc += 1;
        ret
    }

    fn load_pc_16(&mut self, mem: &mem::Mem) -> u16 {
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

    fn read_16(&mut self, reg: WordRegister) -> u16 {
        let (high, low) = match reg {
            WordRegister::AF => (self.a, self.f),
            WordRegister::BC => (self.b, self.c),
            WordRegister::DE => (self.d, self.e),
            WordRegister::HL | WordRegister::HLI | WordRegister::HLD => (self.h, self.l),
        };
        let retVal = ((high as u16) << 8) | low as u16;
        match reg {
            WordRegister::HLI => self.write_16(reg, retVal + 1),
            WordRegister::HLD => self.write_16(reg, retVal - 1),
            _ => {},
        }
        retVal
    }

    fn write_16(&mut self, reg: WordRegister, val: u16) {
        let (high, low) = ((val >> 8) as u8, val as u8);
            match reg {
            WordRegister::AF => {self.a = high; self.f = low},
            WordRegister::BC => {self.b = high; self.c = low},
            WordRegister::DE => {self.d = high; self.e = low},
            WordRegister::HL | WordRegister::HLI | WordRegister::HLD => {self.h = high; self.l = low},
        };
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
