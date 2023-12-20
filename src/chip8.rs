use rand::{thread_rng, Rng};
use crate::opcode::Opcode;

const NUM_KEYS: usize = 16;
const NUM_REGISTERS: usize = 16;
const RAM: usize = 0x1000;
const STACK_SIZE: usize = 16;

pub struct Chip8 {
    //16 registers
    pub registers: [u8; NUM_REGISTERS],
    //use usize for easy indexing
    //program counter
    pub position_in_memory: usize,
    //4096 bytes = 4 kb
    pub memory: [u8; RAM],
    //stack is 16 levels deep
    //after 16 nested function calls
    //program encounters stack overflow
    pub stack: [u16; STACK_SIZE],
    //use usize for easy indexing
    pub stack_pointer: usize,
    pub i_register: u16,
    pub delay_timer_register: u8,
    pub sound_timer_register: u8,
    pub keys: [bool; NUM_KEYS],
}

impl Chip8 {

    pub fn new() -> Self {
        Chip8 {
            position_in_memory: 0,
            registers: [0; NUM_REGISTERS],
            memory: [0; RAM],
            stack: [0; STACK_SIZE],
            stack_pointer: 0,
            i_register: 0,
            delay_timer_register: 0,
            sound_timer_register: 0,
            keys: [false; NUM_KEYS],
        }
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

    pub fn execute(&mut self, opcode: u16) {
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
            Opcode::SkipIfBothValuesNotEqual { x, y } => self.skip_if_both_values_are_not_equal(x, y),
            Opcode::LoadYIntoX { x, y } => self.load_y_into_x(x, y),
            Opcode::BitwiseOrXY { x, y } => self.bitwise_or_xy(x, y),
            Opcode::BitwiseAndXY { x, y } => self.bitwise_and_xy(x, y),
            Opcode::BitwiseXorXY { x, y } => self.bitwise_xor_xy(x, y),
            Opcode::AddXY { x, y } => self.add_xy(x, y),
            Opcode::SubYfromX { x, y } => self.sub_y_from_x(x, y),
            Opcode::SubXfromY { x, y } => self.sub_x_from_y(x, y),
            Opcode::Ret => self.ret(),
            Opcode::ShiftRight { x } => self.shift_right(x),
            Opcode::ShiftLeft { x } => self.shift_left(x),
            Opcode::SetIRegister(nnn) => self.set_i_register(nnn),
            Opcode::RandomNumberToRegisterX { x, kk } => self.random_number_to_x(x, kk),
            Opcode::LoadDelayTimerToVx { x } => self.load_delay_timer_to_vx(x),
            Opcode::SetDelayTimer { x } => self.set_delay_timer(x),
            Opcode::SetSoundTimer { x } => self.set_sound_timer(x),
            Opcode::AddVxToIRegister { x } => self.add_vx_to_i_register(x),
            Opcode::LoadVxAsDecimalIntoMemoryAtIRegister { x } => self.load_vx_as_decimal_into_memory_at_i(x),
            Opcode::LoadRegistersV0ToVxIntoMemoryAtI { x } => self.load_registers_v0_to_vx_into_memory_at_i(x),
            Opcode::FillRegistersV0ToVxFromMmoryAtI { x } => self.fill_registers_v0_to_vx_from_memory_at_i(x),
            Opcode::WaitForKeyPressAndStoreVx { x } => self.wait_for_keypress_store_vx(x),
            Opcode::SkipIfKeyAtVxPressed { x } => self.skip_if_key_at_vx_pressed(x),
            Opcode::SkipIfKeyAtVxNotPressed { x } => self.skip_if_key_at_vx_not_pressed(x),
            Opcode::UnknownOpcode(op) => todo!("opcode {:04x}", op),
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
        self.registers[x as usize] += kk;
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
        let (diff, borrow) = vx.overflowing_sub(vy);
        self.registers[x as usize] = diff;
        self.set_vf(borrow);
    }

    //8xy6 SHR Vx {, Vy}
    fn shift_right(&mut self, x: u8) {
        let vx = self.registers[x as usize];

        self.registers[x as usize] = vx >> 1;
        //lsb in VF
        self.registers[0xF] = vx & 1
    }

    //8xy7 SUBN Vx, Vy
    fn sub_x_from_y(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];
        let (diff, borrow) = vy.overflowing_sub(vx);
        self.registers[x as usize] = diff;
        self.set_vf(borrow);
    }

    //8xyE - SHL Vx {, Vy}
    fn shift_left(&mut self, x: u8) {
        let vx = self.registers[x as usize];
        self.registers[x as usize] = vx << 1;
        //shift last bit 7 and mask 1 to get msb
        self.registers[0xF] = vx >> 7 & 1;
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
        self.i_register += vx as u16;
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
        chip8.memory[0] = 0x12;
        chip8.memory[1] = 0x34;
        assert_eq!(chip8.fetch(), 0x1234);
        assert_eq!(chip8.position_in_memory, 2);
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
        assert_eq!(chip8.position_in_memory, 2);
    }

    #[test]
    fn test_skip_if_not_equal_at_x() {
        //position in memory is 0 on init
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 0x12;
        //don't call tick, call method manually
        chip8.skip_if_not_equal_at_x(0, 0x13);
        //position in memory should be incremented by 2
        assert_eq!(chip8.position_in_memory, 2);
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
        assert_eq!(chip8.position_in_memory, 2);
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
        assert_eq!(chip8.registers[0xF], 0);
    }

    #[test]
    fn test_sub_y_from_x_with_borrow() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 5;
        chip8.registers[1] = 10;
        chip8.sub_y_from_x(0, 1);
        assert_eq!(chip8.registers[0], 251); // 5 - 10 = -5, which wraps around to 251 in unsigned 8-bit arithmetic
        assert_eq!(chip8.registers[0xF], 1);
    }

    #[test]
    fn test_shift_right_lsb_1() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 0b00001001;
        chip8.shift_right(0);
        assert_eq!(1, chip8.registers[0xF]);
        assert_eq!(0b00000100, chip8.registers[0]);
    }

    #[test]
    fn test_shift_right_lsb_0() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 0b00000110;
        chip8.shift_right(0);
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
        assert_eq!(chip8.registers[0xF], 0);
    }

    #[test]
    fn test_sub_x_from_y_with_borrow() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 10;
        chip8.registers[1] = 5;
        chip8.sub_x_from_y(0, 1); 
        assert_eq!(chip8.registers[0], 251); // 5 - 10 = -5, which wraps around to 251 in unsigned 8-bit arithmetic
        assert_eq!(chip8.registers[0xF], 1);
    }

    #[test]
    fn test_shift_left_msb_1() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 0b10010000;
        chip8.shift_left(0);
        assert_eq!(1, chip8.registers[0xF]);
        assert_eq!(0b00100000, chip8.registers[0]);
    }

    #[test]
    fn test_shift_left_msb_0() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 0b01100000;
        chip8.shift_left(0);
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
        assert_eq!(chip8.position_in_memory, 2);
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
}
