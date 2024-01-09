pub enum ChipError {
    StackOverflow,
    StackUnderflow,
    InvalidKey(u8),
    UnknownOpcode(u16),
    SysOpcodeNotSupported(u16),
}