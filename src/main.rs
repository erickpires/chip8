mod cpu;
mod display;

extern crate sdl2;

use std::{fs::File, io::Read};
use std::time::Duration;

use display::Display;
use sdl2::audio::{AudioSpecDesired, AudioCallback, AudioDevice};
use sdl2::rect::Rect;
use sdl2::{pixels::Color, keyboard::Keycode, EventPump, render::Canvas};

const PIXEL_SCALE: u32 = 16;

fn main() {
    let mut is_running = true;
    let mut buzzer_on = false;

    let (audio_device, mut canvas, mut event_pump) = init_sdl();

    let mut rom = Vec::new();
    let mut rom_file = File::open("Pong.ch8").expect("Unable to open ROM file.");
    rom_file.read_to_end(&mut rom).expect("Unable to read ROM file.");

    let mut cpu = cpu::Cpu::new();
    cpu.load_rom(&rom, false);

    let mut tick_counter = 0u32;
    loop {
        if !is_running {
            break;
        }

        if buzzer_on {
            audio_device.resume();
        } else {
            audio_device.pause();
        }

        let events = handle_event_loop(&mut event_pump);

        if events.contains(&Event::Quit) {
            is_running = false;
        }

        cpu.tick();

        if tick_counter % 10 == 0 {
            update_display(&mut canvas, &cpu.display);
            buzzer_on = cpu.decrement_timers();
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 600));

        tick_counter += 1;
    }
}

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

fn init_sdl() -> (AudioDevice<SquareWave>, Canvas<sdl2::video::Window>, EventPump){
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let audio = sdl_context.audio().unwrap();
    let event_pump = sdl_context.event_pump().unwrap();

    let window = video.window(
        "Chip 8", 
        display::DISPLAY_WIDTH as u32 * PIXEL_SCALE, 
        display::DISPLAY_HEIGHT as u32 * PIXEL_SCALE
    ).position_centered().build().unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(255, 0, 255));
    canvas.clear();
    canvas.present();

    let desired_audio_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: None,
    };

    let device = audio.open_playback(None, &desired_audio_spec, | spec | {
        SquareWave {
            phase_inc: 440.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.25
        }
    }).unwrap();

    return (device, canvas, event_pump);
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
            sdl2::event::Event::KeyDown { keycode: Some(code), ..} => {
                println!("Key pressed: {}", code);

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

fn update_display(canvas: &mut Canvas<sdl2::video::Window>, display: &Display) {
    let mut data_index = 0;
    for y_index in 0..(display::DISPLAY_HEIGHT as u32) {
        for x_index in 0..(display::DISPLAY_WIDTH as u32) {
            let color = display.data[data_index];
            canvas.set_draw_color(Color::RGB(color, color, color));

            let _ = canvas.fill_rect(Rect::new((x_index * PIXEL_SCALE) as i32, (y_index * PIXEL_SCALE) as i32, PIXEL_SCALE, PIXEL_SCALE));

            data_index += 1;
        }
    }
    
    canvas.present();
}