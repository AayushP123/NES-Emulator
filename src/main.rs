mod cpu;
mod ppu;

use cpu::Cpu;

fn main() {
    let mut cpu = Cpu {
        a: 0,
        x: 0,
        y: 0,
        pc: 0, // reset will set this from vector
        sp: 0xFD,
        p: 0x24,
        mem_buffer: [0; 65536],
    };

    let start: u16 = 0x8000;
    let sub: u16 = 0x9000;

    // Set reset vector to 0x8000
    cpu.write(0xFFFC, (start & 0x00FF) as u8);
    cpu.write(0xFFFD, (start >> 8) as u8);

    cpu.write(start.wrapping_add(0), 0x20);
    cpu.write(start.wrapping_add(1), (sub & 0x00FF) as u8);
    cpu.write(start.wrapping_add(2), (sub >> 8) as u8);
    cpu.write(start.wrapping_add(3), 0xA9);
    cpu.write(start.wrapping_add(4), 0x01);
    cpu.write(start.wrapping_add(5), 0x00);

    cpu.write(sub.wrapping_add(0), 0xA9);
    cpu.write(sub.wrapping_add(1), 0x10);
    cpu.write(sub.wrapping_add(2), 0x60);

    cpu.reset();

    while cpu.step() {}
}