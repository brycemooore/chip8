use chip8::chip::Chip8;
use sdl2::event::Event;
const SCALE: u32 = 15;
const WINDOW_HEIGHT: u32 = chip8::chip::DISPLAY_MAX_Y as u32 * SCALE;
const WINDOW_WIDTH: u32 =chip8::chip::DISPLAY_MAX_X as u32 * SCALE;

fn main() {

    let mut _chip = Chip8::new();
    
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
                Event::Quit{..} => {
                    break 'gameloop;
                },
                _ => ()
            }
        }
    }
}
