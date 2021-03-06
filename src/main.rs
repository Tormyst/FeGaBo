extern crate sdl2;
#[macro_use]
extern crate clap;

use std::time::{SystemTime, Duration};

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::WindowContext;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Texture, TextureCreator};

const GAMEBOY_WIDTH: u32 = 160;
const GAMEBOY_HEIGHT: u32 = 144;
const RESOLUTION_MULTEPLYER: u32 = 4;

mod gb;
use gb::Output;
use gb::Input;

enum TextureType {
    Screen,
}

struct Window {
    sdl_context: sdl2::Sdl,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    texture_creator: TextureCreator<WindowContext>,
}

impl Window {
    fn new() -> Window {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("FeGaBo",
                    RESOLUTION_MULTEPLYER * GAMEBOY_WIDTH,
                    RESOLUTION_MULTEPLYER * GAMEBOY_HEIGHT)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window
            .into_canvas()
            .target_texture()
            .present_vsync()
            .build()
            .unwrap();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        let texture_creator: TextureCreator<_> = canvas.texture_creator();

        Window {
            sdl_context,
            canvas,
            texture_creator: texture_creator,
        }
    }

    fn create_screen_internal<'a>(texture_creator: &'a TextureCreator<WindowContext>)
        -> Texture<'a> {
            texture_creator
                .create_texture_streaming(PixelFormatEnum::RGB24, GAMEBOY_WIDTH, GAMEBOY_HEIGHT)
                .unwrap()
        }

    fn event_loop(mut self, gbconnect: gb::GbConnect) {
        let mut live_textures = Vec::new();
        live_textures
            .push((TextureType::Screen, Window::create_screen_internal(&self.texture_creator)));
        //self.create_screen();
        let mut event_pump = self.sdl_context.event_pump().unwrap();
        let mut fps = 0;
        let mut start_time = SystemTime::now();
        let mut a = false;
        let mut b = false;
        let mut start = false;
        let mut select = false;
        let mut up = false;
        let mut down = false;
        let mut left = false;
        let mut right = false;
        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } |
                        Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                        Event::KeyDown { keycode: Some(Keycode::C), .. } => a = true,
                        Event::KeyDown { keycode: Some(Keycode::X), .. } => b = true,
                        Event::KeyDown { keycode: Some(Keycode::B), .. } => start = true,
                        Event::KeyDown { keycode: Some(Keycode::V), .. } => select = true,
                        Event::KeyDown { keycode: Some(Keycode::Up), .. } => up = true,
                        Event::KeyDown { keycode: Some(Keycode::Down), .. } => down = true,
                        Event::KeyDown { keycode: Some(Keycode::Left), .. } => left = true,
                        Event::KeyDown { keycode: Some(Keycode::Right), .. } => right = true,

                        Event::KeyUp { keycode: Some(Keycode::C), .. } => a = false,
                        Event::KeyUp { keycode: Some(Keycode::X), .. } => b = false,
                        Event::KeyUp { keycode: Some(Keycode::B), .. } => start = false,
                        Event::KeyUp { keycode: Some(Keycode::V), .. } => select = false,
                        Event::KeyUp { keycode: Some(Keycode::Up), .. } => up = false,
                        Event::KeyUp { keycode: Some(Keycode::Down), .. } => down = false,
                        Event::KeyUp { keycode: Some(Keycode::Left), .. } => left = false,
                        Event::KeyUp { keycode: Some(Keycode::Right), .. } => right = false,
                        _ => {}
                }
            }
            use std::sync::mpsc::TryRecvError;

            match gbconnect.from_gb.try_recv() {
                Ok(Output::Frame) => {
                    for texture in &mut live_textures {
                        match texture.0 {
                            TextureType::Screen => {
                                texture
                                    .1
                                    .with_lock(None, |buffer: &mut [u8], _pitch: usize| {
                                        let frame = gbconnect.canvas.lock().unwrap();
                                        buffer.copy_from_slice(&**frame);
                                    })
                                .unwrap();
                            }
                        }
                        self.canvas.copy(&texture.1, None, None).unwrap();
                    }
                    self.canvas.present();
                    fps += 1;

                    gbconnect.to_gb.send(Input::Buttons(gb::mem::Buttons {
                        a,
                        b,
                        select,
                        start,
                        up,
                        down,
                        left,
                        right,
                    })).unwrap();}
                    Err(err) => {
                        match err {
                            TryRecvError::Disconnected => {
                                panic!("CPU halted unexpectedly.");
                            }
                            _ => {}
                        };
                    }
            }

            if SystemTime::now().duration_since(start_time).unwrap() > Duration::from_secs(1) {
                println!("FPS: {}", fps);
                fps = 0;
                start_time = SystemTime::now();
            }
        }
    }
}

fn read_arguments() -> (String, std::option::Option<String>){
    let app = clap_app!(FeGaBo =>
        (version: "0.1")
        (author: "Raphael BN")
        (about: "A gameboy emulator writen in rust")
        (@arg ROM: +required "Sets the file to use as a gameboy cartrage")
        (@arg BOOTROM: "Sets the file to use as the bootrom")
        // (@arg debug: -d ... "Sets the level of debugging information")
        (@arg disassemble: -d "Disassemble the given file")
    ).get_matches();

    if app.is_present("disassemble") {
        gb::disasemble(app.value_of("ROM").unwrap());
        std::process::exit(0);
    }

    (String::from(app.value_of("ROM").unwrap()), match app.value_of("BOOTROM"){
        Some(bootrom) => Some(String::from(bootrom)),
        _ => None,
    })
}

pub fn main() {
    let roms = read_arguments();

    let window = Window::new();

    let gbconnect = gb::connect(roms);

    window.event_loop(gbconnect);
}
