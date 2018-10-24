use super::decode;
use super::decode::{ByteR, Flag, WordR};
use super::mem;

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
    print: bool,
    state: CPUState,
}

#[derive(Debug)]
enum CPUState {
    Running,
    Halt,
    Stop,
}

impl Cpu {
    pub fn new() -> Self {
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
            print: false,
            state: CPUState::Running,
        }
    }

    pub fn new_after_boot() -> Self {
        let mut cpu = Cpu::new();
        cpu.write_16(WordR::AF, 0x01B0);
        cpu.write_16(WordR::BC, 0x0013);
        cpu.write_16(WordR::DE, 0x00D8);
        cpu.write_16(WordR::HL, 0x014D);
        cpu.sp = 0xFFFE;
        cpu.pc = 0x0100;
        cpu
    }

    pub fn handle_interupt(&mut self, location: u16, mem: &mut mem::Mem) {
        self.push(WordR::PC, mem);
        mem.set_ime(false);
        self.pc = location;
    }

    fn execute_op(&mut self, opcode: decode::Op, mem: &mut mem::Mem) {
        use gb::decode::Op::*;
        match opcode {
            NOP => {}
            HALT => self.state = CPUState::Halt,
            STOP => self.state = CPUState::Stop,

            CPL => {
                self.a = !self.a;
                self.set_flag(Flag::N, true);
                self.set_flag(Flag::H, true);
            }
            SCF => self.set_flag(Flag::C, true),
            CCF => self.set_flag(Flag::C, false),

            RST(op) => {
                self.push(WordR::PC, mem);
                self.pc = op;
            }

            RET(fl) => self.ret(fl, mem),
            RETI => {
                self.ret(decode::OptFlag(None), mem);
                mem.set_ime(true);
            }
            CALL(fl, o) => self.call(fl, o, mem),

            PUSH(o) => self.push(o, mem),
            POP(o) => {
                let data = self.pop(mem);
                self.write_16(o, data);
            }

            LD8(o1, o2) => {
                let data = self.read_8(o2, mem);
                self.write_8(o1, data, mem)
            }
            LD16(o1, o2) => {
                let data = self.read_16(o2);
                self.write_16(o1, data)
            }
            LDMem(o1, o2) => {
                let addr = self.read_16(o1);
                let data = self.read_16(o2);
                mem.write_16(addr, data);
            }

            INC8(o) => {
                // setup
                let before = self.read_8(o.clone(), mem);
                // add
                let data = before.wrapping_add(1);
                // write
                self.write_8(o, data, mem);
                self.set_flag(Flag::Z, data == 0);
                self.set_flag(Flag::N, false);
                self.set_flag(Flag::H, (before & 0x10) != (data & 0x10));
            }
            INC16(o) => {
                let data = self.read_16(o.clone()).wrapping_add(1);
                self.write_16(o, data)
            }

            DEC8(o) => {
                // setup
                let before = self.read_8(o.clone(), mem);
                // add
                let data = before.wrapping_sub(1);
                self.write_8(o, data, mem);

                self.set_flag(Flag::Z, data == 0);
                self.set_flag(Flag::N, true);
                self.set_flag(Flag::H, (before & 0x10) != (data & 0x10));
            }
            DEC16(o) => {
                let data = self.read_16(o.clone()).wrapping_sub(1);
                self.write_16(o, data)
            }

            ADD8(o) => self.add(o, mem, false),
            ADD16(o1, o2) => {
                let src = self.read_16(o2);
                let dest = self.read_16(o1.clone());

                let h = ((src & 0x0FFF) + (dest & 0x0FFF)) > 0x0FFF;
                let (result, c) = dest.overflowing_add(src);

                self.write_16(o1, result);

                self.set_flag(Flag::N, false);
                self.set_flag(Flag::H, h);
                self.set_flag(Flag::C, c);
            }
            ADC(o) => self.add(o, mem, true),

            SUB(o) => self.sub(o, mem, false),
            SBC(o) => self.sub(o, mem, false),

            AND(o) => self.and(o, mem),
            OR(o) => self.or(o, mem),
            XOR(o) => self.xor(o, mem),
            CP(o) => {
                self.cp(o, mem, false);
            }

            JR(fl, o) => self.jr(fl, o),
            JP(fl, o) => self.jp(fl, o),

            RL(op) => self.rl(op, mem),
            RLC(op) => self.rlc(op, mem),
            RLA => self.rl(ByteR::A, mem),
            RLCA => self.rlc(ByteR::A, mem),

            RR(op) => self.rr(op, mem),
            RRC(op) => self.rrc(op, mem),
            RRA => self.rr(ByteR::A, mem),
            RRCA => self.rrc(ByteR::A, mem),

            SLA(op) => self.sla(op, mem),
            SRA(op) => self.sra(op, mem),

            SWAP(op) => self.swap(op, mem),
            SRL(op) => self.srl(op, mem),

            BIT(n, o) => self.bit(n, o, mem),
            RES(n, o) => self.res(n, o, mem),
            SET(n, o) => self.set(n, o, mem),

            DI => mem.set_ime(false),
            EI => mem.set_ime(true),

            _ => panic!("Instruction {} not implemented.", opcode),
        }
    }

    pub fn cycle(&mut self, mem: &mut mem::Mem) -> usize {
        match self.state {
            CPUState::Running => self.cycle_running(mem),
            CPUState::Stop => {
                if mem.load_8(0xFF00) & 0x0F > 0 {
                    self.state = CPUState::Running;
                };
                4
            }
            CPUState::Halt => 4,
        }
    }

    pub fn cycle_running(&mut self, mem: &mut mem::Mem) -> usize {
        // Load opcode
        let (instruction, opcode, op_size, op_time) = decode::decode(self.pc, mem);
        // let mut flag = false;
        if self.print {
            // println!("CPU: {:0X?}", self);
            println!("Executing 0x{:04X}: {}    {}", self.pc, instruction, opcode);
        }
        // Change as debugging needed.
        else if self.pc == 0x0100 {
            self.print = true;
        }

        //Increment PC
        self.pc += op_size;

        // Execute
        self.execute_op(opcode, mem);

        op_time
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
            ByteR::F => self.f = data & 0xF0,
            ByteR::IMM(_) => panic!("You cannot write to a immediate value"),
            ByteR::Mem(addr) => mem.write_8(self.read_16(addr), data),
        }
    }

    fn read_16(&mut self, reg: WordR) -> u16 {
        match reg {
            WordR::SP => self.sp,
            WordR::SPP(offset) => self.sp.wrapping_add(offset as u16),
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
                let data = ((high as u16) << 8) | low as u16;
                // Post read operation for increment and decrement
                match reg {
                    WordR::HLI => self.write_16(reg, data + 1),
                    WordR::HLD => self.write_16(reg, data - 1),
                    _ => {}
                }
                data
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
                self.f = low & 0xF0;
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

    fn flag_condition(&self, fl: decode::OptFlag) -> bool {
        match fl {
            decode::OptFlag(None) => true,
            decode::OptFlag(Some((flag, state))) => self.read_flag(flag) == state,
        }
    }

    fn ret(&mut self, fl: decode::OptFlag, mem: &mut mem::Mem) {
        if self.flag_condition(fl) {
            self.pc = self.pop(mem);
        }
    }

    fn call(&mut self, fl: decode::OptFlag, reg: WordR, mem: &mut mem::Mem) {
        if self.flag_condition(fl) {
            self.push(WordR::PC, mem);
            self.pc = self.read_16(reg);
        }
    }

    fn push(&mut self, reg: WordR, mem: &mut mem::Mem) {
        let data = self.read_16(reg);
        self.sp = self.sp.wrapping_sub(2);
        mem.write_16(self.sp, data);
    }

    fn pop(&mut self, mem: &mut mem::Mem) -> u16 {
        let data = mem.load_16(self.sp);
        self.sp = self.sp.wrapping_add(2);
        data
    }

    fn and(&mut self, reg: ByteR, mem: &mut mem::Mem) {
        self.a = self.read_8(reg, mem) & self.a;
        let zero = self.a == 0;
        self.set_flag(Flag::Z, zero);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, true);
        self.set_flag(Flag::C, false);
    }

    fn or(&mut self, reg: ByteR, mem: &mut mem::Mem) {
        self.a = self.read_8(reg, mem) | self.a;
        let zero = self.a == 0;
        self.set_flag(Flag::Z, zero);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::C, false);
    }

    fn xor(&mut self, reg: ByteR, mem: &mut mem::Mem) {
        self.a = self.read_8(reg, mem) ^ self.a;
        let zero = self.a == 0;
        self.set_flag(Flag::Z, zero);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::C, false);
    }

    fn add(&mut self, reg: ByteR, mem: &mut mem::Mem, with_c: bool) {
        let reg = self.read_8(reg, mem);

        let cin = match with_c {
            false => 0,
            true => {
                match self.read_flag(Flag::C) {
                    true => 1,
                    false => 0,
                }
            }
        };

        let (data, cout) = self.a.overflowing_add(reg + cin);
        let h = (data & 0x0F) < ((reg & 0x0F) + cin);
        let zero = data == 0;
        self.set_flag(Flag::Z, zero);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, h);
        self.set_flag(Flag::C, cout);
        self.a = data;
    }

    fn sub(&mut self, reg: ByteR, mem: &mut mem::Mem, with_c: bool) {
        self.a = self.cp(reg, mem, with_c);
    }

    fn cp(&mut self, reg: ByteR, mem: &mut mem::Mem, with_c: bool) -> u8 {
        let reg = self.read_8(reg, mem);

        let cin = match with_c {
            false => 0,
            true => {
                match self.read_flag(Flag::C) {
                    true => 1,
                    false => 0,
                }
            }
        };

        let data = self.a.wrapping_sub(reg + cin);
        let h = (data & 0x0F) < ((reg & 0x0F) + cin);
        let zero = data == 0;
        let cout = self.a < reg;
        self.set_flag(Flag::Z, zero);
        self.set_flag(Flag::N, true);
        self.set_flag(Flag::H, h);
        self.set_flag(Flag::C, cout);
        data
    }

    fn jr(&mut self, fl: decode::OptFlag, o: i8) {
        if self.flag_condition(fl) {
            self.pc = self.pc.wrapping_add(o as u16);
        }
    }

    fn jp(&mut self, fl: decode::OptFlag, o: WordR) {
        if self.flag_condition(fl) {
            self.pc = self.read_16(o);
        }
    }

    fn rl(&mut self, reg: ByteR, mem: &mut mem::Mem) {
        // setup
        let regval = self.read_8(reg.clone(), mem);
        let old_c = self.read_flag(Flag::C);

        // rotate
        self.set_flag(Flag::C, regval & 0x80 != 0);
        let valout = (regval << 1) |
                     match old_c {
                         true => 1,
                         false => 0,
                     };

        // output
        self.set_flag(Flag::Z, valout == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
        self.write_8(reg, valout, mem);
    }

    fn rlc(&mut self, reg: ByteR, mem: &mut mem::Mem) {
        let regval = self.read_8(reg.clone(), mem).rotate_left(1);

        self.set_flag(Flag::C, regval & 0x01 != 0);
        self.set_flag(Flag::Z, regval == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
        self.write_8(reg, regval, mem);
    }

    fn rr(&mut self, reg: ByteR, mem: &mut mem::Mem) {
        // setup
        let regval = self.read_8(reg.clone(), mem);
        let old_c = self.read_flag(Flag::C) as u8;

        // rotate
        self.set_flag(Flag::C, regval & 0x01 != 0);
        let valout = (regval >> 1) | (old_c << 7);

        // output
        self.set_flag(Flag::Z, valout == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
        self.write_8(reg, valout, mem);
    }

    fn rrc(&mut self, reg: ByteR, mem: &mut mem::Mem) {
        let regval = self.read_8(reg.clone(), mem).rotate_right(1);

        self.set_flag(Flag::C, regval & 0x80 != 0);
        self.set_flag(Flag::Z, regval == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
        self.write_8(reg, regval, mem);
    }

    fn sla(&mut self, reg: ByteR, mem: &mut mem::Mem) {
        let data = self.read_8(reg.clone(), mem);
        self.set_flag(Flag::C, data & 0xF0 != 0);
        let regval = data << 1;

        self.set_flag(Flag::Z, regval == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
        self.write_8(reg, regval, mem);
    }

    fn sra(&mut self, reg: ByteR, mem: &mut mem::Mem) {
        let data = self.read_8(reg.clone(), mem);
        self.set_flag(Flag::C, data & 0x01 != 0);
        let regval = data >> 1;

        self.set_flag(Flag::Z, regval == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
        self.write_8(reg, regval, mem);
    }

    fn swap(&mut self, reg: ByteR, mem: &mut mem::Mem) {
        let regval = self.read_8(reg.clone(), mem).rotate_left(4);

        self.set_flag(Flag::Z, regval == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::C, false);
        self.write_8(reg, regval, mem);
    }

    fn srl(&mut self, reg: ByteR, mem: &mut mem::Mem) {
        let data = self.read_8(reg.clone(), mem);
        self.set_flag(Flag::C, data & 0x01 != 0);
        let regval = (data >> 1) & 0x7F;

        self.set_flag(Flag::Z, regval == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
        self.write_8(reg, regval, mem);
    }

    fn bit(&mut self, n: u8, reg: ByteR, mem: &mut mem::Mem) {
        let zero = self.read_8(reg, mem) & (0x1 << n) == 0;
        self.set_flag(Flag::Z, zero);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, true);
    }

    fn set(&mut self, n: u8, reg: ByteR, mem: &mut mem::Mem) {
        let mask = 1 << n;
        let data = self.read_8(reg.clone(), mem) | mask;

        self.write_8(reg, data, mem);
    }

    fn res(&mut self, n: u8, reg: ByteR, mem: &mut mem::Mem) {
        let mask = 1 << n;
        let data = self.read_8(reg.clone(), mem) & !mask;

        self.write_8(reg, data, mem);
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
