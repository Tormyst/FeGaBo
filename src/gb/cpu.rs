use super::decode;
use super::decode::{ByteR, Flag, Op, WordR};
use super::disassemble;
use super::mem;
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

    fn execute_op(&mut self, opcode: decode::Op, mem: &mut mem::Mem) {
        use gb::decode::Op::*;
        match opcode {
            NOP => {}

            LD8(o1, o2) => {
                let data = self.read_8(o2, mem);
                self.write_8(o1, data, mem)
            }
            LD16(o1, o2) => {
                let data = self.read_16(o2);
                self.write_16(o1, data)
            }

            XOR(o) => self.xor(o, mem),
            
            JR(fl, o) => self.jr(fl, o, mem),

            BIT(n, o) => self.bit(n, o, mem),
            
            _ => panic!("Instruction {} not implemented.", opcode),
        }
    }

    pub fn cycle(&mut self, mem: &mut mem::Mem) {
        // Load opcode
        let (instruction, opcode, op_size, op_time) = decode::decode(self.pc, mem);

        // Print disassemble
        println!("Executing 0x{:04X}: {}    {}", self.pc, instruction, opcode);

        //Increment PC
        self.pc += op_size;

        // Execute
        self.execute_op(opcode, mem);
    }

    fn read_8(&mut self, reg: ByteR, mem: &mem::Mem) -> u8 {
        match reg {
            ByteR::A => self.a,
            ByteR::B => self.b,
            ByteR::C => self.c,
            ByteR::D => self.d,
            ByteR::E => self.e,
            ByteR::H => self.h,
            ByteR::L => self.l,
            ByteR::F => self.f,
            ByteR::IMM(data) => data,
            ByteR::Mem(addr) => mem.load_8(self.read_16(addr)),
        }
    }

    fn write_8(&mut self, reg: ByteR, data: u8, mem: &mut mem::Mem) {
        match reg {
            ByteR::A => self.a = data,
            ByteR::B => self.b = data,
            ByteR::C => self.c = data,
            ByteR::D => self.d = data,
            ByteR::E => self.e = data,
            ByteR::H => self.h = data,
            ByteR::L => self.l = data,
            ByteR::F => self.f = data,
            ByteR::IMM(_) => panic!("You cannot write to a immediate value"),
            ByteR::Mem(addr) => mem.write_8(self.read_16(addr), data),
        }
    }

    fn read_16(&mut self, reg: WordR) -> u16 {
        match reg {
            WordR::SP => self.sp,
            WordR::PC => self.pc,
            WordR::IMM(data) => data,
            WordR::HighC => 0xFF00 | (self.c as u16),
            WordR::High(op) => 0xFF00 | (op as u16),
            _ => {
                let (high, low) = match reg {
                    WordR::AF => (self.a, self.f),
                    WordR::BC => (self.b, self.c),
                    WordR::DE => (self.d, self.e),
                    WordR::HL | WordR::HLI | WordR::HLD => (self.h, self.l),
                    _ => {
                        unreachable!("All cases not handled here should have already been handled")
                    }
                };
                let retVal = ((high as u16) << 8) | low as u16;
                // Post read operation for increment and decrement
                match reg {
                    WordR::HLI => self.write_16(reg, retVal + 1),
                    WordR::HLD => self.write_16(reg, retVal - 1),
                    _ => {}
                }
                retVal
            }
        }
    }

    fn write_16(&mut self, reg: WordR, val: u16) {
        let (high, low) = ((val >> 8) as u8, val as u8);
        match reg {
            WordR::SP => self.sp = val,
            WordR::PC => self.pc = val,
            WordR::AF => {
                self.a = high;
                self.f = low
            }
            WordR::BC => {
                self.b = high;
                self.c = low
            }
            WordR::DE => {
                self.d = high;
                self.e = low
            }
            WordR::HL | WordR::HLI | WordR::HLD => {
                self.h = high;
                self.l = low
            }
            _ => panic!("You cannot write to a read only word"),
        };
    }

    fn flagCondition(&self, fl: decode::OptFlag) -> bool{
        match fl {
            decode::OptFlag(None) => true,
            decode::OptFlag(Some((flag, state))) => self.read_flag(flag) == state,
        }
    }

    fn xor(&mut self, reg: ByteR, mem: &mut mem::Mem) {
        self.a = self.read_8(reg, mem) ^ self.a;
        let zero = self.a == 0;
        self.set_flag(Flag::Z, zero);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::C, false);
    }

    fn jr(&mut self, fl: decode::OptFlag, o: i8, mem: &mut mem::Mem){
        if self.flagCondition(fl) {
            self.pc = self.pc.wrapping_add(o as u16);
        }
    }

    fn bit(&mut self, n: u8, reg: ByteR, mem: &mut mem::Mem) {
        let zero = self.read_8(reg, mem) & (0x1 << n) == 0;
        self.set_flag(Flag::Z, zero);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
    }


    fn read_flag(&self, flag: Flag) -> bool {
        let mask = match flag {
            Flag::Z => 1 << 7,
            Flag::N => 1 << 6,
            Flag::H => 1 << 5,
            Flag::C => 1 << 4,
        };
        mask & self.f != 0
    }

    fn set_flag(&mut self, flag: Flag, val: bool) {
        let mask = match flag {
            Flag::Z => 1 << 7,
            Flag::N => 1 << 6,
            Flag::H => 1 << 5,
            Flag::C => 1 << 4,
        };
        match val {
            true => self.f |= mask,
            false => self.f &= !mask,
        }
    }
}
