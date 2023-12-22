use chip8::chip::Chip8;

fn main() {
    let mut cpu = Chip8::new();
    println!("Chip {:>?}", cpu);
}
