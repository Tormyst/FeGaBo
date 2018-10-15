use std::sync::mpsc;
use std::thread;
use std::sync::{Mutex, Arc};
use ::{GAMEBOY_WIDTH, GAMEBOY_HEIGHT};

const gameboy_screen_buffer_size: u32 = GAMEBOY_WIDTH * GAMEBOY_HEIGHT * 3;
const gameboy_background_size: u32 = 256 * 256 * 3 * 2;
const gameboy_sprite_table_size: u32 = 256 * 96 * 3;

mod cpu;
mod mem;
mod decode;
mod disassemble;

enum GB_KIND {
    GB,
    SGB,
    GBC,
}

pub struct GbConnect {
    pub to_gb: mpsc::Sender<usize>,
    pub from_gb: mpsc::Receiver<usize>,
    pub canvas: Arc<Mutex<Box<[u8; gameboy_screen_buffer_size as usize]>>>,
    pub bgcanvas: Arc<Mutex<Box<[u8; gameboy_background_size as usize]>>>,
    pub stcanvas: Arc<Mutex<Box<[u8; gameboy_sprite_table_size as usize]>>>,
}

struct Gb {
    cpu: cpu::Cpu,
    mem: mem::Mem,
    to_main: mpsc::Sender<usize>,
    from_main: mpsc::Receiver<usize>,
    front_canvas: Arc<Mutex<Box<[u8; gameboy_screen_buffer_size as usize]>>>,
    front_bgcanvas: Arc<Mutex<Box<[u8; gameboy_background_size as usize]>>>,
    front_stcanvas: Arc<Mutex<Box<[u8; gameboy_sprite_table_size as usize]>>>,
}

pub fn connect() -> GbConnect {
    let (to_gb, from_main) = mpsc::channel();
    let (to_main, from_gb) = mpsc::channel();
    let front_canvas = Arc::new(Mutex::new(
            Box::new([0; gameboy_screen_buffer_size as usize])));
    let front_bgcanvas = Arc::new(Mutex::new(
            Box::new([0; gameboy_background_size as usize])));
    let front_stcanvas = Arc::new(Mutex::new(
            Box::new([0; gameboy_sprite_table_size  as usize])));

    let remote_canvas = Arc::clone(&front_canvas);
    let remote_bgcanvas = Arc::clone(&front_bgcanvas);
    let remote_stcanvas = Arc::clone(&front_stcanvas);
    thread::Builder::new().name("GB".to_string()).spawn(move || { 
        Gb::new(GB_KIND::GB, 
                to_main, 
                from_main, 
                remote_canvas, 
                remote_bgcanvas, 
                remote_stcanvas).unwrap().cycle(); 
    });

    GbConnect { 
        to_gb, 
        from_gb, 
        canvas: front_canvas, 
        bgcanvas: front_bgcanvas, 
        stcanvas:front_stcanvas
    }
}

impl Gb {
    fn new(kind: GB_KIND,
           to_main: mpsc::Sender<usize>,
           from_main: mpsc::Receiver<usize>,
           front_canvas: Arc<Mutex<Box<[u8; gameboy_screen_buffer_size as usize]>>>,
           front_bgcanvas: Arc<Mutex<Box<[u8; gameboy_background_size as usize]>>>,
           front_stcanvas: Arc<Mutex<Box<[u8; gameboy_sprite_table_size as usize]>>>)
           -> Option<Gb> {
        match kind {
            GB_KIND::GB => {
                Some(Gb {
                         cpu: cpu::Cpu::new(),
                         mem: mem::Mem::new_gb(
                             mem::GbMapper::new_with_boot_rom(
                                 "assets/rom/dmg_rom.gb".to_string(),
                                 "assets/tetris.gb".to_string())),
                         to_main,
                         from_main,
                         front_canvas,
                         front_bgcanvas,
                         front_stcanvas,
                     })
            }
            _ => None,
        }
    }

    fn cycle(mut self) {
        println!("Everything is set up!!!!");
        loop {
            let time = self.cpu.cycle(&mut self.mem);
            if let Some(rows) = self.mem.time_passes(time) {
                for r in rows {
                    if self.mem.render(r as usize) {
                        // Send frame by swapping buffers and telling main to do something.
                        self.mem.screen_swap(&mut self.front_canvas.lock().unwrap());
                        self.mem.background_swap(&mut self.front_bgcanvas.lock().unwrap());
                        self.mem.sprite_swap(&mut self.front_stcanvas.lock().unwrap());
                        self.to_main.send(0);
                        // This should be updated button info.
                        let val = self.from_main.recv().unwrap(); 
                    }
                }
            }
        }
    }
}
