type DisplayBuffer = [[bool; 64]; 32];

pub struct Chip8 {
    memory: [u8; 4096],
    program_counter: u16,
    display_buffer: DisplayBuffer,
    stack: Vec<u16>,
    registers: [u8; 16],
    index_register: u16,
    keys: [bool; 16],

    delay_timer: u8,
    sound_timer: u8,
}

impl Chip8 {
    const FONT_SET: [u8; 80] = [
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
    pub fn new() -> Chip8 {
        let mut memory = [0; 4096];

        for (index, byte) in Chip8::FONT_SET.iter().enumerate() {
            memory[index] = *byte;
        }

        return Chip8 {
            memory,
            program_counter: 512, // 0x0200
            display_buffer: [[false; 64]; 32],
            stack: Vec::with_capacity(16),
            registers: [0; 16],
            index_register: 0,
            keys: [false; 16],

            delay_timer: 0,
            sound_timer: 0,
        };
    }

    pub fn set_key(&mut self, index: u8, pressed: bool) {
        self.keys[index as usize] = pressed;
    }

    pub fn get_display_buffer(&self) -> DisplayBuffer {
        return self.display_buffer;
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        for (index, byte) in rom.iter().enumerate() {
            self.memory[512 + index] = byte.clone();
        }
    }

    fn get_byte_from_memory(&self, address: u16) -> u8 {
        return self.memory[address as usize];
    }

    fn set_byte_in_memory(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }

    fn get_register_value(&self, index: u8) -> u8 {
        return self.registers[index as usize];
    }

    fn set_register_value(&mut self, index: u8, value: u8) {
        self.registers[index as usize] = value;
    }

    fn get_key_pressed(&self, index: u8) -> bool {
        return self.keys[index as usize];
    }

    pub fn execute_cycle<F>(&mut self, wait_for_input: F)
    where
        F: FnOnce() -> u8,
    {
        let opcode = u16::from_be_bytes([
            self.get_byte_from_memory(self.program_counter),
            self.get_byte_from_memory(self.program_counter + 1),
        ]);

        match Chip8::parse_instruction(opcode) {
            Instruction::ClearDisplay => {
                self.display_buffer = [[false; 64]; 32];
                self.program_counter += 2;
            }
            Instruction::Return => {
                let address = self.stack.pop().unwrap();
                self.program_counter = address;
                self.program_counter += 2;
            }
            Instruction::Jump(address) => {
                self.program_counter = address;
            }
            Instruction::Call(address) => {
                self.stack.push(self.program_counter.clone());
                self.program_counter = address;
            }
            Instruction::SkipEqualK(register, value) => {
                if self.get_register_value(register) == value {
                    self.program_counter += 2;
                }
                self.program_counter += 2;
            }
            Instruction::SkipNotEqualK(register, value) => {
                if self.get_register_value(register) != value {
                    self.program_counter += 2;
                }
                self.program_counter += 2;
            }
            Instruction::SkipEqual(x, y) => {
                if self.get_register_value(x) == self.get_register_value(y) {
                    self.program_counter += 2;
                }
                self.program_counter += 2;
            }
            Instruction::SetK(register, value) => {
                self.set_register_value(register, value);
                self.program_counter += 2;
            }
            Instruction::AddK(register, value) => {
                self.set_register_value(
                    register,
                    self.get_register_value(register).wrapping_add(value),
                );
                self.program_counter += 2;
            }
            Instruction::Set(x, y) => {
                self.set_register_value(x, self.get_register_value(y));
                self.program_counter += 2;
            }
            Instruction::Or(x, y) => {
                self.set_register_value(x, self.get_register_value(x) | self.get_register_value(y));
                self.program_counter += 2;
            }
            Instruction::And(x, y) => {
                self.set_register_value(x, self.get_register_value(x) & self.get_register_value(y));
                self.program_counter += 2;
            }
            Instruction::XOr(x, y) => {
                self.set_register_value(x, self.get_register_value(x) ^ self.get_register_value(y));
                self.program_counter += 2;
            }
            Instruction::Add(x, y) => {
                let is_carry = self
                    .get_register_value(x)
                    .checked_add(self.get_register_value(y))
                    == None;

                self.set_register_value(
                    x,
                    self.get_register_value(x)
                        .wrapping_add(self.get_register_value(y)),
                );
                if is_carry {
                    self.set_register_value(0xF, 1);
                }
                self.program_counter += 2;
            }
            Instruction::Sub(x, y) => {
                self.set_register_value(
                    0xF,
                    if self.get_register_value(x) < self.get_register_value(y) {
                        0
                    } else {
                        1
                    },
                );
                self.set_register_value(
                    x,
                    self.get_register_value(x)
                        .wrapping_sub(self.get_register_value(y)),
                );
                self.program_counter += 2;
            }
            Instruction::ShiftRight(x) => {
                self.set_register_value(0xF, self.get_register_value(x) % 2);
                self.set_register_value(x, self.get_register_value(x) >> 1);
                self.program_counter += 2;
            }
            Instruction::SubInv(x, y) => {
                self.set_register_value(
                    16,
                    if self.get_register_value(y) < self.get_register_value(x) {
                        0
                    } else {
                        1
                    },
                );
                self.set_register_value(
                    x,
                    self.get_register_value(y)
                        .wrapping_sub(self.get_register_value(x)),
                );
                self.program_counter += 2;
            }
            Instruction::ShiftLeft(x) => {
                self.set_register_value(
                    0xF,
                    if self.get_register_value(x) > (u8::MAX / 2) {
                        1
                    } else {
                        0
                    },
                );
                self.set_register_value(x, self.get_register_value(x) << 1);
                self.program_counter += 2;
            }
            Instruction::SkipNotEqual(x, y) => {
                if self.get_register_value(x) != self.get_register_value(y) {
                    self.program_counter += 2;
                }
                self.program_counter += 2;
            }
            Instruction::LoadI(address) => {
                self.index_register = address;
                self.program_counter += 2;
            }
            Instruction::LongJump(_address) => {
                panic!("GotoB has not been implemented")
            }
            Instruction::Rand(x, value) => {
                let random_number: u8 = rand::random();
                self.set_register_value(x, value & random_number);
                self.program_counter += 2;
            }
            Instruction::Draw(x, y, height) => {
                let x = self.get_register_value(x);
                let y = self.get_register_value(y);
                self.set_register_value(0xF, 0);
                for i in 0..height {
                    let pixel = self.get_byte_from_memory(self.index_register + (i as u16));
                    for j in 0..8 {
                        if (y + i) < 32 {
                            if (pixel & (0x80 >> j)) != 0 {
                                if self.display_buffer[((y + i) as usize)][(x + j) as usize] {
                                    self.set_register_value(0xF, 1);
                                }
                                self.display_buffer[(y + i) as usize][(x + j) as usize] ^= true;
                            }
                        }
                    }
                }
                self.program_counter += 2;
            }
            Instruction::SkipPressed(x) => {
                if self.get_key_pressed(self.get_register_value(x)) {
                    self.program_counter += 2;
                }
                self.program_counter += 2;
            }
            Instruction::SkipNotPressed(x) => {
                if !self.get_key_pressed(self.get_register_value(x)) {
                    self.program_counter += 2;
                }
                self.program_counter += 2;
            }
            Instruction::GetTimer(x) => {
                self.set_register_value(x, self.delay_timer);
                self.program_counter += 2;
            }
            Instruction::WaitKey(x) => {
                let index = wait_for_input();
                self.set_register_value(x, index);
                self.program_counter += 2;
            }
            Instruction::SetTimer(x) => {
                self.delay_timer = self.get_register_value(x);
                self.program_counter += 2;
            }
            Instruction::SetSoundTimer(x) => {
                self.sound_timer = self.get_register_value(x);
                self.program_counter += 2;
            }
            Instruction::AddToI(x) => {
                self.index_register += self.get_register_value(x) as u16;
                self.program_counter += 2;
            }
            Instruction::LoadHexGlyph(x) => {
                self.index_register = self.get_register_value(x) as u16 * 5;
                self.program_counter += 2;
            }
            Instruction::StoreBCD(x) => {
                self.set_byte_in_memory(self.index_register, self.get_register_value(x) / 100);
                self.set_byte_in_memory(
                    self.index_register + 1,
                    (self.get_register_value(x) / 10) % 10,
                );
                self.set_byte_in_memory(self.index_register + 2, self.get_register_value(x) % 10);
                self.program_counter += 2;
            }
            Instruction::StoreRegisters(x) => {
                for i in 0..(x + 1) {
                    self.set_byte_in_memory(
                        self.index_register + (i as u16),
                        self.get_register_value(i),
                    )
                }
                self.program_counter += 2;
            }
            Instruction::LoadRegisters(x) => {
                for i in 0..(x + 1) {
                    self.set_register_value(
                        i,
                        self.get_byte_from_memory(self.index_register + (i as u16)),
                    );
                }
                self.program_counter += 2;
            }
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer == 1 {
            println!("BEEP!");
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    fn parse_instruction(opcode: u16) -> Instruction {
        let nibbles = [
            (opcode & 0xF000) >> 12,
            (opcode & 0x0F00) >> 8,
            (opcode & 0x00F0) >> 4,
            (opcode & 0x000F) >> 0,
        ];
        let x = nibbles[1] as u8;
        let y = nibbles[2] as u8;
        let n = nibbles[3] as u8;
        let nn = ((nibbles[2] << 4) + nibbles[3]) as u8;
        let nnn = (nibbles[1] << 8) + (nibbles[2] << 4) + nibbles[3];
        match nibbles {
            [0x0, 0x0, 0xE, 0x0] => return Instruction::ClearDisplay,
            [0x0, 0x0, 0xE, 0xE] => return Instruction::Return,
            [0x1, _, _, _] => return Instruction::Jump(nnn),
            [0x2, _, _, _] => return Instruction::Call(nnn),
            [0x3, _, _, _] => return Instruction::SkipEqualK(x, nn),
            [0x4, _, _, _] => return Instruction::SkipNotEqualK(x, nn),
            [0x5, _, _, 0x0] => return Instruction::SkipEqual(x, y),
            [0x6, _, _, _] => return Instruction::SetK(x, nn),
            [0x7, _, _, _] => return Instruction::AddK(x, nn),
            [0x8, _, _, 0x0] => return Instruction::Set(x, y),
            [0x8, _, _, 0x1] => return Instruction::Or(x, y),
            [0x8, _, _, 0x2] => return Instruction::And(x, y),
            [0x8, _, _, 0x3] => return Instruction::XOr(x, y),
            [0x8, _, _, 0x4] => return Instruction::Add(x, y),
            [0x8, _, _, 0x5] => return Instruction::Sub(x, y),
            [0x8, _, _, 0x6] => return Instruction::ShiftRight(x),
            [0x8, _, _, 0x7] => return Instruction::SubInv(x, y),
            [0x8, _, _, 0xE] => return Instruction::ShiftLeft(x),
            [0x9, _, _, 0x0] => return Instruction::SkipNotEqual(x, y),
            [0xA, _, _, _] => return Instruction::LoadI(nnn),
            [0xB, _, _, _] => return Instruction::LongJump(nnn),
            [0xC, _, _, _] => return Instruction::Rand(x, nn),
            [0xD, _, _, _] => return Instruction::Draw(x, y, n),
            [0xE, _, 0x9, 0xE] => return Instruction::SkipPressed(x),
            [0xE, _, 0xA, 0x1] => return Instruction::SkipNotPressed(x),
            [0xF, _, 0x0, 0x7] => return Instruction::GetTimer(x),
            [0xF, _, 0x0, 0xA] => return Instruction::WaitKey(x),
            [0xF, _, 0x1, 0x5] => return Instruction::SetTimer(x),
            [0xF, _, 0x1, 0x8] => return Instruction::SetSoundTimer(x),
            [0xF, _, 0x1, 0xE] => return Instruction::AddToI(x),
            [0xF, _, 0x2, 0x9] => return Instruction::LoadHexGlyph(x),
            [0xF, _, 0x3, 0x3] => return Instruction::StoreBCD(x),
            [0xF, _, 0x5, 0x5] => return Instruction::StoreRegisters(x),
            [0xF, _, 0x6, 0x5] => return Instruction::LoadRegisters(x),

            _ => panic!("Unexpected opcode: {opcode:#06X}"),
        }
    }
}

type Address = u16;

type RegisterNumber = u8;

enum Instruction {
    ClearDisplay,
    Return,
    Jump(Address),
    Call(Address),
    SkipEqualK(RegisterNumber, u8),
    SkipNotEqualK(RegisterNumber, u8),
    SkipEqual(RegisterNumber, RegisterNumber),
    SetK(RegisterNumber, u8),
    AddK(RegisterNumber, u8),
    Set(RegisterNumber, RegisterNumber),
    Or(RegisterNumber, RegisterNumber),
    And(RegisterNumber, RegisterNumber),
    XOr(RegisterNumber, RegisterNumber),
    Add(RegisterNumber, RegisterNumber),
    Sub(RegisterNumber, RegisterNumber),
    ShiftRight(RegisterNumber),
    SubInv(RegisterNumber, RegisterNumber),
    ShiftLeft(RegisterNumber),
    SkipNotEqual(RegisterNumber, RegisterNumber),
    LoadI(Address),
    LongJump(Address),
    Rand(RegisterNumber, u8),
    Draw(RegisterNumber, RegisterNumber, u8),
    SkipPressed(RegisterNumber),
    SkipNotPressed(RegisterNumber),
    GetTimer(RegisterNumber),
    WaitKey(RegisterNumber),
    SetTimer(RegisterNumber),
    SetSoundTimer(RegisterNumber),
    AddToI(RegisterNumber),
    LoadHexGlyph(RegisterNumber),
    StoreBCD(RegisterNumber),
    StoreRegisters(RegisterNumber),
    LoadRegisters(RegisterNumber),
}
