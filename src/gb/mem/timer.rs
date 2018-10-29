pub struct Timer {
    div: u8,
    tima: u8,
    tma: u8,
    tac: u8,

    buffered_interupt: bool,
    subclock: usize,
    subdiv: usize,
}

impl Timer {
    pub fn new() -> Self {
        Timer { div: 0, tima: 0, tma: 0, tac: 0, buffered_interupt: false, subclock: 0, subdiv: 0, }
    }

    pub fn read(&self, addr: u16) -> Option<u8> {
        match addr {
            0xFF04 => Some(self.div),
            0xFF05 => Some(self.tima),
            0xFF06 => Some(self.tma),
            0xFF07 => Some(self.tac),
            _ => None,
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) -> bool {
        match addr {
            0xFF04 => { self.div = 0x00; true},
            0xFF05 => { self.tima = data; true},
            0xFF06 => { self.tma = data; true},
            0xFF07 => { self.tac = data; true},
            _ => false,
        }
    }

    pub fn check_interupt(&mut self) -> u8 {
        if self.buffered_interupt {
            self.buffered_interupt = false;
            0x04
        }
        else { 0x00 }
    }

    pub fn tick(&mut self, time: usize) {
        // DIV
        self.subdiv += time;
        if self.subdiv >= 256 {
            self.subdiv -= 256;
            self.div = self.div.wrapping_add(1);
        };
        // TIMA
        if (self.tac & 0x04) > 0 {
            self.subclock += time;
            let threshold = match self.tac & 0x03 {
                0 => 1024,
                1 => 16,
                2 => 64,
                3 => 256,
                _ => unreachable!("value should only be two bytes"),
            };
            if self.subclock >= threshold {
                self.subclock -= threshold;
                let (new_tima, overflow) = self.tima.overflowing_add(1);
                self.tima = new_tima;
                if overflow {
                    self.tima = self.tma;
                    self.buffered_interupt = true;
                }
            }
        }
    }
}
