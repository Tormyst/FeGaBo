use std::sync::mpsc;
use std::thread;
use std::sync::{Mutex, Arc};
use ::{GAMEBOY_WIDTH, GAMEBOY_HEIGHT};

const gameboy_screen_buffer_size: u32 = GAMEBOY_WIDTH * GAMEBOY_HEIGHT * 3;

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
}

struct Gb {
    cpu: cpu::Cpu,
    mem: mem::Mem,
    to_main: mpsc::Sender<usize>,
    from_main: mpsc::Receiver<usize>,
    front_buffer: Arc<Mutex<Box<[u8; gameboy_screen_buffer_size as usize]>>>,
}

pub fn connect() -> GbConnect {
    let (to_gb, from_main) = mpsc::channel();
    let (to_main, from_gb) = mpsc::channel();
    let canvas = Arc::new(Mutex::new(
            Box::new([0; gameboy_screen_buffer_size as usize])));

    let front_buffer = Arc::clone(&canvas);
    thread::Builder::new().name("GB".to_string()).spawn(move || { 
        Gb::new(GB_KIND::GB, to_main, from_main, front_buffer).unwrap().cycle(); 
    });

    GbConnect { to_gb, from_gb, canvas }
}

impl Gb {
    fn new(kind: GB_KIND,
           to_main: mpsc::Sender<usize>,
           from_main: mpsc::Receiver<usize>,
           front_buffer: Arc<Mutex<Box<[u8; gameboy_screen_buffer_size as usize]>>>)
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
                         front_buffer,
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
                        self.mem.screen_swap(&mut self.front_buffer.lock().unwrap());
                        self.to_main.send(0);
                        // This should be updated button info.
                        let val = self.from_main.recv().unwrap(); 
                    }
                }
            }
        }
    }
}
