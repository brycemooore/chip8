mod chip8;
mod opcode;

use chip8::Chip8;

fn main() {
    let mut cpu = Chip8::new();

    cpu.registers[0] = 5;
    cpu.registers[1] = 10;

    let mem = &mut cpu.memory;

    //loads 3 opcodes into memory
    //2 says call at address specified by nnn (0x100)
    mem[0x000] = 0x21;
    mem[0x001] = 0x00;
    //call function twice
    mem[0x002] = 0x21;
    mem[0x003] = 0x00;
    //halts program
    mem[0x004] = 0x00;
    mem[0x005] = 0x00;

    //define function add value in register 1 twice to register 0
    let add_twice: [u8; 6] = [
        0x80, 0x14, 0x80, 0x14,
        //special instruction to have the program counrter jump back to the address at the top of the stack
        0x00, 0xEE,
    ];

    //load program into memory at 0x100
    mem[0x100..0x106].copy_from_slice(&add_twice);

    loop {
        cpu.tick();
    }

}
