pub struct PPU {
    lcdc: u8, // 40
    stat: u8, // 41

    scy: u8, // 42
    scx: u8, // 43

    ly: u8, // 44
    lyc: u8, // 45
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
        }
    }
    pub fn read(&self, addr: u16) -> Option<u8> {
        match addr {
            0xFF40 => Some(self.lcdc),
            0xFF41 => Some(self.stat),
            0xFF42 => Some(self.scy),
            0xFF43 => Some(self.scx),
            0xFF44 => Some(self.ly),
            0xFF45 => Some(self.lyc),
            _ => None,
        }
    }
    pub fn write(&mut self, addr: u16, data: u8) -> bool {
        false
    }
}
