use crate::display::Display;

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

#[derive(Debug)]
enum OperandType {
    Register,
    Immediate
}

#[derive(Debug)]
enum ALUOperation {
    SetValue,
    Add,
    Sub,
    SubAndNegate,
    BitwiseOr,
    BitwiseAnd,
    BitwiseXor,
    ShiftLeft,
    ShiftRight,
}

#[derive(Debug)]
enum OpCode {
    ClearDisplay,
    DrawSprite,
    ReturnFromSubrotine,
    JumpAbsolute,
    JumpWithOffset,
    CallSubrotine,
    SkipIfEqual { operand_type: OperandType },
    SkipIfNotEqual { operand_type: OperandType },
    ArithmeticLogic { operand_type: OperandType, operation: ALUOperation },
    SetIndexRegister,
    AddToIndexRegister,
    SetIndexRegisterToFont,
    SetDelayRegister,
    SetSoundRegister,
    ReadDelayRegister,
    SkipIfKeyPressed,
    SkipIfKeyNotPressed,
    WaitForKeyPress,
    DecodeBCD,
    SaveRegisters,
    LoadRegisters,
    Rand,
    Unknown (u16)
}

struct Instruction {
    op_code: OpCode,
    // These indices are actually half bytes, but using a usize avoids
    // some casting latter, when indexing the register bank.
    x_register_index: usize,
    y_register_index: usize,

    immediate_half_byte: u8,
    immediate_byte: u8,
    immediate_word: u16,
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

    pub(crate) fn tick(&mut self) {
        let pc = self.program_counter as usize;
        let instruction_hi = self.memory[pc];
        let instruction_lo = self.memory[pc + 1];
        let instruction_word = ((instruction_hi as u16) << 8) | (instruction_lo as u16);
        self.program_counter += 2;

        let instruction: Instruction = instruction_word.into();

        println!("Instruction: {:?}", instruction.op_code);

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

                self.display.draw_sprite(x_coord, y_coord, sprite);
            },
            OpCode::JumpAbsolute => {
                self.program_counter = instruction.immediate_word;
            },
            OpCode::JumpWithOffset => {
                let offset = self.registers[instruction.x_register_index] as u16;

                let jump_target = instruction.immediate_word + offset;
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

                let (result, carry) = operation.perform(lhs, rhs);
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
            OpCode::SkipIfKeyPressed => todo!(),
            OpCode::SkipIfKeyNotPressed => todo!(),
            OpCode::WaitForKeyPress => todo!(),
            OpCode::DecodeBCD => {
                let value = self.registers[instruction.x_register_index];
                let hundreds = value / 100;
                let tens = (value % 100) / 10;
                let units = value % 10;

                let memory_index = self.index_register as usize;
                self.memory[memory_index + 0] = hundreds;
                self.memory[memory_index + 1] = tens;
                self.memory[memory_index + 2] = units;
            },
            OpCode::SaveRegisters => {
                let mut memory_index = self.index_register as usize;

                for register_index in 0..=instruction.x_register_index {
                    self.memory[memory_index] = self.registers[register_index];

                    memory_index += 1;
                }
            },
            OpCode::LoadRegisters => {
                let mut memory_index = self.index_register as usize;

                for register_index in 0..=instruction.x_register_index {
                    self.registers[register_index] = self.memory[memory_index];

                    memory_index += 1;
                }
            },
            OpCode::Rand => {
                self.registers[instruction.x_register_index] = rand::random::<u8>() & instruction.immediate_byte;
            },
            OpCode::Unknown(instruction_word) => {
                panic!("Unknown instruction: 0x{:04X}", instruction_word);
            },
        }
    }
}

impl From<u16> for Instruction {
    fn from(word: u16) -> Self {
        let op_code = word.try_into().ok();
        Self { 
            op_code: op_code.unwrap_or(OpCode::Unknown(word)), 
            x_register_index: ((word >> 8) & 0xF) as usize,
            y_register_index: ((word >> 4) & 0xF) as usize, 
            immediate_half_byte: (word & 0xF) as u8, 
            immediate_byte: (word & 0xFF) as u8, 
            immediate_word: (word & 0xFFF) 
        }
    }
}

impl TryFrom<u16> for OpCode {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value == 0x00E0 {
            return Ok(Self::ClearDisplay);
        }

        if value == 0x00EE {
            return Ok(Self::ReturnFromSubrotine)
        }
        let higher_nibble = value >> 12;
        let lower_nibble = value & 0xF;
        let lower_byte = value & 0xFF;

        match (higher_nibble, lower_nibble) {
            (0x1, _) => { return Ok(Self::JumpAbsolute) },
            (0x2, _) => { return Ok(Self::CallSubrotine) },
            (0x3, _) => { return Ok(Self::SkipIfEqual { operand_type: OperandType::Immediate }) },
            (0x4, _) => { return Ok(Self::SkipIfNotEqual { operand_type: OperandType::Immediate }) },
            (0x5, 0x0) => { return Ok(Self::SkipIfEqual { operand_type: OperandType::Register }) },
            (0x6, _) => { return Ok(Self::ArithmeticLogic { operand_type: OperandType::Immediate, operation: ALUOperation::SetValue }) },
            (0x7, _) => { return Ok(Self::ArithmeticLogic { operand_type: OperandType::Immediate, operation: ALUOperation::Add }) },
            (0x8, 0x0) => { return Ok(Self::ArithmeticLogic { operand_type: OperandType::Register, operation: ALUOperation::SetValue }) },
            (0x8, 0x1) => { return Ok(Self::ArithmeticLogic { operand_type: OperandType::Register, operation: ALUOperation::BitwiseOr }) },
            (0x8, 0x2) => { return Ok(Self::ArithmeticLogic { operand_type: OperandType::Register, operation: ALUOperation::BitwiseAnd }) },
            (0x8, 0x3) => { return Ok(Self::ArithmeticLogic { operand_type: OperandType::Register, operation: ALUOperation::BitwiseXor }) },
            (0x8, 0x4) => { return Ok(Self::ArithmeticLogic { operand_type: OperandType::Register, operation: ALUOperation::Add }) },
            (0x8, 0x5) => { return Ok(Self::ArithmeticLogic { operand_type: OperandType::Register, operation: ALUOperation::Sub }) },
            (0x8, 0x6) => { return Ok(Self::ArithmeticLogic { operand_type: OperandType::Register, operation: ALUOperation::ShiftRight }) },
            (0x8, 0x7) => { return Ok(Self::ArithmeticLogic { operand_type: OperandType::Register, operation: ALUOperation::SubAndNegate }) },
            (0x8, 0xE) => { return Ok(Self::ArithmeticLogic { operand_type: OperandType::Register, operation: ALUOperation::ShiftLeft }) },
            (0x9, 0x0) => { return Ok(Self::SkipIfNotEqual { operand_type: OperandType::Register }) },
            (0xA, _) => { return Ok(Self::SetIndexRegister) },
            (0xB, _) => { return Ok(Self::JumpWithOffset) },
            (0xC, _) => { return Ok(Self::Rand) },
            (0xD, _) => { return Ok(Self::DrawSprite) },
            (0xE, _) => {
                if lower_byte == 0x9E { return Ok(Self::SkipIfKeyPressed) }
                if lower_byte == 0xA1 { return Ok(Self::SkipIfKeyNotPressed) }
            },
            (0xF, _) => {
                match lower_byte {
                    0x07 => { return Ok(Self::ReadDelayRegister) },
                    0x0A => { return Ok(Self::WaitForKeyPress) },
                    0x15 => { return Ok(Self::SetDelayRegister) },
                    0x18 => { return Ok(Self::SetSoundRegister) },
                    0x1E => { return Ok(Self::AddToIndexRegister) },
                    0x29 => { return Ok(Self::SetIndexRegisterToFont) },
                    0x33 => { return Ok(Self::DecodeBCD) },
                    0x55 => { return Ok(Self::SaveRegisters) },
                    0x65 => { return Ok(Self::LoadRegisters) },
                    _ => { }
                }
            },
            _ => { }
        }

        Err(())
    }
}

impl ALUOperation {
    
    fn perform(&self, lhs: u8, rhs: u8) -> (u8, Option<u8>) {
        match self {
            ALUOperation::SetValue => { (rhs, None) },
            ALUOperation::Add => { 
                let sum = lhs as u32 + rhs as u32;

                ((sum & 0xFF) as u8, if sum > 0xFF { Some(1) } else { Some(0) }) },
            ALUOperation::Sub => { (lhs.wrapping_sub(rhs), if lhs >= rhs { Some(1) } else { Some(0) }) },
            ALUOperation::SubAndNegate => { (rhs.wrapping_sub(lhs), if rhs >= lhs { Some(1) } else { Some(0) }) },
            ALUOperation::BitwiseOr => { (lhs | rhs, None) },
            ALUOperation::BitwiseAnd => { (lhs & rhs, None) },
            ALUOperation::BitwiseXor => { (lhs ^ rhs, None) },
            ALUOperation::ShiftLeft => { (lhs << 1, if lhs & 0x80 != 0 { Some(1) } else { Some(0) }) },
            ALUOperation::ShiftRight => { (lhs >> 1, if lhs & 0x01 != 0 { Some(1) } else { Some(0) }) },
        }
    }
}