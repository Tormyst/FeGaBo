/// The gameboy black and white pallets
/// located at memory FF47 to FF49
/// Each contains four colours expressed as 2 bit values as follows:
/// 0  White
/// 1  Light gray
/// 2  Dark gray
/// 3  Black
///
/// Bits representation: 33221100
/// Object pallets always contain transparent for colour 0.

macro_rules! copy3 {
    ($b:expr, $c:expr) => {
        {
            $b[0] = $c;
            $b[1] = $c;
            $b[2] = $c;
        }
    }
}

pub enum Pallet {
    BGP,
    OBP0,
    OBP1,
}

pub struct GBP {
    /// GB background pallet
    bgp: u8,
    /// GB first object pallet
    obp0: u8,
    /// GB second object pallet
    obp1: u8,
}

impl GBP {
    pub fn new() -> Self {
        GBP {bgp: 0, obp0: 0, obp1: 0}
    }
    pub fn read(&self, addr: u16) -> Option<u8> {
        match addr {
            0xFF47 => Some(self.bgp),
            0xFF48 => Some(self.obp0),
            0xFF49 => Some(self.obp1),
            _ => None,
        }
    }
    pub fn write(&mut self, addr: u16, data: u8) -> bool {
        match addr {
            0xFF47 => {self.bgp = data; true},
            0xFF48 => {self.obp0 = data; true},
            0xFF49 => {self.obp1 = data; true},
            _ => false,
        }
    }

    pub fn apply(&self, p: Pallet, color: u8, buffer: &mut [u8]){
        let p = match p { 
            Pallet::BGP => self.bgp,
            Pallet::OBP0 => self.obp0,
            Pallet::OBP1 => self.obp1,
        };
        let c = (p >> (color * 2)) & 0x03;
        match c {
            0 => copy3!(buffer, 255),
            1 => copy3!(buffer, 170),
            2 => copy3!(buffer, 85),
            3 => copy3!(buffer, 0),
            _ => unreachable!("There are only 4 colours"),
        };
    }
}

#[cfg(test)]
mod tests {
    use gb::mem::gbp::Pallet;
    use gb::mem::gbp::GBP;

    #[test]
    fn apply() {
        let pallet = GBP {bgp: 0, obp0: 0b00011011, obp1: 0b00000000};
        let mut buffer = vec![0,0,0];
        pallet.apply(Pallet::OBP0, 2, &mut buffer[..]);
        assert_eq!(buffer[0], 170);
        assert_eq!(buffer[1], 170);
        assert_eq!(buffer[2], 170);
        pallet.apply(Pallet::OBP1, 1, &mut buffer[..]);
        assert_eq!(buffer[0], 255);
        assert_eq!(buffer[1], 255);
        assert_eq!(buffer[2], 255);
    }
}
