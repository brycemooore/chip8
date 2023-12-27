use crate::opcode::Opcode;
use rand::{thread_rng, Rng};

const NUM_KEYS: usize = 16;
const NUM_REGISTERS: usize = 16;
const RAM: usize = 0x1000;
const STACK_SIZE: usize = 16;
pub const DISPLAY_MAX_Y: usize = 32;
pub const DISPLAY_MAX_X: usize = 64;
const FONTSET_SIZE: usize = 80;
const START_ADDR: usize = 0x200;
const SPRITE_WIDTH: usize = 8;
const FONTSET_ADDR: usize = 0x50;

const FONTSET: [u8; FONTSET_SIZE] = [
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

#[derive(Debug)]
pub struct Chip8 {
    //16 registers
    registers: [u8; NUM_REGISTERS],
    //use usize for easy indexing
    //program counter
    position_in_memory: usize,
    //4096 bytes = 4 kb
    memory: [u8; RAM],
    //stack is 16 levels deep
    //after 16 nested function calls
    //program encounters stack overflow
    stack: [u16; STACK_SIZE],
    //use usize for easy indexing
    stack_pointer: usize,
    i_register: u16,
    delay_timer_register: u8,
    sound_timer_register: u8,
    keys: [bool; NUM_KEYS],
    display: [bool; DISPLAY_MAX_X * DISPLAY_MAX_Y],
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip = Chip8 {
            position_in_memory: 0,
            registers: [0; NUM_REGISTERS],
            memory: [0; RAM],
            stack: [0; STACK_SIZE],
            stack_pointer: 0,
            i_register: 0,
            delay_timer_register: 0,
            sound_timer_register: 0,
            keys: [false; NUM_KEYS],
            display: [false; DISPLAY_MAX_X * DISPLAY_MAX_Y],
        };
        //load fontset
        chip.memory[FONTSET_ADDR..=FONTSET_ADDR + FONTSET_SIZE - 1].copy_from_slice(&FONTSET);
        //start addr
        chip.position_in_memory = START_ADDR;
        chip
    }

    pub fn new_with_rom(rom: &[u8]) -> Self {
        let mut chip = Chip8::new();
        chip.load_rom(rom);
        chip
    }

    pub fn get_display(&self) -> &[bool; DISPLAY_MAX_X * DISPLAY_MAX_Y] {
        &self.display
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        self.memory[START_ADDR..=START_ADDR + rom.len() - 1].copy_from_slice(rom);
    }

    pub fn tick(&mut self) {
        let opcode = self.fetch();
        self.execute(opcode);
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer_register > 0 {
            self.delay_timer_register -= 1;
        }

        if self.sound_timer_register > 0 {
            self.sound_timer_register -= 1;
        }
    }

    pub fn key_press(&mut self, key: u8) {
        //runtime check here, responsibilty falls on keyboard provider
        //should never happen
        if key > 15 {
            panic!("Invalid Key Provided, key must be hexadecmal value of 0x0 through 0xF");
        }

        self.keys[key as usize] = true;
    }

    pub fn key_release(&mut self, key: u8) {
        //runtime check here, responsibilty falls on keyboard provider
        //should never happen
        if key > 15 {
            panic!("Invalid Key Provided, key must be hexadecmal value of 0x0 through 0xF");
        }
        self.keys[key as usize] = false;
    }

    fn fetch(&mut self) -> u16 {
        let p = self.position_in_memory;
        //byte at p
        let op_byte1 = self.memory[p] as u16;
        //byte at p + 1
        let op_byte2 = self.memory[p + 1] as u16;
        //shift position in memory by 2 bytes
        self.position_in_memory += 2;
        //combine into a single 16 bit opcode
        op_byte1 << 8 | op_byte2
    }

    fn execute(&mut self, opcode: u16) {
        match Opcode::decode(opcode) {
            Opcode::Sys(_) => panic!("Syscall not supported"),
            Opcode::Jump(nnn) => self.jump(nnn),
            Opcode::JumpPlusV0(nnn) => self.jump_plus_v0(nnn),
            Opcode::Call(nnn) => self.call(nnn),
            Opcode::SkipIfEqualAtX { x, kk } => self.skip_if_equal_at_x(x, kk),
            Opcode::SkipIfNotEqualAtX { x, kk } => self.skip_if_not_equal_at_x(x, kk),
            Opcode::LoadValueToRegister { x, kk } => self.load_value_to_register(x, kk),
            Opcode::AddToValueInRegister { x, kk } => self.add_to_value_in_register(x, kk),
            Opcode::SkipIfBothValuesEqual { x, y } => self.skip_if_both_values_are_equal(x, y),
            Opcode::SkipIfBothValuesNotEqual { x, y } => {
                self.skip_if_both_values_are_not_equal(x, y)
            }
            Opcode::LoadYIntoX { x, y } => self.load_y_into_x(x, y),
            Opcode::BitwiseOrXY { x, y } => self.bitwise_or_xy(x, y),
            Opcode::BitwiseAndXY { x, y } => self.bitwise_and_xy(x, y),
            Opcode::BitwiseXorXY { x, y } => self.bitwise_xor_xy(x, y),
            Opcode::AddXY { x, y } => self.add_xy(x, y),
            Opcode::SubYfromX { x, y } => self.sub_y_from_x(x, y),
            Opcode::SubXfromY { x, y } => self.sub_x_from_y(x, y),
            Opcode::Ret => self.ret(),
            Opcode::ShiftRight { x, y } => self.shift_right(x, y),
            Opcode::ShiftLeft { x, y } => self.shift_left(x, y),
            Opcode::SetIRegister(nnn) => self.set_i_register(nnn),
            Opcode::RandomNumberToRegisterX { x, kk } => self.random_number_to_x(x, kk),
            Opcode::LoadDelayTimerToVx { x } => self.load_delay_timer_to_vx(x),
            Opcode::SetDelayTimer { x } => self.set_delay_timer(x),
            Opcode::SetSoundTimer { x } => self.set_sound_timer(x),
            Opcode::AddVxToIRegister { x } => self.add_vx_to_i_register(x),
            Opcode::LoadVxAsDecimalIntoMemoryAtIRegister { x } => {
                self.load_vx_as_decimal_into_memory_at_i(x)
            }
            Opcode::LoadRegistersV0ToVxIntoMemoryAtI { x } => {
                self.load_registers_v0_to_vx_into_memory_at_i(x)
            }
            Opcode::FillRegistersV0ToVxFromMmoryAtI { x } => {
                self.fill_registers_v0_to_vx_from_memory_at_i(x)
            }
            Opcode::WaitForKeyPressAndStoreVx { x } => self.wait_for_keypress_store_vx(x),
            Opcode::SkipIfKeyAtVxPressed { x } => self.skip_if_key_at_vx_pressed(x),
            Opcode::SkipIfKeyAtVxNotPressed { x } => self.skip_if_key_at_vx_not_pressed(x),
            Opcode::Draw { x, y, n } => self.draw(x, y, n),
            Opcode::ClearScreen => self.clear_screen(),
            Opcode::SetICorrespondingFontAddressFromVx { x } => {
                self.set_font_address_for_value_in_vx(x)
            }
            Opcode::UnknownOpcode(op) => todo!("opcode {:04x}", op),
        }
    }

    //2nnn - CALL addr
    fn call(&mut self, addr: u16) {
        let sp = self.stack_pointer;
        let stack = &mut self.stack;

        if sp >= stack.len() {
            panic!("Stack overflow");
        }

        //store current position in memory in stack
        stack[sp] = self.position_in_memory as u16;
        //increment stack pointer
        self.stack_pointer += 1;
        //set position in memory to addr provided
        self.position_in_memory = addr as usize;
    }

    //00EE - RET
    fn ret(&mut self) {
        if self.stack_pointer == 0 {
            panic!("Stack underflow");
        }

        //decrement stack pointer
        self.stack_pointer -= 1;
        //get position in memory from stack
        let call_addr = self.stack[self.stack_pointer];
        //set position in memory to call_addr
        self.position_in_memory = call_addr as usize;
    }

    //1nnn - JP addr
    fn jump(&mut self, addr: u16) {
        self.position_in_memory = addr as usize;
    }

    //3xkk - SE Vx, byte
    fn skip_if_equal_at_x(&mut self, x: u8, kk: u8) {
        let vx = self.registers[x as usize];
        if vx == kk {
            self.position_in_memory += 2;
        }
    }

    //4xkk SNE Vx, byte
    fn skip_if_not_equal_at_x(&mut self, x: u8, kk: u8) {
        let vx = self.registers[x as usize];
        if vx != kk {
            self.position_in_memory += 2;
        }
    }

    //5xy0 SE Vx, Vy
    fn skip_if_both_values_are_equal(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];
        if vx == vy {
            self.position_in_memory += 2;
        }
    }

    //6xkk LD Vx, byte
    fn load_value_to_register(&mut self, x: u8, kk: u8) {
        self.registers[x as usize] = kk;
    }

    //7xkk ADD Vx, byte
    fn add_to_value_in_register(&mut self, x: u8, kk: u8) {
        let vx = self.registers[x as usize];
        self.registers[x as usize] = vx.wrapping_add(kk);
    }

    //8xy0 LD Vx, Vy
    fn load_y_into_x(&mut self, x: u8, y: u8) {
        let vy = self.registers[y as usize];
        self.registers[x as usize] = vy;
    }

    //8xy1 OR Vx, Vy
    fn bitwise_or_xy(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];
        self.registers[x as usize] = vx | vy;
    }

    //8xy2 AND Vx, Vy
    fn bitwise_and_xy(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];
        self.registers[x as usize] = vx & vy;
    }

    //8xy3 XOR Vx, Vy
    fn bitwise_xor_xy(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];
        self.registers[x as usize] = vx ^ vy;
    }

    //8xy4 - ADD Vx, Vy
    fn add_xy(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];
        let (sum, carry) = vx.overflowing_add(vy);
        self.registers[x as usize] = sum;
        self.set_vf(carry);
    }

    //8xy5 SUB Vx, Vy
    fn sub_y_from_x(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];
        self.registers[x as usize] = vx.wrapping_sub(vy);
        self.set_vf(vx > vy);
    }

    //8xy7 SUBN Vx, Vy
    fn sub_x_from_y(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];
        self.registers[x as usize] = vy.wrapping_sub(vx);
        self.set_vf(vy > vx);
    }

    //8xy6 SHR Vx {, Vy}
    fn shift_right(&mut self, x: u8, y: u8) {
        let vy = self.registers[y as usize];

        self.registers[x as usize] = vy >> 1;
        //lsb in VF
        self.registers[0xF] = vy & 1
    }

    //8xyE - SHL Vx {, Vy}
    fn shift_left(&mut self, x: u8, y: u8) {
        let vy = self.registers[y as usize];
        self.registers[x as usize] = vy << 1;
        //shift last bit 7 and mask 1 to get msb
        self.registers[0xF] = vy >> 7 & 1;
    }

    //9xy0 - SNE Vx, Vy
    fn skip_if_both_values_are_not_equal(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];
        if vx != vy {
            self.position_in_memory += 2;
        }
    }

    //Annn - LD I, addr
    fn set_i_register(&mut self, nnn: u16) {
        self.i_register = nnn;
    }

    //Bnnn - JP V0, addr
    fn jump_plus_v0(&mut self, nnn: u16) {
        let v0 = self.registers[0];
        self.jump(nnn + (v0 as u16));
    }

    //Cxkk - RND Vx, byte
    fn random_number_to_x(&mut self, x: u8, kk: u8) {
        let random_number: u8 = thread_rng().gen_range(0..=255);
        self.registers[x as usize] = random_number & kk;
    }

    //Fx07 LD Vx, DT
    fn load_delay_timer_to_vx(&mut self, x: u8) {
        self.registers[x as usize] = self.delay_timer_register;
    }

    //Fx15 LD DT, Vx
    fn set_delay_timer(&mut self, x: u8) {
        let vx = self.registers[x as usize];
        self.delay_timer_register = vx;
    }

    //Fx18 LD ST, Vx
    fn set_sound_timer(&mut self, x: u8) {
        let vx = self.registers[x as usize];
        self.sound_timer_register = vx;
    }

    //Fx1E ADD I, Vx
    fn add_vx_to_i_register(&mut self, x: u8) {
        let vx = self.registers[x as usize];
        self.i_register = self.i_register.wrapping_add(vx as u16);
    }

    //Fx33 LD B, Vx
    fn load_vx_as_decimal_into_memory_at_i(&mut self, x: u8) {
        let vx = self.registers[x as usize];
        let hundreds = vx / 100;
        let tens = (vx % 100) / 10;
        let ones = vx % 10;
        let i = self.i_register as usize;
        self.memory[i] = hundreds;
        self.memory[i + 1] = tens;
        self.memory[i + 2] = ones;
    }

    //Fx55 LD [I], Vx
    fn load_registers_v0_to_vx_into_memory_at_i(&mut self, x: u8) {
        let mut i = self.i_register as usize;
        for r in 0..=x {
            let vr = self.registers[r as usize];
            self.memory[i] = vr;
            i += 1;
        }
        // I is set to I + X + 1
        self.i_register += x as u16 + 1;
    }

    //Fx65 LD Vx, [I]
    fn fill_registers_v0_to_vx_from_memory_at_i(&mut self, x: u8) {
        for r in 0..=x {
            self.registers[r as usize] = self.memory[(self.i_register + r as u16) as usize];
        }
        self.i_register += x as u16 + 1;
    }

    //Fx0A LD Vx, K
    fn wait_for_keypress_store_vx(&mut self, x: u8) {
        let mut pressed = false;
        //loop through keys, if any true, a key is pressed
        for (i, key) in self.keys.iter().enumerate() {
            if *key {
                //set register to i
                self.registers[x as usize] = i as u8;
                pressed = true;
            }
        }

        //redo opcode
        if !pressed {
            self.position_in_memory -= 2;
        }
    }

    //Ex9E SKP Vx
    fn skip_if_key_at_vx_pressed(&mut self, x: u8) {
        let vx = self.registers[x as usize];
        if self.keys[vx as usize] {
            self.position_in_memory += 2;
        }
    }

    //ExA1 SKP Vx
    fn skip_if_key_at_vx_not_pressed(&mut self, x: u8) {
        let vx = self.registers[x as usize];
        if !(self.keys[vx as usize]) {
            self.position_in_memory += 2;
        }
    }

    fn draw(&mut self, x: u8, y: u8, n: u8) {
        let x_coord = self.registers[x as usize];
        let y_coord = self.registers[y as usize];
        let mut flipped = false;
        //n is num bytes at i for sprite
        //Each byte represents one line of the sprite top down
        //loop through each of the bytes at I gives us our y axis
        for y_line in 0..n {
            let addr = self.i_register + y_line as u16;
            let pixels = self.memory[addr as usize];

            //each bit in the byte represent the 8 bits that make up x axis
            //sprites are always 8
            for x_line in 0..SPRITE_WIDTH {
                //this shifts through each bit, and checks if it needs to flip
                //only flips when it is 1
                if pixels & (0b10000000 >> x_line) != 0 {
                    // Sprites should wrap around screen, so apply modulo
                    let x = (x_coord + x_line as u8) as usize % DISPLAY_MAX_X;
                    let y = (y_coord + y_line) as usize % DISPLAY_MAX_Y;

                    // Get our pixel's index in the 1D screen array
                    //index = y * width + x
                    let idx = y * DISPLAY_MAX_X + x;
                    // If a pixel on the screen is set to 01,
                    //and the sprite to be drawn contains a 01 for this same pixel,
                    //the screen pixel is turned off and VF is set to 01.
                    //If the sprite is simply drawn on the screen without drawing over any pixels set to 01,
                    //VF is set to 00
                    //boolean |= will only stay false if all of the display spots are false
                    flipped |= self.display[idx];
                    //xor with true, since we only do this when the sprite wants a pixel drawn
                    self.display[idx] ^= true;
                }
            }
        }

        // Populate VF register
        if flipped {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }

    //00E0
    fn clear_screen(&mut self) {
        self.display = [false; DISPLAY_MAX_X * DISPLAY_MAX_Y];
    }

    //Fx29
    fn set_font_address_for_value_in_vx(&mut self, x: u8) {
        let vx = self.registers[x as usize];
        self.i_register = FONTSET_ADDR as u16 + vx as u16 * 5;
    }

    fn set_vf(&mut self, set_to_one: bool) {
        if set_to_one {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        thread,
        time::Duration,
    };

    use super::*;

    #[test]
    fn test_tick() {
        let mut chip = Chip8::new();

        //test load jump command and execute.
        //jump to 400
        chip.memory[0x300] = 0x14;
        chip.memory[0x301] = 0x00;
        chip.position_in_memory = 0x300;
        chip.tick();
        assert_eq!(0x400, chip.position_in_memory);
    }

    #[test]
    fn test_tick_timers() {
        let mut chip = Chip8::new();

        chip.delay_timer_register = 2;
        chip.sound_timer_register = 1;

        chip.tick_timers();

        assert_eq!(1, chip.delay_timer_register);
        assert_eq!(0, chip.sound_timer_register);

        chip.tick_timers();

        assert_eq!(0, chip.delay_timer_register);
        assert_eq!(0, chip.sound_timer_register);
    }

    #[test]
    fn test_fetch() {
        let mut chip8 = Chip8::new();
        chip8.memory[START_ADDR] = 0x12;
        chip8.memory[START_ADDR + 1] = 0x34;
        assert_eq!(chip8.fetch(), 0x1234);
        assert_eq!(chip8.position_in_memory, START_ADDR + 2);
    }

    #[test]
    fn test_call() {
        let mut chip8 = Chip8::new();
        chip8.position_in_memory = 0x200;
        chip8.stack_pointer = 0;
        chip8.call(0x300);
        assert_eq!(chip8.stack_pointer, 1);
        assert_eq!(chip8.stack[0], 0x200);
        assert_eq!(chip8.position_in_memory, 0x300);
    }

    #[test]
    #[should_panic(expected = "Stack overflow")]
    fn test_call_overflow() {
        let mut chip8 = Chip8::new();
        chip8.stack_pointer = 16;
        chip8.call(0x300);
    }

    #[test]
    fn test_ret() {
        let mut chip8 = Chip8::new();
        chip8.stack_pointer = 1;
        chip8.stack[0] = 0x300;
        chip8.ret();
        assert_eq!(chip8.stack_pointer, 0);
        assert_eq!(chip8.position_in_memory, 0x300);
    }

    #[test]
    #[should_panic(expected = "Stack underflow")]
    fn test_ret_underflow() {
        let mut chip8 = Chip8::new();
        chip8.ret();
    }

    #[test]
    fn test_jump() {
        let mut chip8 = Chip8::new();
        chip8.jump(0x300);
        assert_eq!(chip8.position_in_memory, 0x300);
    }

    #[test]
    fn test_jump_plus_v0() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 0x21;
        chip8.jump_plus_v0(0x300);
        assert_eq!(chip8.position_in_memory, 0x321);
    }

    #[test]
    fn test_skip_if_equal_at_x() {
        //position in memory is 0 on init
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 0x12;
        //don't call tick, call method manually
        chip8.skip_if_equal_at_x(0, 0x12);
        //position in memory should be incremented by 2
        assert_eq!(chip8.position_in_memory, START_ADDR + 2);
    }

    #[test]
    fn test_skip_if_not_equal_at_x() {
        //position in memory is 0 on init
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 0x12;
        //don't call tick, call method manually
        chip8.skip_if_not_equal_at_x(0, 0x13);
        //position in memory should be incremented by 2
        assert_eq!(chip8.position_in_memory, START_ADDR + 2);
    }

    #[test]
    fn test_skip_if_both_values_are_equal() {
        //position in memory is 0 on init
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 0x12;
        chip8.registers[1] = 0x12;
        //don't call tick, call method manually
        chip8.skip_if_both_values_are_equal(0, 1);
        //position in memory should be incremented by 2
        assert_eq!(chip8.position_in_memory, START_ADDR + 2);
    }

    #[test]
    fn test_load_value_to_register() {
        let mut chip8 = Chip8::new();
        chip8.load_value_to_register(0, 0x12);
        assert_eq!(chip8.registers[0], 0x12);
    }

    #[test]
    fn test_add_to_value_in_register() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 0x12;
        chip8.add_to_value_in_register(0, 0x12);
        assert_eq!(chip8.registers[0], 0x24);
    }

    #[test]
    fn test_load_y_into_x() {
        let mut chip8 = Chip8::new();
        chip8.registers[1] = 0x12;
        chip8.load_y_into_x(0, 1);
        assert_eq!(chip8.registers[0], 0x12);
    }

    #[test]
    fn test_bitwise_or_xy() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 0b1010;
        chip8.registers[1] = 0b1100;
        chip8.bitwise_or_xy(0, 1);
        assert_eq!(chip8.registers[0], 0b1110);
    }

    #[test]
    fn test_bitwise_and_xy() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 0b1010;
        chip8.registers[1] = 0b1100;
        chip8.bitwise_and_xy(0, 1);
        assert_eq!(chip8.registers[0], 0b1000);
    }

    #[test]
    fn test_bitwise_xor_xy() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 0b1010;
        chip8.registers[1] = 0b1100;
        chip8.bitwise_xor_xy(0, 1);
        assert_eq!(chip8.registers[0], 0b0110);
    }

    #[test]
    fn test_add_xy() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 5;
        chip8.registers[1] = 10;
        chip8.add_xy(0, 1);
        assert_eq!(chip8.registers[0], 15);
        assert_eq!(chip8.registers[0xF], 0);
    }

    #[test]
    fn test_add_xy_no_overflow() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 5;
        chip8.registers[1] = 10;
        chip8.add_xy(0, 1);
        assert_eq!(chip8.registers[0], 15);
        assert_eq!(chip8.registers[0xF], 0);
    }

    #[test]
    fn test_add_xy_with_overflow() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 200;
        chip8.registers[1] = 100;
        chip8.add_xy(0, 1);
        assert_eq!(chip8.registers[0], 44); // 300 % 256 = 44
        assert_eq!(chip8.registers[0xF], 1);
    }

    #[test]
    fn test_sub_y_from_x_no_borrow() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 10;
        chip8.registers[1] = 5;
        chip8.sub_y_from_x(0, 1);
        assert_eq!(chip8.registers[0], 5);
        assert_eq!(chip8.registers[0xF], 1);
    }

    #[test]
    fn test_sub_y_from_x_with_borrow() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 5;
        chip8.registers[1] = 10;
        chip8.sub_y_from_x(0, 1);
        assert_eq!(chip8.registers[0], 251); // 5 - 10 = -5, which wraps around to 251 in unsigned 8-bit arithmetic
        assert_eq!(chip8.registers[0xF], 0);
    }

    #[test]
    fn test_shift_right_lsb_1() {
        let mut chip8 = Chip8::new();
        chip8.registers[1] = 0b00001001;
        chip8.shift_right(0, 1);
        assert_eq!(1, chip8.registers[0xF]);
        assert_eq!(0b00000100, chip8.registers[0]);
    }

    #[test]
    fn test_shift_right_lsb_0() {
        let mut chip8 = Chip8::new();
        chip8.registers[1] = 0b00000110;
        chip8.shift_right(0, 1);
        assert_eq!(0, chip8.registers[0xF]);
        assert_eq!(0b00000011, chip8.registers[0]);
    }

    #[test]
    fn test_sub_x_from_y_no_borrow() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 5;
        chip8.registers[1] = 10;
        chip8.sub_x_from_y(0, 1);
        assert_eq!(chip8.registers[0], 5);
        assert_eq!(chip8.registers[0xF], 1);
    }

    #[test]
    fn test_sub_x_from_y_with_borrow() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 10;
        chip8.registers[1] = 5;
        chip8.sub_x_from_y(0, 1);
        assert_eq!(chip8.registers[0], 251); // 5 - 10 = -5, which wraps around to 251 in unsigned 8-bit arithmetic
        assert_eq!(chip8.registers[0xF], 0);
    }

    #[test]
    fn test_shift_left_msb_1() {
        let mut chip8 = Chip8::new();
        chip8.registers[1] = 0b10010000;
        chip8.shift_left(0, 1);
        assert_eq!(1, chip8.registers[0xF]);
        assert_eq!(0b00100000, chip8.registers[0]);
    }

    #[test]
    fn test_shift_left_msb_0() {
        let mut chip8 = Chip8::new();
        chip8.registers[1] = 0b01100000;
        chip8.shift_left(0, 1);
        assert_eq!(0, chip8.registers[0xF]);
        assert_eq!(0b11000000, chip8.registers[0]);
    }

    #[test]
    fn test_skip_if_both_values_are_not_equal() {
        //position in memory is 0 on init
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 0x12;
        chip8.registers[1] = 0x13;
        //don't call tick, call method manually
        chip8.skip_if_both_values_are_not_equal(0, 1);
        //position in memory should be incremented by 2
        assert_eq!(chip8.position_in_memory, START_ADDR + 2);
    }

    #[test]
    fn test_set_i_register() {
        let mut chip8 = Chip8::new();
        chip8.set_i_register(1);
        assert_eq!(1, chip8.i_register);
    }

    #[test]
    fn test_load_delay_timer_to_vx() {
        let mut chip8 = Chip8::new();
        chip8.delay_timer_register = 1;
        assert_eq!(0, chip8.registers[0]);
        chip8.load_delay_timer_to_vx(0);
        assert_eq!(1, chip8.registers[0]);
    }

    #[test]
    fn test_set_delay_timer() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 1;
        chip8.set_delay_timer(0);
        assert_eq!(1, chip8.delay_timer_register);
    }

    #[test]
    fn test_set_sound_timer() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 1;
        chip8.set_sound_timer(0);
        assert_eq!(1, chip8.sound_timer_register);
    }

    #[test]
    fn test_add_vx_to_i_register() {
        let mut chip = Chip8::new();
        chip.registers[0] = 1;
        chip.i_register = 1;
        assert_eq!(1, chip.i_register);
        chip.add_vx_to_i_register(0);
        assert_eq!(2, chip.i_register);
    }

    #[test]
    fn test_load_vx_as_decimal_into_memory_at_i() {
        let mut chip1 = Chip8::new();
        let mut chip2 = Chip8::new();
        let mut chip3 = Chip8::new();

        chip1.registers[0] = 246;
        chip2.registers[0] = 82;
        chip3.registers[0] = 2;

        chip1.i_register = 0x300;
        chip2.i_register = 0x300;
        chip3.i_register = 0x300;

        chip1.load_vx_as_decimal_into_memory_at_i(0);
        chip2.load_vx_as_decimal_into_memory_at_i(0);
        chip3.load_vx_as_decimal_into_memory_at_i(0);

        assert_eq!(2, chip1.memory[0x300]);
        assert_eq!(4, chip1.memory[0x301]);
        assert_eq!(6, chip1.memory[0x302]);

        assert_eq!(0, chip2.memory[0x300]);
        assert_eq!(8, chip2.memory[0x301]);
        assert_eq!(2, chip2.memory[0x302]);

        assert_eq!(0, chip3.memory[0x300]);
        assert_eq!(0, chip3.memory[0x301]);
        assert_eq!(2, chip3.memory[0x302]);
    }

    #[test]
    fn test_load_v0_to_vx_into_memory_at_i_all_registers() {
        let mut chip = Chip8::new();
        chip.i_register = 0x300;
        for (i, register) in chip.registers.iter_mut().enumerate() {
            *register = i as u8;
        }

        chip.load_registers_v0_to_vx_into_memory_at_i(0xF);

        let mut memory_counter = 0x300;
        //test all 15 registers
        for i in 0x0..=0xF {
            assert_eq!(i, chip.memory[memory_counter]);
            memory_counter += 1;
        }

        //make sure i is squared away
        assert_eq!(0x310, chip.i_register);
    }

    #[test]
    fn test_load_v0_to_vx_into_memory_at_i_some_registers() {
        let mut chip = Chip8::new();
        chip.i_register = 0x300;
        chip.registers[0] = 1;
        chip.registers[1] = 2;
        chip.registers[2] = 0;
        chip.registers[3] = 4;

        chip.load_registers_v0_to_vx_into_memory_at_i(0x3);

        assert_eq!(1, chip.memory[0x300]);
        assert_eq!(2, chip.memory[0x301]);
        assert_eq!(0, chip.memory[0x302]);
        assert_eq!(4, chip.memory[0x303]);
        assert_eq!(0, chip.memory[0x304]);

        //make sure i is squared away
        assert_eq!(0x304, chip.i_register);
    }

    #[test]
    fn test_fill_registers_v0_to_vx_from_memory_at_i_all_registers() {
        let mut chip = Chip8::new();
        chip.i_register = 0x300;

        for i in 0x0..=0xF {
            chip.memory[0x300 + i] = i as u8 + 1;
        }

        chip.fill_registers_v0_to_vx_from_memory_at_i(0xF);

        for i in 0x0..=0xF {
            assert_eq!(i as u8 + 1, chip.registers[i]);
        }

        assert_eq!(0x310, chip.i_register);
    }

    #[test]
    fn test_fill_registers_v0_to_vx_from_memory_at_i_some_registers() {
        let mut chip = Chip8::new();
        chip.i_register = 0x300;

        chip.memory[0x300] = 12;
        chip.memory[0x301] = 6;
        chip.memory[0x302] = 1;

        chip.fill_registers_v0_to_vx_from_memory_at_i(0x2);

        assert_eq!(12, chip.registers[0]);
        assert_eq!(6, chip.registers[1]);
        assert_eq!(1, chip.registers[2]);
        assert_eq!(0, chip.registers[3]);

        assert_eq!(0x303, chip.i_register);
    }

    #[test]
    fn test_wait_for_key_press_and_store_at_vx() {
        let mut chip = Chip8::new();
        chip.position_in_memory = 0x300;
        //press key
        chip.key_press(0xF);
        //wait for keypress
        chip.wait_for_keypress_store_vx(0);
        assert_eq!(0xF, chip.registers[0]);
        //release key
        chip.key_release(0xF);

        //check again no key is pressed
        chip.wait_for_keypress_store_vx(0);
        //program counter drops back 2 spots to re run the wait opcode
        assert_eq!(0x2FE, chip.position_in_memory);
    }

    #[test]
    fn test_wait_for_key_press_and_store_at_vx_threaded() {
        let mut chip = Chip8::new();
        //progam to wait for key and store in V0
        let program = [0xF0, 0x0A];
        //put program in RAM
        chip.memory[0x300..0x302].copy_from_slice(&program);
        //program counter to start of program
        chip.position_in_memory = 0x300;

        //thread safe for multiple references and interior mutability
        let chip_arc = Arc::new(Mutex::new(chip));

        //clone here so chip_arc doesn't move into closure
        let chip_clone = Arc::clone(&chip_arc);
        //thread to wait for key press
        thread::spawn(move || {
            //get chip reference
            let mut chip = chip_clone.lock().unwrap();
            //while we haven't gotten past first opcode, tick
            while chip.position_in_memory < 0x302 {
                //fetch and execute opcode
                //if no key is pressed the program counter will move back 2,
                //to retry until there is a key pressed
                chip.tick();
            }
        });

        //press key
        chip_arc.lock().unwrap().key_press(0xF);
        //wait for thread to wrap up
        thread::sleep(Duration::from_millis(500));

        //Only one reference to chip out in the world
        assert_eq!(1, Arc::strong_count(&chip_arc));
        //the key is currectly stored in register 0
        assert_eq!(0xF, chip_arc.lock().unwrap().registers[0]);
    }

    #[test]
    fn test_skip_if_key_pressed_at_vx() {
        let mut chip = Chip8::new();

        chip.registers[0] = 0xF;
        chip.position_in_memory = 0x300;

        //press key in V0
        chip.key_press(0xF);
        //run skip operation
        chip.skip_if_key_at_vx_pressed(0);
        //program counter move two places
        assert_eq!(0x302, chip.position_in_memory);
        chip.key_release(0xF);

        //press wrong key
        chip.key_press(0xA);
        //run skip
        chip.skip_if_key_at_vx_pressed(0);
        //should not have moved
        assert_eq!(0x302, chip.position_in_memory);
    }

    #[test]
    fn test_skip_if_key_not_pressed_at_vx() {
        let mut chip = Chip8::new();

        chip.registers[0] = 0xF;
        chip.position_in_memory = 0x300;

        //press key in V0
        chip.key_press(0xF);
        //run skip operation
        chip.skip_if_key_at_vx_not_pressed(0);
        //should not skip
        assert_eq!(0x300, chip.position_in_memory);
        chip.key_release(0xF);

        //press wrong key
        chip.key_press(0xA);
        //run skip
        chip.skip_if_key_at_vx_not_pressed(0);
        //should skip
        assert_eq!(0x302, chip.position_in_memory);
    }

    #[test]
    fn test_clear_screen() {
        let mut chip = Chip8::new();

        //font start
        chip.i_register = FONTSET_ADDR as u16;

        //draw first letter in font set 5 bytes at
        chip.draw(0, 0, 5);

        assert!(chip.display.contains(&true));

        chip.clear_screen();

        assert!(!(chip.display.contains(&true)));
    }

    #[test]
    fn test_set_font_address_for_value_in_vx() {
        let mut chip = Chip8::new();

        let mut count = 0;
        for hex in 0x0..=0xF {
            chip.registers[0] = hex;

            chip.set_font_address_for_value_in_vx(0);

            assert_eq!(FONTSET_ADDR as u16 + 5 * count, chip.i_register);
            count += 1;
        }
    }

    #[test]
    fn test_draw() {
        let mut chip = Chip8::new();

        //font start
        chip.i_register = 5 * 0xF + FONTSET_ADDR as u16;

        //draw first letter in font set 5 bytes at
        chip.draw(0, 0, 5);

        for y in 0..5 {
            for x in 0..8 {
                let index = y * DISPLAY_MAX_X + x;
                if chip.display[index] {
                    print!("*");
                } else {
                    print!(" ");
                }
            }
            print!("\n");
        }
    }
}
