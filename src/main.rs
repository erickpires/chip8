mod cpu;
mod instruction;
mod display;
mod sdl_wrapper;

extern crate sdl2;

use std::env;
use std::{fs::File, io::Read};
use std::time::Duration;

const CLOCKS_PER_FRAME: u32 = 16;

#[derive(PartialEq)]
enum Event {
    Quit,
    KeyPressed(u8),
    KeyReleased(u8),
}

fn main() {
    let mut keys_held = [false; 16];
    let mut is_running = true;
    let mut buzzer_on = false;

    let (audio_device, mut canvas, mut event_pump) = sdl_wrapper::init_sdl();

    let args = env::args().collect::<Vec<_>>();

    let rom_path = args.get(1).unwrap_or_else(|| panic!("Usage: {} <ROM path>.", args[0]));

    let mut rom = Vec::new();
    let mut rom_file = File::open(rom_path).expect("Unable to open ROM file.");
    rom_file.read_to_end(&mut rom).expect("Unable to read ROM file.");

    let mut cpu = cpu::Cpu::new();
    cpu.load_rom(&rom, false);

    let mut clock_counter = 0u32;
    loop {
        if !is_running {
            break;
        }

        if buzzer_on {
            audio_device.resume();
        } else {
            audio_device.pause();
        }

        let events = sdl_wrapper::handle_event_loop(&mut event_pump);

        if events.contains(&Event::Quit) {
            is_running = false;
        }

        for event in events {
            match event {
                Event::Quit => {
                    is_running = false;
                },
                Event::KeyPressed(key_num) => {
                    keys_held[key_num as usize] = true;
                },
                Event::KeyReleased(key_num) => {
                    keys_held[key_num as usize] = false;
                },
            }
        }

        cpu.display.fade_pixels();

        let keys = keys_held
            .into_iter()
            .enumerate()
            .filter_map(|(index, is_held)| if is_held { Some(index as u8) } else { None })
            .collect();

        cpu.tick(keys);

        clock_counter += 1;
        if clock_counter == CLOCKS_PER_FRAME { // NEW FRAME
            sdl_wrapper::update_display(&mut canvas, &cpu.display);
            buzzer_on = cpu.decrement_timers();

            clock_counter = 0;
        }

        // TODO: Actually we need to figure out how much time we used in the current clock cicle
        // and wait only the amount of time until the next clock.
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / (60 * CLOCKS_PER_FRAME)));
    }
}
