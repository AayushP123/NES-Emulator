// Entire CPU data structure state
struct Cpu {
    a : u8,
    x : u8,
    y : u8,
    pc : u16, // Is the pointer to the next instruction for the CPU to run
    p : u8, // Used to determine special flags (i.e. zero, neg, overflow, etc)
    sp : u8, // Used for temporary state and tracks top of stack
    mem_buffer : [u8 ; 65536], // Each index holds one byte in the CPU
}
fn main() {
    // Mutable CPU values, set start to index of first pc value
    let mut cpu = Cpu {
        a: 0,
        x: 0,
        y: 0,
        pc: 0x8000,
        sp: 0xFD,
        p: 0x24,
        mem_buffer: [0; 65536]
    };
    let start = cpu.pc as usize;

    // load program
    cpu.mem_buffer[start] = 0xA9;
    cpu.mem_buffer[start + 1] = 0x10;
    cpu.mem_buffer[start + 2] = 0x00;

    // Starts loop at "start" opcode
    loop {
        let opcode = cpu.mem_buffer[cpu.pc as usize];
        if (opcode == 0xA9) {
            cpu.pc += 1;
            let value = cpu.mem_buffer[cpu.pc as usize];
            cpu.a = value;
            cpu.pc += 1;
            print!("pc = 0x{:04X} and a = {:02X}", cpu.pc, cpu.a);
        }
        else if (opcode == 0x00){
            break;
    }
}
    return;


}