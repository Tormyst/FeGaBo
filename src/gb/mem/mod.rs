use std::fs::File;
use std::io::Read;

use ::{GAMEBOY_WIDTH, GAMEBOY_HEIGHT};

mod ppu;
mod gbp;
mod cm;

const kb_8: usize = 0x2000;
const gameboy_screen_buffer_size: u32 = GAMEBOY_WIDTH * GAMEBOY_HEIGHT * 3;

pub trait MemMapper {
    fn read(&self, addr: u16) -> Option<u8>;
    fn write(&mut self, addr: u16, data: u8) -> bool;
    fn time_passes(&mut self, time: usize) -> Option<Vec<u8>>;
    fn render(&self, row: u8, buffer: &mut [u8]);
    fn print_background_map(&self);
    fn print_sprite_table(&self);
}


struct BootRom {
    rom: Vec<u8>,
}

impl BootRom {
    fn new(boot: Vec<u8>) -> Self {
        BootRom { rom: boot }
    }
    fn read(&self, addr: u16) -> Option<u8> {
        match self.rom.get(addr as usize) {
            Some(data) => Some(*data as u8),
            None => None,
        }
    }
    fn write(&mut self, addr: u16, data: u8) -> bool {
        false
    }
}

#[derive(Copy, Clone)]
struct OamEntry{y:u8, x:u8, t:u8, a:u8}

const defaultOamEntry: OamEntry = OamEntry{y:0, x:0, t:0, a:0};

impl OamEntry {
    fn write(&mut self, index: usize, data: u8) -> bool{
        match index {
            0 => self.y = data,
            1 => self.x = data,
            2 => self.t = data,
            3 => self.a = data,
            _ => unreachable!("Only 2 bits used"),
        }
        true
    }

    fn read(&self, index: usize) -> u8{
        match index {
            0 => self.y,
            1 => self.x,
            2 => self.t,
            3 => self.a,
            _ => unreachable!("Only 2 bits used"),
        }
    }
}

struct Oam {
    data: [OamEntry;40],
}

impl Oam {
    fn new() -> Self {
        Oam { data: [defaultOamEntry;40] }
    }

    fn entryNum(addr: u16) -> (usize, usize) {
        (((addr >> 2) & 0x003F) as usize, (addr & 0x0003) as usize)
    }

    fn read(&self, addr: u16) -> Option<u8> {
        let (e, n) = Oam::entryNum(addr);
        Some(self.data[e].read(n))
    }
    fn write(&mut self, addr: u16, data: u8) -> bool {
        let (e, n) = Oam::entryNum(addr);
        self.data[e].write(n, data)
    }
}

pub struct Mem {
    map_holder: Box<MemMapper>,
    screen: Box<[u8; gameboy_screen_buffer_size as usize]>,
    ime: bool,
}

pub struct GbMapper {
    cartrage: Box<cm::CartrageMapper>,
    boot_rom: BootRom,
    vram: [u8; kb_8],
    wram: [u8; kb_8],
    boot: bool,
    oam: Oam,
    hram: [u8; 127],
    interupt_enable: u8,
    interupt_flag: u8,
    ppu: ppu::PPU,
    gbp: gbp::GBP,
}

impl GbMapper {
    pub fn new(cartrage: String) -> Self {
        GbMapper {
            cartrage: cm::new(cartrage),
            boot_rom: BootRom::new(vec![]),
            vram: [0; kb_8],
            wram: [0; kb_8],
            boot: false,
            oam: Oam::new(),
            hram: [0; 127],
            interupt_enable: 0,
            interupt_flag: 0,
            ppu: ppu::PPU::new(),
            gbp: gbp::GBP::new(),
        }
    }

    pub fn new_with_boot_rom(boot_rom: String, cartrage: String) -> Self {
        let mut buffer = Vec::new();
        let boot_size = File::open(boot_rom)
            .unwrap()
            .read_to_end(&mut buffer)
            .unwrap();
        println!("Boot rom loaded: {:X} bytes long", boot_size);

        GbMapper {
            cartrage: cm::new(cartrage),
            boot_rom: BootRom::new(buffer),
            vram: [0; kb_8],
            wram: [0; kb_8],
            boot: false,
            oam: Oam::new(),
            hram: [0; 127],
            interupt_enable: 0,
            interupt_flag: 0,
            ppu: ppu::PPU::new(),
            gbp: gbp::GBP::new(),
        }
    }
}

impl MemMapper for GbMapper {
    fn read(&self, addr: u16) -> Option<u8> {
        // Main table
        match addr {
            0x0000...0x00FF => if !self.boot { self.boot_rom.read(addr) } else { (*self.cartrage).read(addr) }
            0x0000...0x7FFF => (*self.cartrage).read(addr),
            0x8000...0x9FFF => Some(self.vram[addr as usize & 0x0FFF]),
            0xA000...0xBFFF => (*self.cartrage).read_ram(addr),
            0xC000...0xDFFF => Some(self.wram[addr as usize & 0x0FFF]),
            0xE000...0xFDFF => Some(self.wram[addr as usize & 0x0FFF]),
            0xFE00...0xFE9F => self.oam.read(addr),
            // 0xFEA0...0xFEFF Not Used by anything.
            // 0xFF00 Joypad
            0xFF0F => Some(self.interupt_flag),
            0xFF10...0xFF3F => Some(0xFF), // Audio device not implemented.
            0xFF40...0xFF45 => self.ppu.read(addr), // PPU state
            0xFF47...0xFF49 => self.gbp.read(addr), // Pallet for GB
            0xFF50 => Some(match self.boot{true => 0xFE, false => 0xFF}),
            0xFF80...0xFFFE => Some(self.hram[addr as usize & 0x007F]),
            0xFFFF => Some(self.interupt_enable),
            _ => None,
        }
    }
    fn write(&mut self, addr: u16, data: u8) -> bool {
        match addr {
            // Main table
            0x0000...0x7FFF => (*self.cartrage).write(addr, data),
            0x8000...0x9FFF => {self.vram[addr as usize & 0x0FFF] = data; true}
            0xA000...0xBFFF => (*self.cartrage).write_ram(addr, data),
            0xC000...0xDFFF => {self.wram[addr as usize & 0x0FFF] = data; true}
            0xE000...0xFDFF => {self.wram[addr as usize & 0x0FFF] = data; true}
            0xFE00...0xFE9F => self.oam.write(addr, data),
            // 0xFEA0...0xFEFF Not Usable.  Tetris write here.
            // 0xFF00 => false, Joypad
            0xFF01...0xFF02 => true, // Not implemented serial
            0xFF0F => {self.interupt_flag = data; true}
            0xFF10...0xFF3F => true, // Audio device not implemented.
            0xFF40...0xFF45 => self.ppu.write(addr, data), // PPU state
            0xFF47...0xFF49 => self.gbp.write(addr, data), // Pallet for GB
            0xFF50 => {self.boot = (0x01 & data) > 0; true},
            0xFF80...0xFFFE => {self.hram[addr as usize & 0x007F] = data; true}
            0xFFFF => {self.interupt_enable = data; true},
            _ => false,
        }
    }
    fn time_passes(&mut self, time: usize) -> Option<Vec<u8>>{
        self.ppu.time_passes(time)
    }
    fn render(&self, row: u8, buffer: &mut [u8]) {
        let x_offset = self.ppu.scx;
        let y_offset = self.ppu.scy + row;
        for i in 0..GAMEBOY_WIDTH as u8 {
            let buff_offset = i as usize * 3;
            self.renderBackground(x_offset.wrapping_add(i), 
                                  y_offset, 
                                  &mut buffer[buff_offset..buff_offset + 3]);
        }
    }

    fn print_background_map(&self) {
        let map_offset = match self.ppu.lcdc_get(3) {
            true => 0x9C00,
            false => 0x9800, 
        };
        println!("Using Map: {:0X}", map_offset);
        print!("   ");
        (0..32).for_each(move |x| print!(" {:2X}", x));
        println!();
        for i in 0..32 {
            print!("{:2X}:",i);
            (0..32).for_each(move |x| 
                             print!(" {:2X}", self.read(map_offset 
                                                       + (i as u16 * 32) 
                                                       + x
                                                       ).unwrap()
                                    ));
            println!();
        }
    }
    fn print_sprite_table(&self) {
        println!("Character Ram");
        for i in 0..384 {
            let addr = (i*16) + 0x8000;
            print!("Tile {:3X} 0x{:04X}:",i, addr);
            (0..16).for_each(move |x| 
                             print!(" {:2X}", self.read(addr
                                                       + x
                                                       ).unwrap()
                                    ));
            println!();
        }
    }
}

impl GbMapper {
    fn renderBackground(&self, x:u8,y:u8, buffer: &mut [u8]){
        //println!("Background x: {}, y: {}", x, y);
        // Bit 6 - Window Tile Map Display Select (0=9800-9BFF, 1=9C00-9FFF)
        let map_offset = match self.ppu.lcdc_get(3) {
            true => 0x9C00,
            false => 0x9800, 
        };

        let x:u16 = x as u16;
        let y:u16 = y as u16;

        let map_location = map_offset + ((y / 8) * 32) + (x / 8);
        let map_data = self.read(map_location).unwrap();

        // This + the next 15 are the sprite
        let sprite_base = match self.ppu.lcdc_get(4) {
            true => (map_data as u16 * 16) + 0x8000,
            // This needs to be a signed offset
            false => ((map_data as i16 * 16) + 0x9000) as u16,
        };
        
        // Move to the correct row
        let x_offset = 7 - (x % 8);
        let sprite_location = sprite_base + ((y % 8) * 2);

        // print!("Sprite {:3X} at {:4X} offset {:X}", map_data, sprite_base, sprite_location);

        let pixel_byte_low = self.read(sprite_location).unwrap();
        let pixel_byte_high = self.read(sprite_location + 1).unwrap();
        // let pixel_color = ( pixel_byte >> (bit_offset * 2)) & 0x03;
        let pixel_color_low = (pixel_byte_low >> x_offset) & 0x01;
        let pixel_color_high = (pixel_byte_high >> x_offset) & 0x01;
        let pixel_color = pixel_color_low + (2 * pixel_color_high);

        // println!(" color {:1X} from {:2X}", pixel_color, pixel_byte);

        self.gbp.apply(gbp::Pallet::BGP, pixel_color, buffer);
    }
}

impl Mem {
    pub fn new_gb(mapper: GbMapper) -> Self {
        Mem { 
            map_holder: Box::new(mapper), 
            screen: Box::new([0; gameboy_screen_buffer_size as usize]),
            ime: false,
        }
    }

    pub fn load_8(&self, addr: u16) -> u8 {
        // Look value up in memory map
        match self.map_holder.read(addr) {
            Some(data) => data,
            None => {
                eprintln!("Memory read failed for address: {:04X}.",
                         addr);
                0xFF
            }
        }
    }

    pub fn time_passes(&mut self, time: usize) -> Option<Vec<u8>>{
        self.map_holder.time_passes(time)
    }

    pub fn set_ime(&mut self, value: bool) {
        self.ime = value;
    }
    
    pub fn render(&mut self, row: usize) -> bool{
        if row < 144 {
            // Actually render
            let start_row = GAMEBOY_WIDTH as usize * 3 * row;
            let end_row = start_row + (GAMEBOY_WIDTH as usize * 3);
            self.map_holder.render(row as u8, &mut self.screen[start_row..end_row]);
            false
        }
        else {
            // Vblank, and send frame at 145
            row == 145
        }
    }

    pub fn screen_swap(&mut self, other: &mut Box<[u8; gameboy_screen_buffer_size as usize]>) {
        use std::mem::swap;

        swap(&mut self.screen, other);
        // Prints that should be done once a frame:
        // self.map_holder.print_background_map();
        // self.map_holder.print_sprite_table();
    }

    pub fn write_8(&mut self, addr: u16, data: u8) {
        // Look value up in memory map
        if !self.map_holder.write(addr, data) {
            eprintln!("Memory write failed for address: {:04X}", addr)
        }
    }

    pub fn write_16(&mut self, addr: u16, data: u16) {
        // Look value up in memory map
        self.write_8(addr, data as u8);
        self.write_8(addr + 1, (data >> 8) as u8);
    }

    pub fn load_16(&self, addr: u16) -> u16 {
        // Look value up in memory map

        let low: u16 = self.load_8(addr) as u16;
        let high: u16 = self.load_8(addr + 1) as u16;

        (high << 8) + low
    }
}
