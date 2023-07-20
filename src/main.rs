extern crate sdl2;

use std::time::Duration;

use sdl2::{pixels::Color, keyboard::Keycode};

fn main() {
    let mut is_running = true;

    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();

    let window = video.window("Chip 8", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(255, 0, 255));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut i = 0;
    loop {
        if !is_running {
            break;
        }

        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => {
                    is_running = false;
                },
                sdl2::event::Event::KeyUp { keycode: Some(code), ..} => {
                    print!("Key pressed: {}", code);

                    if code == Keycode::Escape {
                        is_running = false;
                    }
                },
                _ => { }
            }
        }

        canvas.set_draw_color(Color::RGB(i, 0, i));
        canvas.clear();
        canvas.present();

        i = if i < 255 { i + 1 } else { 0 };

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60))
    }
}
