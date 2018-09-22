use std::sync::mpsc;
use std::thread;

mod cpu;
mod mem;

enum GB_KIND {
    GB,
    SGB,
    GBC,
}

pub struct GbConnect {
    pub to_gb: mpsc::Sender<usize>,
    pub from_gb: mpsc::Receiver<usize>,
}

struct Gb {
    cpu: cpu::Cpu,
    mem: mem::Mem<mem::GbMapper>,
    to_main: mpsc::Sender<usize>,
    from_main: mpsc::Receiver<usize>,
}

pub fn connect() -> GbConnect {
    let (to_gb, from_main) = mpsc::channel();
    let (to_main, from_gb) = mpsc::channel();

    thread::spawn(move || { Gb::new(GB_KIND::GB, to_main, from_main).unwrap().cycle(); });

    GbConnect { to_gb, from_gb }
}

impl Gb {
    fn new(kind: GB_KIND,
           to_main: mpsc::Sender<usize>,
           from_main: mpsc::Receiver<usize>)
           -> Option<Gb> {
        match kind {
            GB_KIND::GB => {
                Some(Gb {
                         cpu: cpu::Cpu::new(),
                         mem: mem::Mem::new(mem::GbMapper::new_with_boot_rom("assets/rom/dmg_rom.gb"
                                                                                 .to_string())),
                         to_main,
                         from_main,
                     })
            }
            _ => None,
        }
    }

    fn cycle(mut self) {
        println!("Everything is set up!!!!");
        loop {
            self.cpu.cycle(&mut self.mem);
        }
    }
}
