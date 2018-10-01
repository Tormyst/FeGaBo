pub trait CartrageMapper {
    fn read(&self, addr: u16) -> Option<u8>;
    fn write(&mut self, addr: u16, data: u8) -> bool;
    fn read_ram(&self, addr: u16) -> Option<u8>;
    fn write_ram(&mut self, addr: u16, data: u8) -> bool;
}

pub struct NoneCartrageMapper {}

impl CartrageMapper for NoneCartrageMapper {
    fn read(&self, addr: u16) -> Option<u8> {
        None
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
