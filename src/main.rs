extern crate sdl2;

use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::collections::HashMap;
use std::time::Duration;

mod cpu;
mod mem;
mod system;
mod video;
mod cpu_tests;

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            if self.phase >= 0.0 && self.phase <= 0.5 {
                *x = self.volume;
            } else {
                *x = -self.volume;
            }
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}
pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().expect("Unable to init audio");

    let window = video_subsystem
        .window(
            "chip8",
            (video::SCREEN_WIDTH * video::PIXEL_SIZE as usize) as u32,
            (video::SCREEN_HEIGHT * video::PIXEL_SIZE as usize) as u32,
        )
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    let mut emulator = system::System::new();

    emulator.load_game("<Your rom file here!>");

    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut keys = HashMap::new();
    keys.insert(Keycode::Num1, 0);
    keys.insert(Keycode::Num2, 0);
    keys.insert(Keycode::Num3, 0);
    keys.insert(Keycode::Num4, 0);
    keys.insert(Keycode::Q, 0);
    keys.insert(Keycode::W, 0);
    keys.insert(Keycode::E, 0);
    keys.insert(Keycode::R, 0);
    keys.insert(Keycode::A, 0);
    keys.insert(Keycode::S, 0);
    keys.insert(Keycode::D, 0);
    keys.insert(Keycode::F, 0);
    keys.insert(Keycode::Z, 0);
    keys.insert(Keycode::X, 0);
    keys.insert(Keycode::C, 0);
    keys.insert(Keycode::V, 0);

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1), // mono
        samples: None,     // default sample size
    };

    let device = audio_subsystem
        .open_playback(None, &desired_spec, |spec| {
            // initialize the audio callback
            SquareWave {
                phase_inc: 440.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.25,
            }
        })
        .unwrap();

    'running: loop {
        // Start playback
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode, .. } => match keys.get(&keycode.unwrap()) {
                    Some(_) => {
                        keys.insert(keycode.unwrap(), 1);
                    }
                    None => {}
                },
                Event::KeyUp { keycode, .. } => match keys.get(&keycode.unwrap()) {
                    Some(_) => {
                        keys.insert(keycode.unwrap(), 0);
                    }
                    None => {}
                },
                _ => {}
            }
        }
        emulator.tick(&keys);
        if emulator.should_draw() {
            canvas.clear();
            emulator.draw(&mut canvas);
            canvas.present();
        }
        if emulator.should_play_sound() {
            device.resume();
        } else {
            device.pause();
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 300));
    }
}
