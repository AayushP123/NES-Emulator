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

// Attach behaviour to CPU
impl Cpu {
    // Fetch byte, read one byte and interpret it, then advance PC by one
    fn fetch_byte(&mut self) -> u8 {
        let byte = self.mem_buffer[self.pc as usize];
        self.pc = self.pc.wrapping_add(1);
        byte
    }
    
    // Modifies CPU state
    fn step(&mut self) -> bool{

        // Reads byte from current PC in memory
        let opcode = self.mem_buffer[self.pc as usize];

        // Interprets the opcode (0xA9 means LDA immediate, meaning the next byte is the value not the address)
        if (opcode == 0xA9) {
            let value = self.fetch_byte(); // opcode value is set to the byte
            self.a = value;
            println!("pc = 0x{:04X} and a = {:02X}", self.pc, self.a);
            true
        } else if (opcode == 0x00) {
            false
        }
        else {
            panic!(
            "Unknown opcode {:02X}, Unknown value {:02X}", opcode, self.a
            )
        }
    }

    fn reset (&mut self) {
        self.pc = 0x8000;
        self.a = 0;
    }
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

    while cpu.step() {}
}