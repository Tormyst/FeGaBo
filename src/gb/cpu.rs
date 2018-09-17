use super::mem;

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

    pub fn cycle(&mut self, mem: &mut mem::Mem) {
        let opcode = self.load_pc_8(mem);
        println!("Executing 0x{:04X}: 0x{:02X}", self.pc - 1, opcode);
    }

    fn load_pc_8(&mut self, mem: &mem::Mem) -> u8 {
        // Memory load
        let ret = mem.load_8(self.pc);
        self.pc += 1;
        ret
    }

    fn read_16(&self, reg: WordRegister) -> u16 {
        let (high, low) =
        match reg {
            WordRegister::AF => (self.a, self.f),
            WordRegister::BC => (self.b, self.c),
            WordRegister::DE => (self.d, self.e),
            WordRegister::HL => (self.h, self.l),
        };
        ((high as u16) << 8) | low as u16
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
}
