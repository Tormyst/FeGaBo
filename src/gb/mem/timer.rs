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
            0xFF04 => { self.div = 0x00; self.subdiv = 0x00; self.subclock = 0x00; true},
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
        let threshold = match self.tac & 0x03 {
            0 => 9,
            1 => 3,
            2 => 5,
            3 => 7,
            _ => unreachable!("value should only be two bytes"),
        };
        let mut tac_key = (self.subdiv >> threshold) & 0x01 > 0;
        let tac_ena = (self.tac & 0x04) > 0;

        for _ in 0..time {
            // DIV
            self.subdiv += 1;
            let new_tac_key = (self.subdiv >> threshold) & 0x01 > 0;

            if  tac_ena && tac_key && !new_tac_key {
                //TIMA Increment
                let (new_tima, overflow) = self.tima.overflowing_add(1);
                self.tima = new_tima;
                if overflow {
                    self.tima = self.tma;
                    self.buffered_interupt = true;
                }
            };

            tac_key = new_tac_key;
        }
        self.div = (self.subdiv >> 8 & 0xFF) as u8;
    }
}
