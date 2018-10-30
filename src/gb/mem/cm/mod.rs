use std::fs::File;
use std::io::Read;

#[derive(Debug)]
enum CMtype {
    ROM,
    MBC1,
    MBC2,
    MBC3,
}

pub struct CartrageMapper {
    cm_chip: CMtype,
    rom_page: u8,
    ram_page: u8,
    ram_enable: bool,
    mbc_mode: usize,
    rom: Vec<u8>,
    ram: Vec<u8>,
}
impl CartrageMapper {
    fn ram_size(r: &u8) -> usize {
        match r {
            0 => 0,
            1 => 0x0800,
            2 => 0x2000,
            3 => 0x8000,
            _ => panic!("Unknown ram size")
        }
    }
    fn cm_type(t: &u8, r: &u8) -> (CMtype, Option<usize>) {
        match t {
            0x00 => (CMtype::ROM, None),
            0x01 => (CMtype::MBC1, None),
            0x02 => (CMtype::MBC1, Some(Self::ram_size(r))),
            0x03 => (CMtype::MBC1, Some(Self::ram_size(r))),
            0x05...0x06 => (CMtype::MBC2, None),
            0x08...0x09 => (CMtype::ROM, None),
            0x10...0x13 => (CMtype::MBC3, None),
            _ => panic!("Unknown Cartrage Mapper"),
        }
    }

    pub fn new(cartrage: String) -> CartrageMapper {
        use std::str;
        match File::open(cartrage) {
            Ok(mut file) => {
                let mut rom = Vec::new();
                let size = file.read_to_end(&mut rom).unwrap();

                println!("Header Title: {}",
                        str::from_utf8(&rom[0x134..0x13E]).unwrap());
                println!("Header Manufacturer Code: {}",
                        str::from_utf8(&rom[0x13F..0x142]).unwrap());

                let (cm_chip, ram_type) = Self::cm_type(&rom[0x147], &rom[0x148]);
                println!("Chip type: {:?}", cm_chip);

                let mut ram = match ram_type {
                    Some(size) => { println!("Ram size: {}", size); vec![0;size] },
                    None => { println!("No RAM"); vec![] },
                };

                CartrageMapper {
                    cm_chip,
                    rom,
                    ram,
                    rom_page: 1,
                    ram_page: 0,
                    mbc_mode: 0,
                    ram_enable: false,
                }
            }
            Err(_) => panic!("File dose not exist"),
        }
    }
    pub fn read(&self, addr: u16) -> Option<u8> {
        if addr < 0x4000 {
            Some(self.rom[addr as usize])
        }
        else if addr < 0x8000 {
            let addr = (addr as usize) - 0x4000 + (0x4000 * self.rom_page as usize);
            println!("Effective address {:04X} from page {:02X}", addr, self.rom_page);
            Some(self.rom[addr])
        }
        else { None }
    }
    pub fn write(&mut self, addr: u16, data: u8) -> bool {
        println!("Write to ROM at {:04X}, with data {:02X}", addr, data);
        match self.cm_chip {
            CMtype::ROM => println!("Unexpected write to ROM."),
            CMtype::MBC1 => match addr {
                    0x0000..=0x1FFF => self.ram_enable = data & 0x0F == 0x0A,
                    0x2000..=0x3FFF => {
                        self.rom_page = (self.rom_page & 0xE0) | (data & 0x1F);
                        if self.rom_page & 0x1F == 0 { self.rom_page += 1; }
                        println!("ROM bank {:X} loaded using data {:02X} and addr {:04X}", 
                                 self.rom_page,
                                 data,
                                 addr);
                    }
                    0x4000..=0x5FFF => match self.mbc_mode {
                        0 => {
                            self.rom_page = (data << 5) & 0x60 | self.rom_page & 0x1F;
                            if self.rom_page & 0x1F == 0 { self.rom_page += 1; }
                        }
                        1 => self.ram_page = data & 0x03,
                        _ => panic!("unknown mode"),
                    }
                    0x6000..=0x7FFF => {
                        self.mbc_mode = (data & 0x01) as usize;
                        match self.mbc_mode {
                            0 => self.ram_page = 0,
                            1 => self.rom_page = self.rom_page & 0x1F,
                            _ => panic!("unknown mode"),
                        }
                    }
                    _ => unreachable!("Only 0 to 7FFF is addressable"),
                }
            _ => panic!("Unimplemented chip type {:?}", self.cm_chip),
        }
        true
    }

    pub fn read_ram(&self, addr: u16) -> Option<u8> {
        let addr = (addr as usize - 0xA000) + (self.ram_page as usize * 0x2000);
        match self.ram.get(addr as usize) {
            Some(data) => Some(*data),
            None => None,
        }
    }
    pub fn write_ram(&mut self, addr: u16, data: u8) -> bool {
        let addr = (addr as usize - 0xA000) + (self.ram_page as usize * 0x2000);
        if let Some(mem) = self.ram.get_mut(addr) {
            *mem = data;
            true
        }
        else { false }
    }
}
