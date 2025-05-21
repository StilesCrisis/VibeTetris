extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem.window("SDL2 Window", 800, 600)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build()
        .map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGB(0, 0, 0)); // Black color
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        // Redraw the black screen in case of window events (e.g., minimize, maximize)
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        // Sleep a little to not hog the CPU
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
