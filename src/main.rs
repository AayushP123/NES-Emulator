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
        let byte = self.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
    }


    fn fetch_word(&mut self) -> u16 {
        let low = self.fetch_byte();
        let high = self.fetch_byte();
        (low as u16) | ((high as u16) << 8)
    }

    // Modifies CPU state
    fn step(&mut self) -> bool{

        // Reads byte from current PC in memory
        let opcode = self.fetch_byte();

        // Interprets the opcode (0xA9 means LDA immediate, meaning the next byte is the value not the address)
        if (opcode == 0xA9) {
            let value = self.fetch_byte(); // opcode value is set to the byte
            self.a = value;
            self.set_flag(self.a);
            println!("pc = 0x{:04X} and a = {:02X}", self.pc, self.a);
            true
        } else if (opcode == 0xA0) {
            let value = self.fetch_byte();
            self.y = value;
            self.set_flag(self.y);
            println!("pc = 0x{:04X} and y = {:02X}", self.pc, self.y);
            true
        }
        else if (opcode == 0xA2) {
            let value = self.fetch_byte();
            self.x = value;
            self.set_flag(self.x);
            println!("pc = 0x{:04X} and x = {:02X}", self.pc, self.x);
            true
        }
        else if (opcode == 0x00) {
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

    // Read takes address, and asks for the byte from the memory at that address index
    fn read (&self, addr : u16) -> u8 {
        self.mem_buffer[addr as usize]
    }

    // Write takes address, converts it to array index, and stores byte at that memory location
    fn write (&mut self, addr : u16, data : u8) {
        self.mem_buffer[addr as usize] = data;
    }
    const FLAG_ZERO : u8 = 0b0000_0010;
    const FLAG_NEG : u8 = 0b1000_0000;

    fn set_flag(&mut self, val : u8) {

        if(val == 0x00) {
            self.p |= Self::FLAG_ZERO
        } else{
            self.p &= !Self::FLAG_ZERO
        }

        if (val & 0x80) != 0 {
            self.p |= Self::FLAG_NEG
        } else{
            self.p &= !Self::FLAG_NEG
        }
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
    let start = cpu.pc;

    // load program
    cpu.write(start, 0xA9);
    cpu.write(start.wrapping_add(1), 0x10);
    cpu.write(start.wrapping_add(2), 0x00);

    while cpu.step() {}
}