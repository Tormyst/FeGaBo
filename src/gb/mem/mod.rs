use std::fs::File;
use std::io::Read;

use ::{GAMEBOY_WIDTH, GAMEBOY_HEIGHT};

mod ppu;
mod gbp;
mod timer;
mod cm;
use self::cm::CartrageMapper;

const KB_8: usize = 0x2000;
const KB_8_MASK: usize = 0x1FFF;
const GAMEBOY_SCREEN_BUFFER_SIZE: u32 = GAMEBOY_WIDTH * GAMEBOY_HEIGHT * 3;

pub struct Buttons {
    pub a: bool,
    pub b: bool,
    pub start: bool,
    pub select: bool,
    pub up: bool,
    pub down: bool,
    pub right: bool,
    pub left: bool,
}

impl Buttons {
    pub fn buttons(&self) -> u8 {
        let mut res = 0;
        if self.start { res |= 0x8; }
        if self.select { res |= 0x4; }
        if self.b { res |= 0x2; }
        if self.a { res |= 0x1; }
        (!res & 0xF)
    }

    pub fn dpad(&self) -> u8 {
        let mut res = 0;
        if self.down { res |= 0x8; }
        if self.up { res |= 0x4; }
        if self.left { res |= 0x2; }
        if self.right { res |= 0x1; }
        (!res & 0xF)
    }
}

use std::iter;
macro_rules! get_sprite {
    ($map:expr, $sprite_index:expr, $row:expr) => {
        (0..8).rev()
            .zip(iter::repeat($map.read(0x8000 + $sprite_index * 16 + $row * 2).unwrap()))
            .zip(iter::repeat($map.read(0x8000 + $sprite_index * 16 + $row * 2 + 1).unwrap()))
            .map(|((val, low), high)| {
            let (low, high) = (((low >> val) & 0x01) > 0, ((high >> val) & 0x01) > 0);
            (high as u8) * 2 + (low as u8)
        })
    }
}

use std::iter::Map;
pub trait MemMapper {
    fn read(&self, addr: u16) -> Option<u8>;
    fn write(&mut self, addr: u16, data: u8) -> bool;
    fn time_passes(&mut self, time: usize) -> Option<Vec<u8>>;
    fn update_input(&mut self, buttons: Buttons);
    fn check_interupt(&mut self, ime: bool) -> Option<u16>;
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
}

#[derive(Copy, Clone)]
struct OamEntry{y:u8, x:u8, t:u8, a:u8}

const DEFAULT_OAM_ENTRY: OamEntry = OamEntry{y:0, x:0, t:0, a:0};

enum OamAtribute{
    Priority,
    YFlip,
    XFlip,
    Pallet,
}

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

    fn read_artibute(&self, atribute: OamAtribute) -> bool {
        match atribute {
            OamAtribute::Priority => 0 < self.a & 1 << 7,
            OamAtribute::YFlip => 0 < self.a & 1 << 6,
            OamAtribute::XFlip => 0 < self.a & 1 << 5,
            OamAtribute::Pallet => 0 < self.a & 1 << 4,
        }
    }
}

struct Oam {
    data: [OamEntry;40],
}

impl Oam {
    fn new() -> Self {
        Oam { data: [DEFAULT_OAM_ENTRY;40] }
    }

    fn entry_num(addr: u16) -> (usize, usize) {
        (((addr >> 2) & 0x003F) as usize, (addr & 0x0003) as usize)
    }

    fn read(&self, addr: u16) -> Option<u8> {
        let (e, n) = Oam::entry_num(addr);
        Some(self.data[e].read(n))
    }
    fn write(&mut self, addr: u16, data: u8) -> bool {
        let (e, n) = Oam::entry_num(addr);
        self.data[e].write(n, data)
    }

    fn sprite_line(&self, map: &MemMapper, scanline: u8) -> Vec<Option<u8>> {
        let mut line = vec![None; GAMEBOY_WIDTH as usize];
        let scanline = scanline + 9; // First 8 lines are not used

        let mut sprites: Vec<_> = self.data.iter()
            .filter(|obj| obj.y >= scanline && obj.y < scanline + 8)
            .collect();
        sprites.sort_by(|obj, obj2| obj.x.cmp(&obj2.x));
        sprites.iter().take(10)
            .for_each(|obj| {
                let sprite = get_sprite!(map, obj.t as u16, 7 - (obj.y - scanline) as u16);

                for (offset, color) in (0..8).zip(sprite) {
                    let xpos = obj.x.wrapping_add(offset).wrapping_sub(8) ;
                    if xpos < GAMEBOY_WIDTH as u8 && line[xpos as usize] == None {
                        line[xpos as usize] = Some(color);
                    }
                }
            });
        line
    }
}

pub struct Mem {
    map_holder: Box<MemMapper>,
    screen: Box<[u8; GAMEBOY_SCREEN_BUFFER_SIZE as usize]>,
    ime: bool,
}

pub struct GbMapper {
    cartrage: CartrageMapper,
    boot_rom: BootRom,
    vram: [u8; KB_8],
    wram: [u8; KB_8],
    boot: bool,
    oam: Oam,
    buttons: Buttons,
    joypad: u8,
    timer: timer::Timer,
    hram: [u8; 127],
    interupt_enable: u8,
    interupt_flag: u8,
    ppu: ppu::PPU,
    gbp: gbp::GBP,
}

impl GbMapper {
    pub fn new(cartrage: String) -> Self {
        let mut mapper = GbMapper {
            cartrage: CartrageMapper::new(cartrage),
            boot_rom: BootRom::new(vec![]),
            vram: [0; KB_8],
            wram: [0; KB_8],
            boot: true,
            oam: Oam::new(),
            buttons: Buttons {a:false,b:false,start:false,select:false,up:false,down:false,left:false,right:false},
            joypad: 0,
            timer: timer::Timer::new(),
            hram: [0; 127],
            interupt_enable: 0,
            interupt_flag: 0,
            ppu: ppu::PPU::new(),
            gbp: gbp::GBP::new(),
        };
        mapper.write(0xFF10, 0x80);
        mapper.write(0xFF11, 0xBF);
        mapper.write(0xFF12, 0xF3);
        mapper.write(0xFF14, 0xBF);
        mapper.write(0xFF16, 0x3F);
        mapper.write(0xFF17, 0x00);
        mapper.write(0xFF19, 0xBF);
        mapper.write(0xFF1A, 0x7F);
        mapper.write(0xFF1B, 0xFF);
        mapper.write(0xFF1C, 0x9F);
        mapper.write(0xFF1E, 0xBF);
        mapper.write(0xFF20, 0xFF);
        mapper.write(0xFF21, 0x00);
        mapper.write(0xFF22, 0x00);
        mapper.write(0xFF23, 0xBF);
        mapper.write(0xFF24, 0x77);
        mapper.write(0xFF25, 0xF3);
        mapper.write(0xFF26, 0xF1);
        mapper.write(0xFF40, 0x91);
        mapper.write(0xFF42, 0x00);
        mapper.write(0xFF43, 0x00);
        mapper.write(0xFF45, 0x00);
        mapper.write(0xFF47, 0xFC);
        mapper.write(0xFF48, 0xFF);
        mapper.write(0xFF49, 0xFF);
        mapper
    }

    pub fn new_with_boot_rom(boot_rom: String, cartrage: String) -> Self {
        let mut buffer = Vec::new();
        let boot_size = File::open(boot_rom)
            .unwrap()
            .read_to_end(&mut buffer)
            .unwrap();
        println!("Boot rom loaded: {:X} bytes long", boot_size);

        GbMapper {
            cartrage: CartrageMapper::new(cartrage),
            boot_rom: BootRom::new(buffer),
            vram: [0; KB_8],
            wram: [0; KB_8],
            boot: false,
            buttons: Buttons {a:false,b:false,start:false,select:false,up:false,down:false,left:false,right:false},
            joypad: 0,
            timer: timer::Timer::new(),
            oam: Oam::new(),
            hram: [0; 127],
            interupt_enable: 0,
            interupt_flag: 0,
            ppu: ppu::PPU::new(),
            gbp: gbp::GBP::new(),
        }
    }

    fn dma(&mut self, data: u8) -> bool{
        let start = (data as u16) << 8;
        for offset in 0..0x9F {
            let data = self.read(start + offset).unwrap();
            self.write(0xFE00 + offset, data);
        }
        true
    }

}

impl MemMapper for GbMapper {
    fn read(&self, addr: u16) -> Option<u8> {
        // Main table
        match addr {
            0x0000...0x00FF => if !self.boot { self.boot_rom.read(addr) } else { self.cartrage.read(addr) }
            0x0000...0x7FFF => self.cartrage.read(addr),
            0x8000...0x9FFF => Some(self.vram[addr as usize & KB_8_MASK]),
            0xA000...0xBFFF => self.cartrage.read_ram(addr),
            0xC000...0xDFFF => Some(self.wram[addr as usize & KB_8_MASK]),
            0xE000...0xFDFF => Some(self.wram[addr as usize & KB_8_MASK]),
            0xFE00...0xFE9F => self.oam.read(addr),
            // 0xFEA0...0xFEFF Not Used by anything.
            0xFF00 => Some(self.joypad), // Joypad
            0xFF04...0xFF07 => self.timer.read(addr),
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
            0x0000...0x7FFF => self.cartrage.write(addr, data),
            0x8000...0x9FFF => {self.vram[addr as usize & KB_8_MASK] = data; true}
            0xA000...0xBFFF => self.cartrage.write_ram(addr, data),
            0xC000...0xDFFF => {self.wram[addr as usize & KB_8_MASK] = data; true}
            0xE000...0xFDFF => {self.wram[addr as usize & KB_8_MASK] = data; true}
            0xFE00...0xFE9F => self.oam.write(addr, data),
            // 0xFEA0...0xFEFF Not Usable.  Tetris write here.
            0xFF00 => { // Joypad
                self.joypad = data & 0x30; // Only control bits.
                if self.joypad & 0x10 == 0 {
                    self.joypad |= self.buttons.dpad();
                }
                if self.joypad & 0x20 == 0 {
                    self.joypad |= self.buttons.buttons();
                }
                if self.joypad & 0x0F < 0x0F {
                    self.interupt_flag |= 0x10;
                }
                true
            },
            0xFF04...0xFF07 => self.timer.write(addr, data),
            0xFF01...0xFF02 => true, // Not implemented serial
            0xFF0F => {self.interupt_flag = data; println!("IF set to {:02X}", data); true}
            0xFF10...0xFF3F => true, // Audio device not implemented.
            0xFF40...0xFF45 => self.ppu.write(addr, data), // PPU state
            0xFF46 => self.dma(data),
            0xFF47...0xFF49 => self.gbp.write(addr, data), // Pallet for GB
            0xFF50 => {self.boot = self.boot || (data & 0x01) > 0; true},
            0xFF80...0xFFFE => {self.hram[addr as usize & 0x007F] = data; true}
            0xFFFF => {self.interupt_enable = data; println!("IE set to {:02X}", data); true},
            _ => false,
        }
    }
    fn time_passes(&mut self, time: usize) -> Option<Vec<u8>>{
        self.timer.tick(time);
        self.ppu.time_passes(time)
    }
    fn update_input(&mut self, buttons: Buttons) {
        self.buttons = buttons;
        let data = self.read(0xFF00).unwrap();
        self.write(0xFF00, data);
    }

    fn check_interupt(&mut self, ime: bool) -> Option<u16> {
        self.interupt_flag |= self.ppu.interupt_update();
        self.interupt_flag |= self.timer.check_interupt();
        let interupt_triggers = self.interupt_flag & self.interupt_enable;
        if !ime || interupt_triggers == 0 { None }
        else if interupt_triggers & 0x01 > 0 { self.interupt_flag = self.interupt_flag & !0x01; Some(0x40) } // v blank
        else if interupt_triggers & 0x02 > 0 { self.interupt_flag = self.interupt_flag & !0x02; Some(0x48) } // Stat
        else if interupt_triggers & 0x04 > 0 { self.interupt_flag = self.interupt_flag & !0x04; Some(0x50) } // Timer
        else if interupt_triggers & 0x10 > 0 { self.interupt_flag = self.interupt_flag & !0x10; Some(0x60) } // Timer
        else { None }
    }

    fn render(&self, row: u8, buffer: &mut [u8]) {
        let x_offset = self.ppu.scx;
        let y_offset = self.ppu.scy.wrapping_add(row);
        let sprites = self.oam.sprite_line(self, row);
        let background = self.background_line(row);
        for i in 0..GAMEBOY_WIDTH as usize {
            let buff_offset = i * 3;
            if let Some(data) = sprites[i] {
                self.gbp.apply(gbp::Pallet::OBP1,
                               data,
                               &mut buffer[buff_offset..buff_offset + 3]);
            }
            else {
                self.gbp.apply(gbp::Pallet::BGP,
                               background[i],
                               &mut buffer[buff_offset..buff_offset + 3]);
            }
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
    fn background_line(&self, scanline: u8) -> Vec<u8> {
        let x_offset = self.ppu.scx;
        let y_offset = self.ppu.scy.wrapping_add(scanline) as u16;
        let map_x_offset = (x_offset & 0xF8) as u16; // offset rounded by 8
        let map_offset = match self.ppu.lcdc_get(3) {
            true => 0x9C00,
            false => 0x9800,
        };

        let map_row = map_offset + ((y_offset / 8) * 32);
        let sprite_offset = y_offset % 8;

        (0..32)
            .map(|sprite_on_line| self.read(map_row + ((sprite_on_line + map_x_offset) % 32)).unwrap())
            .map(|map_data|
                 match self.ppu.lcdc_get(4) {
                     true => map_data as u16,
                     // This needs to be a signed offset
                     false => (255 + (map_data as i16)) as u16,
                 })
            .flat_map(|sprite_index| get_sprite!(self, sprite_index, sprite_offset))
            .skip(x_offset as usize % 8)
            .take(GAMEBOY_WIDTH as usize)
            .collect()
    }
}

impl Mem {
    pub fn new_gb(mapper: GbMapper) -> Self {
        Mem {
            map_holder: Box::new(mapper),
            screen: Box::new([0; GAMEBOY_SCREEN_BUFFER_SIZE as usize]),
            ime: false,
        }
    }

    pub fn load_8(&self, addr: u16) -> u8 {
        // Look value up in memory map
        match self.map_holder.read(addr) {
            Some(data) => data,
            None => {
                println!("Memory read failed for address: {:04X}.",
                         addr);
                0xFF
            }
        }
    }

    pub fn time_passes(&mut self, time: usize) -> Option<Vec<u8>>{
        self.map_holder.time_passes(time)
    }

    pub fn set_ime(&mut self, value: bool) {
        // println!("IME set to {}", value);
        self.ime = value;
    }

    pub fn check_interupt(&mut self) -> Option<u16> {
        self.map_holder.check_interupt(self.ime)
    }

    pub fn update_input(&mut self, buttons: Buttons) {
        self.map_holder.update_input(buttons)
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
            row == 144
        }
    }

    pub fn screen_swap(&mut self, other: &mut Box<[u8; GAMEBOY_SCREEN_BUFFER_SIZE as usize]>) {
        use std::mem::swap;

        swap(&mut self.screen, other);
        // Prints that should be done once a frame:
        // self.map_holder.print_background_map();
        // self.map_holder.print_sprite_table();
    }

    pub fn write_8(&mut self, addr: u16, data: u8) {
        // Look value up in memory map
        // println!("Memory write to: {:04X} of data {:02X}", addr, data);
        if !self.map_holder.write(addr, data) {
            println!("Memory write failed for address: {:04X}", addr)
        }
    }

    pub fn write_16(&mut self, addr: u16, data: u16) {
        // Look value up in memory map
        self.write_8(addr, data as u8);
        self.write_8(addr.wrapping_add(1), (data >> 8) as u8);
    }

    pub fn load_16(&self, addr: u16) -> u16 {
        // Look value up in memory map

        let low: u16 = self.load_8(addr) as u16;
        let high: u16 = self.load_8(addr + 1) as u16;

        (high << 8) + low
    }
}
