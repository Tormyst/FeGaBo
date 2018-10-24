use std::sync::mpsc;
use std::thread;
use std::sync::{Mutex, Arc};
use ::{GAMEBOY_WIDTH, GAMEBOY_HEIGHT};

const GAMEBOY_SCREEN_BUFFER_SIZE: u32 = GAMEBOY_WIDTH * GAMEBOY_HEIGHT * 3;

mod cpu;
pub mod mem;
mod decode;

enum GbKind {
    GB,
    // SGB,
    // GBC,
}

pub enum Input {
    Buttons(mem::Buttons),
}

pub enum Output {
    Frame,
}

pub struct GbConnect {
    pub to_gb: mpsc::Sender<Input>,
    pub from_gb: mpsc::Receiver<Output>,
    pub canvas: Arc<Mutex<Box<[u8; GAMEBOY_SCREEN_BUFFER_SIZE as usize]>>>,
}

struct Gb {
    cpu: cpu::Cpu,
    mem: mem::Mem,
    to_main: mpsc::Sender<Output>,
    from_main: mpsc::Receiver<Input>,
    front_buffer: Arc<Mutex<Box<[u8; GAMEBOY_SCREEN_BUFFER_SIZE as usize]>>>,
}

pub fn connect(roms: (String, Option<String>)) -> GbConnect {
    let (to_gb, from_main) = mpsc::channel();
    let (to_main, from_gb) = mpsc::channel();
    let canvas = Arc::new(Mutex::new(
            Box::new([0; GAMEBOY_SCREEN_BUFFER_SIZE as usize])));

    let front_buffer = Arc::clone(&canvas);
    thread::Builder::new().name("GB".to_string()).spawn(move || {
        Gb::new(GbKind::GB, roms, to_main, from_main, front_buffer).unwrap().cycle();
    }).unwrap();

    GbConnect { to_gb, from_gb, canvas }
}

impl Gb {
    fn new(kind: GbKind,
           roms: (String, Option<String>),
           to_main: mpsc::Sender<Output>,
           from_main: mpsc::Receiver<Input>,
           front_buffer: Arc<Mutex<Box<[u8; GAMEBOY_SCREEN_BUFFER_SIZE as usize]>>>)
           -> Option<Gb> {
        match kind {
            GbKind::GB => {
                match &roms.1 { 
                    Some(bootrom) => Some(Gb {
                            cpu: cpu::Cpu::new(),
                            mem: mem::Mem::new_gb(
                                mem::GbMapper::new_with_boot_rom(bootrom.to_string(), 
                                                                 roms.0)),
                            to_main,
                            from_main,
                            front_buffer,
                        }),
                    None => Some(Gb {
                            cpu: cpu::Cpu::new_after_boot(),
                            mem: mem::Mem::new_gb(mem::GbMapper::new(roms.0)),
                            to_main,
                            from_main,
                            front_buffer,
                        })
                }
            }
            // _ => None,
        }
    }

    fn cycle(mut self) {
        println!("Everything is set up!!!!");
        'cycle_loop: loop {
            let time = self.cpu.cycle(&mut self.mem);
            self.time_passes(time); 
            if let Some(interupt) = self.mem.check_interupt() {
                self.cpu.handle_interupt(interupt, &mut self.mem);
                self.time_passes(16); 
            }
        }
    }

    fn time_passes(&mut self, time: usize) {
        if let Some(rows) = self.mem.time_passes(time) {
            for r in rows {
                if self.mem.render(r as usize) {
                    // Send frame by swapping buffers and telling main to do something.
                    self.mem.screen_swap(&mut self.front_buffer.lock().unwrap());
                    self.to_main.send(Output::Frame).unwrap();
                    // This should be updated button info.
                    match self.from_main.recv() {
                        Ok(super::Input::Buttons(buttons)) => self.mem.update_input(buttons),
                        // Err(_) => break 'cycle_loop,
                        _ => panic!("Unknown message from main"),
                    }
                }
            }
        }
    }
}
