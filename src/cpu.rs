use std::collections::HashSet;

use crate::display::Display;
use crate::instruction::{Instruction, OpCode, OperandType};

const FONT_START_ADDR: usize = 0x50;
const ROM_START_ADDR: usize = 0x200;

const FONT_LINES_PER_CHAR: usize = 5;
const DEFAULT_FONT: [u8; 16 * FONT_LINES_PER_CHAR] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub(crate) struct Cpu {
    pub(crate) display: Display,

    memory: [u8; 4 * 1024],
    registers: [u8; 16],
    program_counter: u16,
    index_register: u16,
    delay_timer: u8,
    sound_timer: u8,

    stack: Vec<u16>,

    compatibility_mode: bool,
}

impl Cpu {
    pub(crate) const fn new() -> Self {
        Self { 
            memory: [0; 4 * 1024], 
            registers: [0; 16], 
            program_counter: 0, 
            index_register: 0, 
            delay_timer: 0, 
            sound_timer: 0,

            stack: Vec::new(), 
            display: Display::new(),

            compatibility_mode: false,
        }
    }

    pub(crate) fn load_rom(&mut self, rom: &Vec<u8>, compatibity_mode: bool) {
        self.compatibility_mode = compatibity_mode;
        self.program_counter = ROM_START_ADDR as u16;
        
        let mut font_addr = FONT_START_ADDR;
        for char_line in DEFAULT_FONT {
            self.memory[font_addr] = char_line;
            font_addr += 1;
        }

        let mut rom_addr = ROM_START_ADDR;
        for rom_byte in rom {
            self.memory[rom_addr] = *rom_byte;
            rom_addr += 1;
        }
    }

    pub(crate) fn tick(&mut self, keys: HashSet<u8>) {
        let pc = self.program_counter as usize;
        let instruction_hi = self.memory[pc];
        let instruction_lo = self.memory[pc + 1];
        let instruction_word = ((instruction_hi as u16) << 8) | (instruction_lo as u16);
        self.program_counter += 2;

        let instruction: Instruction = instruction_word.into();

        match instruction.op_code {
            OpCode::ClearDisplay => {
                self.display.clear();
            },
            OpCode::DrawSprite => {
                let x_coord = self.registers[instruction.x_register_index];
                let y_coord = self.registers[instruction.y_register_index];

                let sprite_start_index = self.index_register as usize;
                let sprite_len = instruction.immediate_half_byte as usize;

                let sprite = &self.memory[sprite_start_index..(sprite_start_index + sprite_len)];

                let has_colision = self.display.draw_sprite(x_coord, y_coord, sprite);
                self.registers[0xF] = if has_colision { 0x01 } else { 0x00 };
            },
            OpCode::JumpAbsolute => {
                self.program_counter = instruction.immediate_word;
            },
            OpCode::JumpWithOffset => {
                let offset = if self.compatibility_mode { self.registers[0] } else { self.registers[instruction.x_register_index] };

                let jump_target = instruction.immediate_word + (offset as u16);
                self.program_counter = jump_target;
            },
            OpCode::CallSubrotine => {
                self.stack.push(self.program_counter);
                self.program_counter = instruction.immediate_word;
            },
            OpCode::ReturnFromSubrotine => {
                self.program_counter = self.stack.pop().expect("Tried to pop, but stack is empty.");
            },
            OpCode::SkipIfEqual { operand_type } => {
                let lhs = self.registers[instruction.x_register_index];
                let rhs = match operand_type {
                    OperandType::Register => { self.registers[instruction.y_register_index] },
                    OperandType::Immediate => { instruction.immediate_byte },
                };

                if lhs == rhs {
                    self.program_counter += 2;
                }
            },
            OpCode::SkipIfNotEqual { operand_type } => {
                let lhs = self.registers[instruction.x_register_index];
                let rhs = match operand_type {
                    OperandType::Register => { self.registers[instruction.y_register_index] },
                    OperandType::Immediate => { instruction.immediate_byte },
                };

                if lhs != rhs {
                    self.program_counter += 2;
                }
            },
            OpCode::ArithmeticLogic { operand_type, operation } => {
                let lhs = self.registers[instruction.x_register_index];
                let rhs = match operand_type {
                    OperandType::Register => { self.registers[instruction.y_register_index] },
                    OperandType::Immediate => { instruction.immediate_byte },
                };

                let (result, carry) = operation.perform(lhs, rhs, self.compatibility_mode);
                self.registers[instruction.x_register_index] = result;

                if let Some(carry_value) = carry {
                    self.registers[0xF] = carry_value;
                }
            },
            OpCode::SetIndexRegister => {
                self.index_register = instruction.immediate_word;
            },
            OpCode::AddToIndexRegister => {
                self.index_register += self.registers[instruction.x_register_index] as u16;
            },
            OpCode::SetIndexRegisterToFont => {
                let char_index = self.registers[instruction.x_register_index] as usize;

                self.index_register = (FONT_START_ADDR + (char_index * FONT_LINES_PER_CHAR)) as u16;
            },
            OpCode::SetDelayRegister => {
                self.delay_timer = self.registers[instruction.x_register_index];
            },
            OpCode::SetSoundRegister => {
                self.sound_timer = self.registers[instruction.x_register_index];
            },
            OpCode::ReadDelayRegister => {
                self.registers[instruction.x_register_index] = self.delay_timer;
            },
            OpCode::SkipIfKeyPressed => {
                let expected_key = self.registers[instruction.x_register_index];

                if keys.contains(&expected_key) {
                    self.program_counter += 2;
                }
             },
            OpCode::SkipIfKeyNotPressed => { 
                let expected_key = self.registers[instruction.x_register_index];

                if !keys.contains(&expected_key) {
                    self.program_counter += 2;
                }
            },
            OpCode::WaitForKeyPress => {
                if let Some(key_num) = keys.iter().next() { 
                    self.registers[instruction.x_register_index] = *key_num;   
                } else { // Loop
                    self.program_counter -= 2;
                }
            },
            OpCode::DecodeBCD => {
                let value = self.registers[instruction.x_register_index];
                let hundreds = value / 100;
                let tens = (value % 100) / 10;
                let units = value % 10;

                let memory_index = self.index_register as usize;
                self.memory[memory_index] = hundreds;
                self.memory[memory_index + 1] = tens;
                self.memory[memory_index + 2] = units;
            },
            OpCode::SaveRegisters => {
                let mut memory_index = self.index_register as usize;

                for register_index in 0..=instruction.x_register_index {
                    self.memory[memory_index] = self.registers[register_index];

                    memory_index += 1;
                }

                self.index_register += instruction.x_register_index as u16 + 1;
            },
            OpCode::LoadRegisters => {
                let mut memory_index = self.index_register as usize;

                for register_index in 0..=instruction.x_register_index {
                    self.registers[register_index] = self.memory[memory_index];

                    memory_index += 1;
                }

                self.index_register += instruction.x_register_index as u16 + 1;
            },
            OpCode::Rand => {
                self.registers[instruction.x_register_index] = rand::random::<u8>() & instruction.immediate_byte;
            },
            OpCode::Unknown(instruction_word) => {
                panic!("Unknown instruction: 0x{:04X}", instruction_word);
            },
        }
    }

    pub(crate) fn decrement_timers(&mut self) -> bool {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }

        self.sound_timer > 0
    }
}

