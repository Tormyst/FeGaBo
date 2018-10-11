
macro_rules! get_bit {
    ($data:expr, $bit:expr) => {
        (($data >> $bit) & 0x01) > 0
    }
}

const LINE_CYCLE: usize = 456;
const LYMAX: u8 = 153;


pub struct PPU {
    lcdc: u8, // 40
    stat: u8, // 41

    pub scy: u8, // 42
    pub scx: u8, // 43

    ly: u8, // 44
    lx: usize, // Hidden value used to tell where we are in the write cycle
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
            lx: 0,
            lyc: 0,
            wy: 0,
            wx: 0,
        }
    }
    pub fn read(&self, addr: u16) -> Option<u8> {
        //println!("Reading from PPU at {:04X}", addr);
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
        //println!("Writing to PPU at {:4X} the value {:2X}", addr, data);
        match addr {
            0xFF40 => {self.lcdc_set(data)},
            0xFF41 => {self.stat = data & 0xF8 | self.stat & 0x7; true},
            0xFF42 => {self.scy = data; true},
            0xFF43 => {self.scx = data; true},
            0xFF44 => {self.ly = 0; true}, // LY is read only.  Writing resets the value
            0xFF45 => {self.lyc; true},
            0xFF4A => {self.wy; true},
            0xFF4B => {self.wx; true},
            _ => false,
        }
    }

    fn lcdc_set(&mut self, data: u8) -> bool{
        // If bit 7 is set, then we have to set up for display.  This cannot be done during a
        // frame, only during vblank.
        if get_bit!(self.lcdc, 7) == false && get_bit!(data, 7) == true {
            self.ly = 0;
            self.lx = 0;
            self.set_state();
        }
        else if get_bit!(self.lcdc, 7) == true && get_bit!(data, 7) == false {
            match self.state() {
                State::VBlank => panic!("Turning off screen during vblank.  Gameboy crashes."),
                _ => {}
            }
        }
        self.lcdc = data;
        println!("Setting LCDC: {:08b}", self.lcdc);
        true
    }

    pub fn lcdc_get(&self, offset: u8) -> bool {
        self.lcdc & (0x01 << offset) != 0
    }

    pub fn time_passes(&mut self, time: usize) -> Option<Vec<u8>>{
        self.lx += time;
        if self.lx > LINE_CYCLE {
            let mut ret = vec![];
            while self.lx > LINE_CYCLE {
                ret.push(self.ly);
                self.ly += 1;
                self.lx -= 456;

                if self.ly > LYMAX {
                    self.ly -= LYMAX;
                    ret.push(0);
                }
            }
            self.set_state();
            Some(ret)
        }
        else {
            self.set_state();
            None
        }
    }

    fn set_state(&mut self) {
        if self.ly >= 144 {
            self.stat = self.stat & 0xF8 | 0x01; // Vblank
        }
        else {
            self.stat = match self.lx {
                0...77 => self.stat & 0xF8 | 0x02,
                78...247 => self.stat & 0xF8 | 0x03,
                248...456 => self.stat & 0xF8 | 0x00,
                _ => unreachable!("There should be no value of lx great then 456"),
            }
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
