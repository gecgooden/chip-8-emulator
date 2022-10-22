use std::env;
use std::fs;
use std::process::exit;

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow;
use graphics::rectangle;

use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderEvent, UpdateEvent};
use piston::window::WindowSettings;
use piston::EventLoop;
use piston::PressEvent;

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: GlutinWindow = WindowSettings::new("Chip8 Emulator", [640, 320])
        .graphics_api(opengl)
        .resizable(false)
        .samples(1)
        .vsync(true)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut gl = GlGraphics::new(opengl);

    let font_set: [u8; 80] = [
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80, // F
    ];

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("rom path not provided");
        exit(1);
    }

    let mut display: [[bool; 64]; 32] = [[false; 64]; 32];

    let rom_path = &args[1];
    println!("rom path: {rom_path}");

    let rom = fs::read(rom_path).expect("error reading rom file");
    let mut memory: [u8; 4096] = [0; 4096];
    for (index, byte) in font_set.iter().enumerate() {
        memory[index] = *byte;
    }
    for (index, byte) in rom.iter().enumerate() {
        memory[512 + index] = *byte;
    }

    let mut registers: [u8; 16] = [0; 16];
    let mut index_register: u16 = 0;
    let mut stack = Vec::new();

    let mut program_counter = 0x200;

    let mut events = Events::new(EventSettings::new());
    events.set_ups(500);
    events.set_ups_reset(0);
    // events.set_max_fps(240);
    println!("{:?}", events.get_event_settings());
    use piston::input::{Button, Key};

    while let Some(e) = events.next(&mut window) {
        if let Some(Button::Keyboard(key)) = e.press_args() {
            println!("{}", Key::code(&key));
        }
        if let Some(args) = e.render_args() {
            // println!("{}", args.ext_dt);
            const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
            const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
            gl.draw(args.viewport(), |c, gl| {
                for (y, row) in display.iter().enumerate() {
                    for (x, pixel) in row.iter().enumerate() {
                        let square = rectangle::square((x as f64) * 10.0, (y as f64) * 10.0, 10.0);
                        rectangle(
                            if *pixel == true { WHITE } else { BLACK },
                            square,
                            c.transform,
                            gl,
                        );
                    }
                }
            });
        }

        if let Some(_args) = e.update_args() {
            let opcode: u16 =
                u16::from_be_bytes([memory[program_counter], memory[program_counter + 1]]);
            match opcode {
                0x00E0 => {
                    display = [[false; 64]; 32];
                    program_counter += 2;
                }
                0x00EE => {
                    let address = stack.pop().expect("not to be last");
                    program_counter = address + 2;
                }
                0x1000..=0x1FFF => {
                    let address = opcode & 0x0FFF;
                    program_counter = address as usize;
                }
                0x2000..=0x2FFF => {
                    stack.push(program_counter.clone());
                    let address = opcode & 0x0FFF;
                    program_counter = address as usize;
                }
                0x3000..=0x3FFF => {
                    let register_number = opcode.to_be_bytes()[0] & 0x0F;
                    if registers[register_number as usize] == opcode.to_be_bytes()[1] {
                        program_counter += 4;
                    } else {
                        program_counter += 2;
                    }
                }
                0x4000..=0x4FFF => {
                    let register_number = opcode.to_be_bytes()[0] & 0x0F;
                    if registers[register_number as usize] != opcode.to_be_bytes()[1] {
                        program_counter += 4;
                    } else {
                        program_counter += 2;
                    }
                }
                0x5000..=0x5FFF => {
                    let x_register = opcode.to_be_bytes()[0] & 0x0F;
                    let y_register = (opcode & 0x00F0) / 16;
                    let x = registers[x_register as usize] as u16;
                    let y = registers[y_register as usize] as u16;
                    if x == y {
                        program_counter += 2;
                    }
                    program_counter += 2;
                }
                0x6000..=0x6FFF => {
                    let register_number = opcode.to_be_bytes()[0] & 0x0F;
                    registers[register_number as usize] = opcode.to_be_bytes()[1];
                    program_counter += 2;
                }
                0x7000..=0x7FFF => {
                    let bytes = opcode.to_be_bytes();
                    let register_number = bytes[0] & 0x0F;
                    let value = bytes[1];
                    registers[register_number as usize] =
                        registers[register_number as usize].wrapping_add(value);

                    program_counter += 2;
                }
                0x8000..=0x8FFF => {
                    let bytes = opcode.to_be_bytes();
                    match bytes[1] & 0x0F {
                        0x00 => {
                            let x_register = opcode.to_be_bytes()[0] & 0x0F;
                            let y_register = (opcode & 0x00F0) / 16;
                            registers[x_register as usize] = registers[y_register as usize];
                            program_counter += 2;
                        }
                        0x01 => {
                            let x_register = opcode.to_be_bytes()[0] & 0x0F;
                            let y_register = (opcode & 0x00F0) / 16;
                            registers[x_register as usize] |= registers[y_register as usize];
                            program_counter += 2;
                        }
                        0x02 => {
                            let x_register = opcode.to_be_bytes()[0] & 0x0F;
                            let y_register = (opcode & 0x00F0) / 16;
                            registers[x_register as usize] &= registers[y_register as usize];
                            program_counter += 2;
                        }
                        0x03 => {
                            let x_register = opcode.to_be_bytes()[0] & 0x0F;
                            let y_register = (opcode & 0x00F0) / 16;
                            registers[x_register as usize] ^= registers[y_register as usize];
                            program_counter += 2;
                        }
                        0x04 => {
                            // println!("matched 0x04 (0xFXY4)");
                            let x_register = opcode.to_be_bytes()[0] & 0x0F;
                            let y_register = (opcode & 0x00F0) / 16;
                            let x = registers[x_register as usize] as u16;
                            let y = registers[y_register as usize] as u16;
                            let result: u16 = x + y;
                            registers[x_register as usize] = (result & 0x00FF) as u8;
                            if result > 0xFF {
                                registers[0xF] = 1;
                            }
                            program_counter += 2;
                        }
                        0x05 => {
                            let x_register = opcode.to_be_bytes()[0] & 0x0F;
                            let y_register = (opcode & 0x00F0) / 16;
                            let x = registers[x_register as usize] as u8;
                            let y = registers[y_register as usize] as u8;
                            if x < y {
                                registers[0xF] = 0;
                            } else {
                                registers[0xF] = 1;
                            }
                            registers[x_register as usize] = x.wrapping_sub(y);
                            program_counter += 2;
                        }
                        0x06 => {
                            let x_register = opcode.to_be_bytes()[0] & 0x0F;
                            let x = registers[x_register as usize] as u8;
                            registers[0xF] = x % 2;
                            registers[x_register as usize] >>= 1;
                            program_counter += 2;
                        }
                        0x0E => {
                            let x_register = opcode.to_be_bytes()[0] & 0x0F;
                            let x = registers[x_register as usize] as u8;
                            if x > u8::MAX / 2 {
                                registers[0xF] = 1;
                            } else {
                                registers[0xF] = 0;
                            }
                            registers[x_register as usize] <<= 1;
                            program_counter += 2;
                        }
                        _ => panic!("8 input didn't match anything. Opcode: {opcode:#04X}"),
                    }
                }
                0x9000..=0x9FFF => {
                    let x_register = opcode.to_be_bytes()[0] & 0x0F;
                    let y_register = (opcode & 0x00F0) / 16;
                    let x = registers[x_register as usize] as u16;
                    let y = registers[y_register as usize] as u16;
                    if x != y {
                        program_counter += 2;
                    }
                    program_counter += 2;
                }
                0xA000..=0xAFFF => {
                    index_register = opcode & 0x0FFF;
                    program_counter += 2;
                }
                0xC000..=0xCFFF => {
                    let bytes = opcode.to_be_bytes();
                    let register_number = bytes[0] & 0x0F;
                    let random_number: u8 = rand::random();
                    registers[register_number as usize] = bytes[1] & random_number;
                    program_counter += 2;
                }
                0xD000..=0xDFFF => {
                    // println!("matched 0xD000..0xDFFF");
                    let x = registers[(opcode.to_be_bytes()[0] & 0x0F) as usize];
                    let y = registers[((opcode & 0x00F0) / 16) as usize];
                    let height = opcode.to_be_bytes()[1] & 0x0F;
                    registers[0x0F] = 0;
                    for i in 0..height {
                        let pixel = memory[(index_register + (i as u16)) as usize];
                        for j in 0..8 {
                            if (pixel & (0x80 >> j)) != 0 {
                                if display[(y + i) as usize][(x + j) as usize] {
                                    registers[0x0F] = 1;
                                }
                                display[(y + i) as usize][(x + j) as usize] ^= true;
                            }
                        }
                    }

                    program_counter += 2;
                }
                0xF000..=0xFFFF => {
                    let bytes = opcode.to_be_bytes();
                    match bytes[1] {
                        0x0A => {
                            // let register_number = bytes[0] & 0x0F;
                            program_counter += 2;
                        }
                        0x15 => {
                            // let register_number = bytes[0] & 0x0F;
                            program_counter += 2;
                        }
                        0x18 => {
                            // let register_number = bytes[0] & 0x0F;
                            program_counter += 2;
                        }
                        0x29 => {
                            let register_number = bytes[0] & 0x0F;
                            index_register = (registers[register_number as usize] as u16) * 5;
                            program_counter += 2;
                        }
                        0x33 => {
                            let x_register = bytes[0] & 0x0F;
                            memory[index_register as usize] = registers[x_register as usize] / 100;
                            memory[index_register as usize + 1] =
                                (registers[x_register as usize] / 10) % 10;
                            memory[index_register as usize + 2] =
                                registers[x_register as usize] % 10;
                            program_counter += 2;
                        }
                        0x55 => {
                            let x_register = bytes[0] & 0x0F;
                            for i in 0..((x_register as u16) + 1) {
                                memory[(index_register + i) as usize] = registers[i as usize];
                            }
                            program_counter += 2;
                        }
                        0x65 => {
                            let x_register = bytes[0] & 0x0F;
                            for i in 0..((x_register as usize) + 1) {
                                registers[i] = memory[(index_register as usize) + i];
                            }
                            program_counter += 2;
                        }
                        _ => panic!("F input didn't match anything. Opcode: {opcode:#04X}"),
                    }
                }
                _ => {
                    panic!("input didn't match anything. Opcode: {opcode:#04X}");
                }
            }
        }
    }
}
