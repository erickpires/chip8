mod cpu;
mod display;

extern crate sdl2;

use std::{fs::File, io::Read};
use std::time::Duration;

use sdl2::{pixels::Color, keyboard::Keycode, EventPump, render::Canvas};

fn main() {
    let mut is_running = true;

    let (mut canvas, mut event_pump) = init_sdl();

    let mut rom = Vec::new();
    let mut rom_file = File::open("rom.ch8").expect("Unable to open ROM file.");
    rom_file.read_to_end(&mut rom).expect("Unable to read ROM file.");

    let mut cpu = cpu::Cpu::new();
    cpu.load_rom(&rom, false);


    loop {
        if !is_running {
            break;
        }

        let events = handle_event_loop(&mut event_pump);

        if events.contains(&Event::Quit) {
            is_running = false;
        }

        cpu.tick();

        update_display(&mut canvas);

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60))
    }
}

fn init_sdl() -> (Canvas<sdl2::video::Window>, EventPump){
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

    let event_pump = sdl_context.event_pump().unwrap();

    return (canvas, event_pump);
}

#[derive(PartialEq)]
enum Event {
    Quit,
    KeyPressed(u8)
}

fn handle_event_loop(event_pump: &mut EventPump) -> Vec<Event> {
    let mut result = Vec::new();

    for event in event_pump.poll_iter() {
        match event {
            sdl2::event::Event::Quit {..} => {
                result.push(Event::Quit)
            },
            sdl2::event::Event::KeyUp { keycode: Some(code), ..} => {
                print!("Key pressed: {}", code);

                if code == Keycode::Escape {
                    result.push(Event::Quit)
                }

                // TODO: Parse the key
                result.push(Event::KeyPressed(0));
            },
            _ => { }
        }
    }

    return result
}

fn update_display(canvas: &mut Canvas<sdl2::video::Window>) {
    let rand_color = rand::random();
    canvas.set_draw_color(Color::RGB(rand_color, 0, rand_color));
    canvas.clear();
    canvas.present();
}