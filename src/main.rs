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
    let mut cpu = Cpu{a : 0, x : 0, y : 0, pc : 0x8000, sp : 0xFD, p : 0x24, mem_buffer : [0;65536]};
    let start = cpu.pc as usize;

    // tiny program bytes that changes byte values in CPU
    cpu.mem_buffer[start] = 0xA9;
    cpu.mem_buffer[start + 1] = 0x10;
    cpu.mem_buffer[start + 2] = 0x00;


    // check values
    println!("{:02X} {:02X} {:02X}",
             cpu.mem_buffer[start],
             cpu.mem_buffer[start + 1],
             cpu.mem_buffer[start + 2],
    )
}