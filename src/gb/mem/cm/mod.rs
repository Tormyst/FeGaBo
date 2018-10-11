use std::fs::File;
use std::io::Read;

pub trait CartrageMapper {
    fn read(&self, addr: u16) -> Option<u8>;
    fn write(&mut self, addr: u16, data: u8) -> bool;
    fn read_ram(&self, addr: u16) -> Option<u8>;
    fn write_ram(&mut self, addr: u16, data: u8) -> bool;
}

pub fn new(cartrage: String) -> Box<CartrageMapper> {
    let mut buffer = Vec::new();
    let size = File::open(cartrage)
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();
    Box::new(BaseROM { rom: buffer })
}

pub struct BaseROM {
    rom: Vec<u8>,
}

impl CartrageMapper for BaseROM {
    fn read(&self, addr: u16) -> Option<u8> {
        Some(self.rom[addr as usize])
    }
    fn write(&mut self, addr: u16, data: u8) -> bool {
        false
    }

    fn read_ram(&self, addr: u16) -> Option<u8> {
        None
    }
    fn write_ram(&mut self, addr: u16, data: u8) -> bool {
        false
    }
}

pub struct NoneCartrageMapper {}

impl CartrageMapper for NoneCartrageMapper {
    fn read(&self, addr: u16) -> Option<u8> {
        Some(0x00)
    }
    fn write(&mut self, addr: u16, data: u8) -> bool {
        false
    }

    fn read_ram(&self, addr: u16) -> Option<u8> {
        None
    }
    fn write_ram(&mut self, addr: u16, data: u8) -> bool {
        false
    }
}
