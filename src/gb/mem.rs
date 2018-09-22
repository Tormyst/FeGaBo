use std::fs::File;
use std::io::Read;

const kb_8: usize = 0x2000;

pub trait MemMapper {
    fn read(&self, addr: u16) -> Option<u8>;
    fn write(&mut self, addr: u16, data: u8) -> bool;
}

mod cm {
    pub trait CartrageMapper {
        fn read(&self, addr: u16) -> Option<u8>;
        fn write(&mut self, addr: u16, data: u8) -> bool;
    }

    pub struct NoneCartrageMapper {}

    impl CartrageMapper for NoneCartrageMapper {
        fn read(&self, addr: u16) -> Option<u8> {
            None
        }
        fn write(&mut self, addr: u16, data: u8) -> bool {
            false
        }
    }
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

struct Oam {}

struct Io {}

pub struct Mem<T: MemMapper> {
    map_holder: T,
}

pub struct GbMapper {
    cartrage: Box<cm::CartrageMapper>,
    boot_rom: BootRom,
    vram: [u8; kb_8],
    wram: [u8; kb_8],
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
            oam: Oam {},
            io: Io {},
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
        println!("Boot rom loaded: {:x} bytes long", boot_size);

        GbMapper {
            cartrage: Box::new(cm::NoneCartrageMapper {}),
            boot_rom: BootRom::new(buffer),
            vram: [0; kb_8],
            wram: [0; kb_8],
            oam: Oam {},
            io: Io {},
            hram: [0; 126],
            ie: 0,
        }
    }
}

impl MemMapper for GbMapper {
    fn read(&self, addr: u16) -> Option<u8> {
        // Boot rom overlay
        if let Some(data) = self.boot_rom.read(addr) {
            return Some(data);
        }
        // Main table
        match addr {
            0x0000...0x7fff => (*self.cartrage).read(addr),
            _ => None,
        }
    }
    fn write(&mut self, addr: u16, data: u8) -> bool {
        false
    }
}

impl<T: MemMapper> Mem<T> {
    pub fn new(mapper: T) -> Self {
        Mem { map_holder: mapper }
    }

    pub fn load_8(&self, addr: u16) -> u8 {
        // Look value up in memory map
        match self.map_holder.read(addr) {
            Some(data) => data,
            None => {
                println!("Memory read failed for address: {:04x}.  Fallback to 0",
                         addr);
                0
            }
        }
    }
}
