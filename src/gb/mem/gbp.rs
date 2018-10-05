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
///
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
}
