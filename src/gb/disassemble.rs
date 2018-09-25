use super::mem::Mem;

macro_rules! p0 {
    ($format:expr) => {
        {
        println!($format);
        1
        }
    }
}

macro_rules! p8 {
    ($format:expr, $addr:expr, $mem:expr) => {
        {
        println!($format, $mem.load_8($addr + 1));
        2
        }
    }
}

macro_rules! p16 {
    ($format:expr, $addr:expr, $mem:expr) => {
        {
        println!($format, $mem.load_16($addr + 1));
        3
        }
    }
}

pub fn daInst(addr: u16, mem: &mut Mem) -> usize {
    let op = mem.load_8(addr);
    match op {
        0x00 => p0!("NOP"),
        0x01 => p16!("LD BC, 0x{:04X}", addr, mem),
        0x02 => p0!("LD (BC),A"),
        0x03 => p0!("INC BC"),

        0x11 => p16!("LD DE, 0x{:04X}", addr, mem),
        0x12 => p0!("LD (DE),A"),
        0x13 => p0!("INC DE"),

        0x21 => p16!("LD HL, 0x{:04X}", addr, mem),
        0x22 => p0!("LD (HL+),A"),
        0x23 => p0!("INC HL"),

        0x31 => p16!("LD SP, 0x{:04X}", addr, mem),
        0x32 => p0!("LD (HL-),A"),
        0x33 => p0!("INC SP"),

        0xAF => p0!("XOR A"),
        _ => panic!("Unknown instruction {:02X}", op),
    }
}
