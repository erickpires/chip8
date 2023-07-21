
#[derive(Debug)]
pub(crate) enum OperandType {
    Register,
    Immediate
}

#[derive(Debug)]
pub(crate) enum ALUOperation {
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
pub(crate) enum OpCode {
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

pub(crate) struct Instruction {
    pub(crate) op_code: OpCode,
    // These indices are actually half bytes, but using a usize avoids
    // some casting latter, when indexing the register bank.
    pub(crate) x_register_index: usize,
    pub(crate) y_register_index: usize,

    pub(crate) immediate_half_byte: u8,
    pub(crate) immediate_byte: u8,
    pub(crate) immediate_word: u16,
}

impl From<u16> for Instruction {
    fn from(word: u16) -> Self {
        Self { 
            op_code: word.into(), 
            x_register_index: ((word >> 8) & 0xF) as usize,
            y_register_index: ((word >> 4) & 0xF) as usize, 
            immediate_half_byte: (word & 0xF) as u8, 
            immediate_byte: (word & 0xFF) as u8, 
            immediate_word: (word & 0xFFF) 
        }
    }
}

impl From<u16> for OpCode {
    fn from(value: u16) -> Self {
        if value == 0x00E0 {
            return Self::ClearDisplay;
        }

        if value == 0x00EE {
            return Self::ReturnFromSubrotine;
        }
        let higher_nibble = value >> 12;
        let lower_nibble = value & 0xF;
        let lower_byte = value & 0xFF;

        match (higher_nibble, lower_nibble) {
            (0x1, _) => { return Self::JumpAbsolute },
            (0x2, _) => { return Self::CallSubrotine },
            (0x3, _) => { return Self::SkipIfEqual { operand_type: OperandType::Immediate } },
            (0x4, _) => { return Self::SkipIfNotEqual { operand_type: OperandType::Immediate } },
            (0x5, 0x0) => { return Self::SkipIfEqual { operand_type: OperandType::Register } },
            (0x6, _) => { return Self::ArithmeticLogic { operand_type: OperandType::Immediate, operation: ALUOperation::SetValue } },
            (0x7, _) => { return Self::ArithmeticLogic { operand_type: OperandType::Immediate, operation: ALUOperation::Add } },
            (0x8, 0x0) => { return Self::ArithmeticLogic { operand_type: OperandType::Register, operation: ALUOperation::SetValue } },
            (0x8, 0x1) => { return Self::ArithmeticLogic { operand_type: OperandType::Register, operation: ALUOperation::BitwiseOr } },
            (0x8, 0x2) => { return Self::ArithmeticLogic { operand_type: OperandType::Register, operation: ALUOperation::BitwiseAnd } },
            (0x8, 0x3) => { return Self::ArithmeticLogic { operand_type: OperandType::Register, operation: ALUOperation::BitwiseXor } },
            (0x8, 0x4) => { return Self::ArithmeticLogic { operand_type: OperandType::Register, operation: ALUOperation::Add } },
            (0x8, 0x5) => { return Self::ArithmeticLogic { operand_type: OperandType::Register, operation: ALUOperation::Sub } },
            (0x8, 0x6) => { return Self::ArithmeticLogic { operand_type: OperandType::Register, operation: ALUOperation::ShiftRight } },
            (0x8, 0x7) => { return Self::ArithmeticLogic { operand_type: OperandType::Register, operation: ALUOperation::SubAndNegate } },
            (0x8, 0xE) => { return Self::ArithmeticLogic { operand_type: OperandType::Register, operation: ALUOperation::ShiftLeft } },
            (0x9, 0x0) => { return Self::SkipIfNotEqual { operand_type: OperandType::Register } },
            (0xA, _) => { return Self::SetIndexRegister },
            (0xB, _) => { return Self::JumpWithOffset },
            (0xC, _) => { return Self::Rand },
            (0xD, _) => { return Self::DrawSprite },
            (0xE, _) => {
                if lower_byte == 0x9E { return Self::SkipIfKeyPressed }
                if lower_byte == 0xA1 { return Self::SkipIfKeyNotPressed }
            },
            (0xF, _) => {
                match lower_byte {
                    0x07 => { return Self::ReadDelayRegister },
                    0x0A => { return Self::WaitForKeyPress },
                    0x15 => { return Self::SetDelayRegister },
                    0x18 => { return Self::SetSoundRegister },
                    0x1E => { return Self::AddToIndexRegister },
                    0x29 => { return Self::SetIndexRegisterToFont },
                    0x33 => { return Self::DecodeBCD },
                    0x55 => { return Self::SaveRegisters },
                    0x65 => { return Self::LoadRegisters },
                    _ => { }
                }
            },
            _ => { }
        }

        Self::Unknown(value)
    }
}

impl ALUOperation {   
    pub(crate) fn perform(&self, lhs: u8, rhs: u8, compatibility_mode: bool) -> (u8, Option<u8>) {
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
            ALUOperation::ShiftLeft => { 
                let operand = if compatibility_mode { rhs } else { lhs };

                (operand << 1, if operand & 0x80 != 0 { Some(1) } else { Some(0) }) },
            ALUOperation::ShiftRight => {
                let operand = if compatibility_mode { rhs } else { lhs };

                (operand >> 1, if operand & 0x01 != 0 { Some(1) } else { Some(0) }) },
        }
    }
}