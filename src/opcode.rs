
#[derive(Debug, PartialEq)]
pub enum Opcode {
    Sys(u16),
    Jump(u16),
    Call(u16),
    SkipIfEqualAtX { x: u8, kk: u8 },
    SkipIfNotEqualAtX { x: u8, kk: u8 },
    LoadValueToRegister { x: u8, kk: u8 },
    AddToValueInRegister { x: u8, kk: u8 },
    SkipIfBothValuesEqual { x: u8, y: u8 },
    LoadYIntoX { x: u8, y: u8 },
    BitwiseOrXY { x: u8, y: u8 },
    BitwiseAndXY { x: u8, y: u8 },
    BitwiseXorXY {x: u8, y: u8},
    AddXY { x: u8, y: u8 },
    SubXfromY { x: u8, y: u8 },
    SubYfromX {x: u8, y: u8},
    Ret,
    ShiftRight{ x: u8 },
    ShiftLeft{ x: u8 },
    SkipIfBothValuesNotEqual{ x:  u8, y: u8 },
    SetIRegister(u16),
    JumpPlusV0(u16),
    RandomNumberToRegisterX{ x: u8, kk: u8 },
    LoadDelayTimerToVx{ x: u8 },
    SetDelayTimer{ x: u8 },
    SetSoundTimer{ x: u8 },
    AddVxToIRegister{ x: u8 },
    LoadVxAsDecimalIntoMemoryAtIRegister{ x: u8 },
    UnknownOpcode(u16),
}

impl Opcode {
    pub fn decode(opcode: u16) -> Self {
        let c = ((opcode & 0xF000) >> 12) as u8;
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let y = ((opcode & 0x00F0) >> 4) as u8;
        let d = (opcode & 0x000F) as u8;
        let nnn = opcode & 0x0FFF;
        let kk = (opcode & 0x00FF) as u8;

        match (c, x, y, d) {
            (0x1, _, _, _) => Opcode::Jump(nnn),
            (0x2, _, _, _) => Opcode::Call(nnn),
            (0x3, x, _, _) => Opcode::SkipIfEqualAtX { x, kk },
            (0x4, x, _, _) => Opcode::SkipIfNotEqualAtX { x, kk },
            (0x5, x, y, 0x0) => Opcode::SkipIfBothValuesEqual { x, y },
            (0x6, x, _, _) => Opcode::LoadValueToRegister { x, kk },
            (0x7, x, _, _) => Opcode::AddToValueInRegister { x, kk },
            (0x8, x, y, 0x0) => Opcode::LoadYIntoX { x, y },
            (0x8, x, y, 0x1) => Opcode::BitwiseOrXY { x, y },
            (0x8, x, y, 0x2) => Opcode::BitwiseAndXY { x, y },
            (0x8, x, y, 0x3) => Opcode::BitwiseXorXY {x, y},
            (0x8, x, y, 0x4) => Opcode::AddXY { x, y },
            (0x8, x, y, 0x5) => Opcode::SubXfromY { x, y },
            (0x8, x, _, 0x6) => Opcode::ShiftRight { x },
            (0x8, x, y, 0x7) => Opcode::SubYfromX { x, y },
            (0x8, x, _, 0xE) => Opcode::ShiftLeft { x }, 
            (0x9, x, y, 0x0) => Opcode::SkipIfBothValuesNotEqual { x, y },
            (0xA, _ , _, _) => Opcode::SetIRegister(nnn),
            (0xB, _, _, _) => Opcode::JumpPlusV0(nnn),
            (0xC, x, _ ,_) => Opcode::RandomNumberToRegisterX{ x, kk },
            (0xF, x, 0x0, 0x7) => Opcode::LoadDelayTimerToVx{ x },
            (0xF, x, 0x1, 0x5) => Opcode::SetDelayTimer{ x },
            (0xF, x, 0x1, 0x8) => Opcode::SetSoundTimer{ x },
            (0xF, x, 0x1, 0xE) => Opcode::AddVxToIRegister{ x },
            (0xF, x, 0x3, 0x3) => Opcode::LoadVxAsDecimalIntoMemoryAtIRegister{ x },
            (0x0, 0x0, 0xE, 0xE) => Opcode::Ret,
            (0x0, _, _, _) => Opcode::Sys(nnn),
            _ => Opcode::UnknownOpcode(opcode),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_jump() {
        let opcode = 0x1234;
        assert_eq!(Opcode::decode(opcode), Opcode::Jump(0x234));
    }

    #[test]
    fn test_decode_call() {
        let opcode = 0x2345;
        assert_eq!(Opcode::decode(opcode), Opcode::Call(0x345));
    }

    #[test]
    fn test_decode_skip_if_equal_at_x() {
        let opcode = 0x3ABC;
        assert_eq!(Opcode::decode(opcode), Opcode::SkipIfEqualAtX { x: 0xA, kk: 0xBC });
    }

    #[test]
    fn test_decode_load_value_to_register() {
        let opcode = 0x6ABC;
        assert_eq!(Opcode::decode(opcode), Opcode::LoadValueToRegister { x: 0xA, kk: 0xBC });
    }

    #[test]
    fn test_decode_unknown_opcode() {
        let opcode = 0xFFFF;
        assert_eq!(Opcode::decode(opcode), Opcode::UnknownOpcode(opcode));
    }
}

