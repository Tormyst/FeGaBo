use std::fs::File;
use std::io::Read;

mod ppu;
mod cm;

const kb_8: usize = 0x2000;

pub trait MemMapper {
    fn read(&self, addr: u16) -> Option<u8>;
    fn write(&mut self, addr: u16, data: u8) -> bool;
}


struct BootRom {
    rom: Vec<u8>,
}

impl BootRom {
    fn new(boot: Vec<u8>) -> Self {
        BootRom { rom: boot }
    }
    fn read(&self, addr: u16) -> Option<u8> {
        match self.rom.get(addr as usize) {
            Some(data) => Some(*data as u8),
            None => None,
        }
    }
    fn write(&mut self, addr: u16, data: u8) -> bool {
        false
    }
}

#[derive(Copy, Clone)]
struct OamEntry{y:u8, x:u8, t:u8, a:u8}

const defaultOamEntry: OamEntry = OamEntry{y:0, x:0, t:0, a:0};

impl OamEntry {
    fn write(&mut self, index: usize, data: u8) -> bool{
        match index {
            0 => self.y = data,
            1 => self.x = data,
            2 => self.t = data,
            3 => self.a = data,
            _ => unreachable!("Only 2 bits used"),
        }
        true
    }

    fn read(&self, index: usize) -> u8{
        match index {
            0 => self.y,
            1 => self.x,
            2 => self.t,
            3 => self.a,
            _ => unreachable!("Only 2 bits used"),
        }
    }
}

struct Oam {
    data: [OamEntry;40],
}

impl Oam {
    fn new() -> Self {
        Oam { data: [defaultOamEntry;40] }
    }

    fn entryNum(addr: u16) -> (usize, usize) {
        ((addr & 0x00FC) as usize, (addr & 0x0003) as usize)
    }

    fn read(&self, addr: u16) -> Option<u8> {
        let (e, n) = Oam::entryNum(addr);
        Some(self.data[e].read(n))
    }
    fn write(&mut self, addr: u16, data: u8) -> bool {
        let (e, n) = Oam::entryNum(addr);
        self.data[e].write(n, data)
    }
}

struct Io {
    ppu: ppu::PPU,
}

impl Io {
    fn new() -> Self {
        Io {ppu: ppu::PPU::new()}
    }

    fn read(&self, addr: u16) -> Option<u8> {
        match addr {
            0xFF00 => {None} // Joypad
            0xFF10...0xFF3F => {Some(0x00)} // Audio device not implemented.
            0xFF40...0xFF45 => {self.ppu.read(addr)} // PPU state 
            _ => panic!("Unknown IO port read at {:X}", addr)
        }
    }

    fn write(&mut self, addr: u16, data: u8) -> bool {
        match addr {
            0xFF00 => {false} // Joypad
            0xFF10...0xFF3F => {true} // Audio device not implemented.
            0xFF40...0xFF45 => {self.ppu.write(addr, data)} // PPU state
            _ => panic!("Unknown IO port write at {:X}", addr)
        }
    }
}

pub struct Mem {
    map_holder: Box<MemMapper>,
}

pub struct GbMapper {
    cartrage: Box<cm::CartrageMapper>,
    boot_rom: BootRom,
    vram: [u8; kb_8],
    wram: [u8; kb_8],
    boot: bool,
    oam: Oam,
    io: Io,
    hram: [u8; 126],
    ie: u8,
}

impl GbMapper {
    pub fn new() -> Self {
        GbMapper {
            cartrage: Box::new(cm::NoneCartrageMapper {}),
            boot_rom: BootRom::new(vec![]),
            vram: [0; kb_8],
            wram: [0; kb_8],
            boot: false,
            oam: Oam::new(),
            io: Io::new(),
            hram: [0; 126],
            ie: 0,
        }
    }

    pub fn new_with_boot_rom(boot_rom: String) -> Self {
        let mut buffer = Vec::new();
        let boot_size = File::open(boot_rom)
            .unwrap()
            .read_to_end(&mut buffer)
            .unwrap();
        println!("Boot rom loaded: {:X} bytes long", boot_size);

        GbMapper {
            cartrage: Box::new(cm::NoneCartrageMapper {}),
            boot_rom: BootRom::new(buffer),
            vram: [0; kb_8],
            wram: [0; kb_8],
            boot: false,
            oam: Oam::new(),
            io: Io::new(),
            hram: [0; 126],
            ie: 0,
        }
    }
}

impl MemMapper for GbMapper {
    fn read(&self, addr: u16) -> Option<u8> {
        // Main table
        match addr {
            0x0000...0x00FF => if !self.boot { self.boot_rom.read(addr) } else { (*self.cartrage).read(addr) }
            0x0000...0x7FFF => (*self.cartrage).read(addr),
            0x8000...0x9FFF => Some(self.vram[addr as usize & 0x0FFF]),
            0xA000...0xBFFF => (*self.cartrage).read_ram(addr),
            0xC000...0xDFFF => Some(self.wram[addr as usize & 0x0FFF]),
            0xE000...0xFDFF => Some(self.wram[addr as usize & 0x0FFF]),
            0xFE00...0xFE9F => self.oam.read(addr),
            0xFEA0...0xFEFF	=> None, // Not Usable
            0xFF00...0xFF4F => self.io.read(addr), // I/O Registers
            0xFF50 => Some(match self.boot{true => 0xFE, false => 0xFF}),
            0xFF51...0xFF7F	=> self.io.read(addr), //more I/O Registers
            0xFF80...0xFFFE => Some(self.hram[addr as usize & 0x007F]),
            0xFFFF => Some(self.ie),
            _ => None,
        }
    }
    fn write(&mut self, addr: u16, data: u8) -> bool {
        match addr {
            // Main table
            0x0000...0x7FFF => (*self.cartrage).write(addr, data),
            0x8000...0x9FFF => {self.vram[addr as usize & 0x0FFF] = data; true}
            0xA000...0xBFFF => (*self.cartrage).write_ram(addr, data),
            0xC000...0xDFFF => {self.wram[addr as usize & 0x0FFF] = data; true}
            0xE000...0xFDFF => {self.wram[addr as usize & 0x0FFF] = data; true}
            0xFE00...0xFE9F => self.oam.write(addr, data),
            0xFEA0...0xFEFF	=> false, // Not Usable
            0xFF00...0xFF4F	=> self.io.write(addr, data), //I/O Registers
            0xFF50 => {self.boot = (0x01 & data) > 0; true},
            0xFF51...0xFF7F	=> self.io.write(addr, data), //I/O Registers
            0xFF80...0xFFFE => {self.hram[addr as usize & 0x007F] = data; true}
            0xFFFF => {self.ie = data; true},
            _ => false,
        }
    }
}

impl Mem {
    pub fn new_gb(mapper: GbMapper) -> Self {
        Mem { map_holder: Box::new(mapper) }
    }

    pub fn load_8(&self, addr: u16) -> u8 {
        // Look value up in memory map
        match self.map_holder.read(addr) {
            Some(data) => data,
            None => {
                panic!("Memory read failed for address: {:04X}.",
                         addr);
            }
        }
    }

    pub fn write_8(&mut self, addr: u16, data: u8) {
        // Look value up in memory map
        if !self.map_holder.write(addr, data) {
            panic!("Memory write failed for address: {:04X}", addr)
        }
    }

    pub fn write_16(&mut self, addr: u16, data: u16) {
        // Look value up in memory map
        self.write_8(addr, data as u8);
        self.write_8(addr + 1, (data >> 8) as u8);
    }

    pub fn load_16(&self, addr: u16) -> u16 {
        // Look value up in memory map

        let low: u16 = self.load_8(addr) as u16;
        let high: u16 = self.load_8(addr + 1) as u16;

        (high << 8) + low
    }
}
