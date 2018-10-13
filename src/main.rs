extern crate sdl2;

use std::time::{SystemTime, Duration};

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::WindowContext;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Texture, TextureCreator};
use sdl2::rect::Rect;

const GAMEBOY_WIDTH: u32 = 160;
const GAMEBOY_HEIGHT: u32 = 144;
const RESOLUTION_MULTEPLYER: u32 = 4;

const SPRITES_PER_ROW: u32 = 32;
const SPRITE_ROWS: u32 = 12;

mod gb;
// use gb::{gameboy_screen_buffer_size, gameboy_background_size, gameboy_sprite_table_size}

enum TextureType {
    Screen,
    BGMap,
    SpriteTable,
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
                    (RESOLUTION_MULTEPLYER * GAMEBOY_WIDTH) + 256,
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

    fn create_Screen_internal<'a>(texture_creator: &'a TextureCreator<WindowContext>)
                                  -> Texture<'a> {
        texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, GAMEBOY_WIDTH, GAMEBOY_HEIGHT)
            .unwrap()
    }

    fn create_SpriteTable_internal<'a>(texture_creator: &'a TextureCreator<WindowContext>)
                                  -> Texture<'a> {
        texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, 256, 96)
            .unwrap()
    }

    fn create_BGMap_internal<'a>(texture_creator: &'a TextureCreator<WindowContext>)
                                  -> Texture<'a> {
        texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, 32 * 8, 32 * 8)
            .unwrap()
    }

    fn event_loop(mut self, gbconnect: gb::GbConnect) {
        let mut live_textures = Vec::new();
        live_textures
            .push((TextureType::Screen, Window::create_Screen_internal(&self.texture_creator)));
        live_textures
            .push((TextureType::BGMap, Window::create_BGMap_internal(&self.texture_creator)));
        live_textures
            .push((TextureType::SpriteTable, Window::create_SpriteTable_internal(&self.texture_creator)));
        //self.create_screen();
        let mut event_pump = self.sdl_context.event_pump().unwrap();
        let mut frame: u32 = 0;
        let mut fps = 0;
        let mut start_time = SystemTime::now();
        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                    _ => {}
                }
            }
            use std::sync::mpsc::TryRecvError;

            match gbconnect.from_gb.try_recv() {
                Ok(event) => {
                    for texture in &mut live_textures {
                        match texture.0 {
                            TextureType::Screen => {
                                texture
                                    .1
                                    .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                                        let frame = gbconnect.canvas.lock().unwrap();
                                        buffer.copy_from_slice(&**frame);
                                    })
                                    .unwrap();
                                self.canvas.copy(&texture.1, None, 
                                                 Some(Rect::new(0,0,
                                                                RESOLUTION_MULTEPLYER * GAMEBOY_WIDTH, 
                                                                RESOLUTION_MULTEPLYER * GAMEBOY_HEIGHT
                                                                ))).unwrap();
                            },
                            TextureType::BGMap => {
                                texture
                                    .1
                                    .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                                        let frame = gbconnect.bgcanvas.lock().unwrap();
                                        eprintln!("BGMap: {}", frame.len());
                                        eprintln!("buffer: {}", buffer.len());
                                        buffer.copy_from_slice(&**frame);
                                    })
                                    .unwrap();
                                self.canvas.copy(&texture.1, None, 
                                                 Some(Rect::new((RESOLUTION_MULTEPLYER * GAMEBOY_WIDTH) as i32,
                                                                0,
                                                                256,
                                                                256))).unwrap();
                            },
                            TextureType::SpriteTable => {
                                texture
                                    .1
                                    .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                                        let frame = gbconnect.stcanvas.lock().unwrap();
                                        eprintln!("sprite table: {}", frame.len());
                                        eprintln!("buffer: {}", buffer.len());
                                        buffer.copy_from_slice(&**frame);
                                    })
                                    .unwrap();
                                self.canvas.copy(&texture.1, None, 
                                                 Some(Rect::new((RESOLUTION_MULTEPLYER * GAMEBOY_WIDTH) as i32,
                                                                256,
                                                                SPRITES_PER_ROW * 8, 
                                                                SPRITE_ROWS * 8))).unwrap();
                            }
                            _ => {}
                        }
                    }
                    self.canvas.present();
                    fps += 1;
                    
                    gbconnect.to_gb.send(frame as usize).unwrap();}
                Err(err) => {
                    match err {
                        TryRecvError::Disconnected => {
                            std::thread::sleep(std::time::Duration::from_secs(10));
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


pub fn main() {
    let window = Window::new();

    let gbconnect = gb::connect();

    window.event_loop(gbconnect);
}
