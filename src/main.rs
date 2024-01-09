use chip8::chip::Chip8;
use chip8::chip::{DISPLAY_MAX_X, DISPLAY_MAX_Y};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::{event::Event, rect::Rect};
use std::fs::File;
use std::io::Read;

const SCALE: u32 = 15;
const WINDOW_HEIGHT: u32 = DISPLAY_MAX_Y as u32 * SCALE;
const WINDOW_WIDTH: u32 = DISPLAY_MAX_X as u32 * SCALE;
const TICKS_PER_FRAME: u8 = 10;

fn main() {
    let file_path = "./roms/PONG2";
    let file_buffer = Box::new(get_file_buffer(file_path));

    let mut chip = Chip8::new();
    chip.load_rom(file_buffer.into_boxed_slice());

    //setup sdl2
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Chip-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    //get canvas
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();

    //game loop and event check
    let mut event_pump = sdl_context.event_pump().unwrap();
    'gameloop: loop {
        for evt in event_pump.poll_iter() {
            match evt {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'gameloop;
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = key2btn(key) {
                        chip.key_press(k);
                    } else {
                        chip.restart();
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = key2btn(key) {
                        chip.key_release(k);
                    }
                }
                _ => (),
            }
        }

        for _ in 0..=TICKS_PER_FRAME {
            chip.tick();
        }
        chip.tick_timers();
        draw_screen(&chip, &mut canvas);
    }
}

fn draw_screen(chip: &Chip8, canvas: &mut Canvas<Window>) {
    //draw color is black
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let display_buffer = chip.get_display();

    //set white
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for (i, pixel) in display_buffer.iter().enumerate() {
        if *pixel {
            //index % mod max x gives you the remainder disregarding the row, which would be the x
            let x = (i % DISPLAY_MAX_X) as u32;
            // index / max x gives you which row
            let y = (i / DISPLAY_MAX_X) as u32;

            //draw rectangle with scale
            let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            canvas.fill_rect(rect).unwrap();
        }
    }
    canvas.present();
}

fn key2btn(key: Keycode) -> Option<u8> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}

fn get_file_buffer(path: &str) -> Vec<u8> {
    let mut file = File::open(path).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    buffer
}
