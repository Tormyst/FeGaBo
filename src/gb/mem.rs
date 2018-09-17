const kb_8: usize = 0x2000;

pub trait MemMapper {
    fn read(&self, addr: u16) -> Option<u8>;
    fn write(&mut self, addr: u16, data: u8) -> bool;
}

mod CM {

    pub trait CartrageMapper {
        fn read(&self, addr: u16) -> u8;
        fn write(&mut self, addr: u16, data: u8) -> bool;
    }

    pub struct NoneCartrageMapper {}

    impl CartrageMapper for NoneCartrageMapper {
        fn read(&self, addr: u16) -> u8 {
            0
        }
        fn write(&mut self, addr: u16, data: u8) -> bool {
            false
        }
    }

}

trait BootRom {}

struct Oam {}

struct Io {}

pub struct Mem<T: MemMapper> {
    map_holder: T,
}

pub struct GbMapper {
    cartrage: Box<CM::CartrageMapper>,
    boot_rom: Option<Box<BootRom>>,
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
            cartrage: Box::new(CM::NoneCartrageMapper {}),
            boot_rom: None,
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
        None
    }
    fn write(&mut self, addr: u16, data: u8) -> bool {
        false
    }
}

impl<T: MemMapper> Mem<T> {
    pub fn new(mapper: T) -> Self {
        Mem { map_holder: mapper }
    }

    pub fn load_8(&self, location: u16) -> u8 {
        // Look value up in memory map
        0
    }
}
