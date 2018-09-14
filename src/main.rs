extern crate sdl2;

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

fn create_screen<'a>(texture_creator: &'a TextureCreator<WindowContext>) -> Texture<'a> {
    texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, GAMEBOY_WIDTH, GAMEBOY_HEIGHT)
        .unwrap()
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("myst gameboy",
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
    let mut square_texture = create_screen(&texture_creator);

    let mut event_pump = sdl_context.event_pump().unwrap();
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

        // Sample update tester
        if frame >= 255 {
            frame = 0;
        }

        enum MODE_UPDATE {
            ADD,
            SUB,
        }

        square_texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                let mut frame = frame as u8;
                let mut mode = MODE_UPDATE::ADD;
                for y in 0..GAMEBOY_HEIGHT as usize {
                    for x in 0..GAMEBOY_WIDTH as usize {
                        let offset = y * pitch + x * 3;
                        buffer[offset] = frame;
                        buffer[offset + 1] = frame;
                        buffer[offset + 2] = frame;
                        frame = match mode {
                            MODE_UPDATE::ADD => {
                                if frame == 255 {
                                    mode = MODE_UPDATE::SUB;
                                    frame
                                } else {
                                    frame + 1
                                }
                            }
                            MODE_UPDATE::SUB => {
                                if frame == 0 {
                                    mode = MODE_UPDATE::ADD;
                                    frame
                                } else {
                                    frame - 1
                                }
                            }
                        }

                    }
                }
            })
            .unwrap();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.copy(&square_texture, None, None).unwrap();
        canvas.present();
        frame += 1;
        fps += 1;
        if SystemTime::now().duration_since(start_time).unwrap() > Duration::from_secs(1) {
            println!("FPS: {}", fps);
            fps = 0;
            start_time = SystemTime::now();
        }
    }
}
