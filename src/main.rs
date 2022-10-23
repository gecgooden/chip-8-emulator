use std::env;
use std::fs;
use std::process::exit;

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow;
use graphics::clear;
use graphics::rectangle;

use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{Button, RenderEvent, UpdateEvent};
use piston::window::WindowSettings;
use piston::EventLoop;
use piston::Key;
use piston::PressEvent;
use piston::ReleaseEvent;
use piston::Window;

use crate::chip8::Chip8;

pub mod chip8;

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

fn map_key_to_index(key: Key) -> Option<u8> {
    match &key {
        Key::D1 => return Option::Some(0x1),
        Key::D2 => return Option::Some(0x2),
        Key::D3 => return Option::Some(0x3),
        Key::D4 => return Option::Some(0xC),
        Key::Q => return Option::Some(0x4),
        Key::W => return Option::Some(0x5),
        Key::E => return Option::Some(0x6),
        Key::R => return Option::Some(0xD),
        Key::A => return Option::Some(0x7),
        Key::S => return Option::Some(0x8),
        Key::D => return Option::Some(0x9),
        Key::F => return Option::Some(0xE),
        Key::Z => return Option::Some(0xA),
        Key::X => return Option::Some(0x0),
        Key::C => return Option::Some(0xB),
        Key::V => return Option::Some(0xF),
        _ => {
            return Option::None;
        }
    }
}

fn main() {
    let opengl = OpenGL::V3_2;

    let mut window: GlutinWindow = WindowSettings::new("Chip8 Emulator", [640, 320])
        .graphics_api(opengl)
        .resizable(false)
        .samples(1)
        .vsync(true)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut gl = GlGraphics::new(opengl);

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("rom path not provided");
        exit(1);
    }

    let rom_path = &args[1];
    println!("rom path: {rom_path}");

    let rom = fs::read(rom_path).expect("error reading rom file");

    let mut chip8 = Chip8::new();
    chip8.load_rom(rom);

    let mut events = Events::new(EventSettings::new());
    events.set_ups(500);
    events.set_ups_reset(0);

    while let Some(e) = events.next(&mut window) {
        if let Some(Button::Keyboard(key)) = e.press_args() {
            if let Some(index) = map_key_to_index(key) {
                chip8.set_key(index, true);
            }
        }
        if let Some(Button::Keyboard(key)) = e.release_args() {
            if let Some(index) = map_key_to_index(key) {
                chip8.set_key(index, false);
            }
        }

        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, gl| {
                clear(BLACK, gl);
                for x in 0..64 {
                    for y in 0..32 {
                        let square = rectangle::square((x as f64) * 10.0, (y as f64) * 10.0, 10.0);
                        rectangle(
                            if chip8.get_display_buffer()[y][x] == true {
                                WHITE
                            } else {
                                BLACK
                            },
                            square,
                            c.transform,
                            gl,
                        );
                    }
                }
            });
        }

        if let Some(_args) = e.update_args() {
            let wait_key_closure = || loop {
                let event = window.wait_event();
                if let Some(Button::Keyboard(key)) = event.press_args() {
                    if let Some(index) = map_key_to_index(key) {
                        return index;
                    }
                }
            };
            chip8.execute_cycle(wait_key_closure);
        }
    }
}
