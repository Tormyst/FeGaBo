use std::sync::mpsc;
use std::thread;
use std::sync::{Mutex, Arc};
use ::{GAMEBOY_WIDTH, GAMEBOY_HEIGHT};
use std::path::Path;

const GAMEBOY_SCREEN_BUFFER_SIZE: u32 = GAMEBOY_WIDTH * GAMEBOY_HEIGHT * 3;
const DMG_ROM: &'static str = "assets/rom/dmg_rom.gb";
const ROM_FILE: &'static str = "assets/tetris.gb";

mod cpu;
mod mem;
mod decode;

enum GbKind {
    GB,
    SGB,
    GBC,
}

pub struct GbConnect {
    pub to_gb: mpsc::Sender<usize>,
    pub from_gb: mpsc::Receiver<usize>,
    pub canvas: Arc<Mutex<Box<[u8; GAMEBOY_SCREEN_BUFFER_SIZE as usize]>>>,
}

struct Gb {
    cpu: cpu::Cpu,
    mem: mem::Mem,
    to_main: mpsc::Sender<usize>,
    from_main: mpsc::Receiver<usize>,
    front_buffer: Arc<Mutex<Box<[u8; GAMEBOY_SCREEN_BUFFER_SIZE as usize]>>>,
}

pub fn connect() -> GbConnect {
    let (to_gb, from_main) = mpsc::channel();
    let (to_main, from_gb) = mpsc::channel();
    let canvas = Arc::new(Mutex::new(
            Box::new([0; GAMEBOY_SCREEN_BUFFER_SIZE as usize])));

    let front_buffer = Arc::clone(&canvas);
    thread::Builder::new().name("GB".to_string()).spawn(move || { 
        Gb::new(GbKind::GB, to_main, from_main, front_buffer).unwrap().cycle(); 
    }).unwrap();

    GbConnect { to_gb, from_gb, canvas }
}

impl Gb {
    fn new(kind: GbKind,
           to_main: mpsc::Sender<usize>,
           from_main: mpsc::Receiver<usize>,
           front_buffer: Arc<Mutex<Box<[u8; GAMEBOY_SCREEN_BUFFER_SIZE as usize]>>>)
           -> Option<Gb> {
        match kind {
            GbKind::GB => {
                if Path::new(DMG_ROM).is_file() {
                    Some(Gb {
                            cpu: cpu::Cpu::new(),
                            mem: mem::Mem::new_gb(
                                mem::GbMapper::new_with_boot_rom(DMG_ROM.to_string(),
                                ROM_FILE.to_string())),
                            to_main,
                            from_main,
                            front_buffer,
                        })
                }
                else {
                    Some(Gb {
                            cpu: cpu::Cpu::new_after_boot(),
                            mem: mem::Mem::new_gb(mem::GbMapper::new(ROM_FILE.to_string())),
                            to_main,
                            from_main,
                            front_buffer,
                        })
                }
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
                        self.to_main.send(0).unwrap();
                        // This should be updated button info.
                        let _val = self.from_main.recv().unwrap(); 
                    }
                }
            }
        }
    }
}
