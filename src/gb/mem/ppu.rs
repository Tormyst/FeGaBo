pub struct PPU {
    lcdc: u8, // 40
    stat: u8, // 41

    scy: u8, // 42
    scx: u8, // 43

    ly: u8, // 44
    lyc: u8, // 45

    wy: u8, //4A
    wx: u8, //4B
}

enum State {
    HBlank,
    VBlank,
    Oam,
    Vram,
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            lcdc: 0,
            stat: 0,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            wy: 0,
            wx: 0,
        }
    }
    pub fn read(&self, addr: u16) -> Option<u8> {
        println!("Reading from PPU at {:04X}", addr);
        match addr {
            0xFF40 => Some(self.lcdc),
            0xFF41 => Some(self.stat),
            0xFF42 => Some(self.scy),
            0xFF43 => Some(self.scx),
            0xFF44 => Some(self.ly),
            0xFF45 => Some(self.lyc),
            0xFF4A => Some(self.wy),
            0xFF4B => Some(self.wx),
            _ => None,
        }
    }
    pub fn write(&mut self, addr: u16, data: u8) -> bool {
        println!("Writing to PPU at {:4X} the value {:2X}", addr, data);
        match addr {
            0xFF40 => {self.lcdc; true},
            0xFF41 => {self.stat = data & 0xF8 | self.stat & 0x7; true},
            0xFF42 => {self.scy; true},
            0xFF43 => {self.scx; true},
            0xFF44 => false, // LY is read only
            0xFF45 => {self.lyc; true},
            0xFF4A => {self.wy; true},
            0xFF4B => {self.wx; true},
            _ => false,
        }
    }
    fn state(&self) -> State {
        match 0x02 & self.stat {
            0 => State::HBlank,
            1 => State::VBlank,
            2 => State::Oam,
            3 => State::Vram,
            _ => unreachable!("There are only 4 2bit states."),
        }
    }
}
