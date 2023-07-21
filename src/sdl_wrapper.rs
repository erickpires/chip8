use sdl2::audio::{AudioSpecDesired, AudioCallback, AudioDevice};
use sdl2::rect::Rect;
use sdl2::{pixels::Color, keyboard::Keycode, EventPump, render::Canvas};

use crate::display::{Display, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::Event;


const PIXEL_SCALE: u32 = 16;

pub(crate) struct SquareWave {
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

pub(crate) fn init_sdl() -> (AudioDevice<SquareWave>, Canvas<sdl2::video::Window>, EventPump){
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let audio = sdl_context.audio().unwrap();
    let event_pump = sdl_context.event_pump().unwrap();

    let window = video.window(
        "Chip 8", 
        DISPLAY_WIDTH as u32 * PIXEL_SCALE, 
        DISPLAY_HEIGHT as u32 * PIXEL_SCALE
    ).position_centered().build().unwrap();

    let canvas = window.into_canvas().build().unwrap();

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

    (device, canvas, event_pump)
}

pub(crate) fn handle_event_loop(event_pump: &mut EventPump) -> Vec<Event> {
    let mut result = Vec::new();

    for event in event_pump.poll_iter() {
        match event {
            sdl2::event::Event::Quit {..} => {
                result.push(Event::Quit)
            },
            sdl2::event::Event::KeyDown { keycode: Some(code), ..} => {
                if code == Keycode::Escape {
                    result.push(Event::Quit)
                }

                if let Some(key_num) = try_keycode_into_key_num(code) {
                    result.push(Event::KeyPressed(key_num))
                }
            },
            sdl2::event::Event::KeyUp { keycode: Some(code), ..} => {
                if let Some(key_num) = try_keycode_into_key_num(code) {
                    result.push(Event::KeyReleased(key_num))
                }
            },
            _ => { }
        }
    }

    result
}

fn try_keycode_into_key_num(keycode: Keycode) -> Option<u8> {
    match keycode {
        Keycode::Num1 => { Some(0x1) },
        Keycode::Num2 => { Some(0x2) },
        Keycode::Num3 => { Some(0x3) },
        Keycode::Num4 => { Some(0xC) },

        Keycode::Q => { Some(0x4) },
        Keycode::W => { Some(0x5) },
        Keycode::E => { Some(0x6) },
        Keycode::R => { Some(0xD) },

        Keycode::A => { Some(0x7) },
        Keycode::S => { Some(0x8) },
        Keycode::D => { Some(0x9) },
        Keycode::F => { Some(0xE) },

        Keycode::Z => { Some(0xA) },
        Keycode::X => { Some(0x0) },
        Keycode::C => { Some(0xB) },
        Keycode::V => { Some(0xF) },
        _ => None
    }
}

pub(crate) fn update_display(canvas: &mut Canvas<sdl2::video::Window>, display: &Display) {
    let mut data_index = 0;
    for y_index in 0..(DISPLAY_HEIGHT as u32) {
        for x_index in 0..(DISPLAY_WIDTH as u32) {
            let color = display.data[data_index];
            canvas.set_draw_color(Color::RGB(color, color, color));

            let _ = canvas.fill_rect(Rect::new((x_index * PIXEL_SCALE) as i32, (y_index * PIXEL_SCALE) as i32, PIXEL_SCALE, PIXEL_SCALE));

            data_index += 1;
        }
    }
    
    canvas.present();
}