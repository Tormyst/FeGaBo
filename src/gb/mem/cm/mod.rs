use std::fs::File;
use std::io::Read;

pub trait CartrageMapper {
    fn read(&self, addr: u16) -> Option<u8>;
    fn write(&mut self, addr: u16, data: u8) -> bool;
    fn read_ram(&self, addr: u16) -> Option<u8>;
    fn write_ram(&mut self, addr: u16, data: u8) -> bool;
}

pub fn new(cartrage: String) -> Box<CartrageMapper> {
    match File::open(cartrage) {
        Ok(mut file) => { 
            let mut buffer = Vec::new();
            let size = file.read_to_end(&mut buffer).unwrap();
            match size {
                0...0x10000 => Box::new(BaseROM { rom: buffer }),
                _ => panic!("AAAA cartrage too big."),
            }
        }
        Err(_) => Box::new(NoneCartrageMapper {}),
    }
}

pub struct BaseROM {
    rom: Vec<u8>,
}

impl CartrageMapper for BaseROM {
    fn read(&self, addr: u16) -> Option<u8> {
        Some(self.rom[addr as usize])
    }
    fn write(&mut self, _addr: u16, _data: u8) -> bool {
        eprintln!("Unexpected write to ROM.");
        true
    }

    fn read_ram(&self, _addr: u16) -> Option<u8> {
        None
    }
    fn write_ram(&mut self, _addr: u16, _data: u8) -> bool {
        false
    }
}

pub struct NoneCartrageMapper {}

impl CartrageMapper for NoneCartrageMapper {
    fn read(&self, _addr: u16) -> Option<u8> {
        None
    }
    fn write(&mut self, _addr: u16, _data: u8) -> bool {
        false
    }

    fn read_ram(&self, _addr: u16) -> Option<u8> {
        None
    }
    fn write_ram(&mut self, _addr: u16, _data: u8) -> bool {
        false
    }
}
