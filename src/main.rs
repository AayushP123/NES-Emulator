// Entire CPU data structure state
struct Cpu {
    a: u8,
    x: u8,
    y: u8,
    pc: u16, // pointer to next instruction
    p: u8,   // status flags
    sp: u8,  // stack pointer
    mem_buffer: [u8; 65536],
}

impl Cpu {
    // Fetch byte at PC, then increment PC
    fn fetch_byte(&mut self) -> u8 {
        let byte = self.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    // Fetch 16-bit word (little-endian) from instruction stream
    fn fetch_word(&mut self) -> u16 {
        let low = self.fetch_byte();
        let high = self.fetch_byte();
        (low as u16) | ((high as u16) << 8)
    }

    // Read 16-bit word (little-endian) from memory at addr
    fn read_word(&self, addr: u16) -> u16 {
        let lo = self.read(addr) as u16;
        let hi = self.read(addr.wrapping_add(1)) as u16;
        lo | (hi << 8)
    }

    // Compute stack address (page 0x01 + SP)
    fn stack_addr(&self) -> u16 {
        0x0100u16 | (self.sp as u16)
    }

    // Push one byte onto stack
    fn push_byte(&mut self, v: u8) {
        let addr = self.stack_addr();
        self.write(addr, v);
        self.sp = self.sp.wrapping_sub(1);
    }

    // Pop one byte from stack
    fn pop_byte(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let addr = self.stack_addr();
        self.read(addr)
    }

    // Push 16-bit word onto stack (high byte first)
    fn push_word(&mut self, v: u16) {
        let hi = (v >> 8) as u8;
        let lo = (v & 0x00FF) as u8;
        self.push_byte(hi);
        self.push_byte(lo);
    }

    // Pop 16-bit word from stack (low byte first)
    fn pop_word(&mut self) -> u16 {
        let lo = self.pop_byte() as u16;
        let hi = self.pop_byte() as u16;
        lo | (hi << 8)
    }

    // Execute one instruction. Return false to stop (BRK).
    fn step(&mut self) -> bool {
        let opcode_pc = self.pc;
        let opcode = self.fetch_byte();

        match opcode {
            0xA9 => {
                // LDA immediate
                let value = self.fetch_byte();
                self.a = value;
                self.set_zn(self.a);
                true
            }
            0xA2 => {
                // LDX immediate
                let value = self.fetch_byte();
                self.x = value;
                self.set_zn(self.x);
                true
            }
            0xA0 => {
                // LDY immediate
                let value = self.fetch_byte();
                self.y = value;
                self.set_zn(self.y);
                true
            }
            0xAA => {
                // TAX
                self.x = self.a;
                self.set_zn(self.x);
                true
            }
            0x8A => {
                // TXA
                self.a = self.x;
                self.set_zn(self.a);
                true
            }
            0xA8 => {
                // TAY
                self.y = self.a;
                self.set_zn(self.y);
                true
            }
            0x98 => {
                // TYA
                self.a = self.y;
                self.set_zn(self.a);
                true
            }
            0xE8 => {
                // INX
                self.x = self.x.wrapping_add(1);
                self.set_zn(self.x);
                true
            }
            0xCA => {
                // DEX
                self.x = self.x.wrapping_sub(1);
                self.set_zn(self.x);
                true
            }
            0xC8 => {
                // INY
                self.y = self.y.wrapping_add(1);
                self.set_zn(self.y);
                true
            }
            0x88 => {
                // DEY
                self.y = self.y.wrapping_sub(1);
                self.set_zn(self.y);
                true
            }
            0x20 => {
                // JSR abs
                // Fetch target address
                let target = self.fetch_word();

                // Push return address (PC - 1)
                let ret = self.pc.wrapping_sub(1);
                self.push_word(ret);

                // Jump to target
                self.pc = target;
                true
            }
            0x60 => {
                // RTS
                // Pull return address and add 1
                let ret = self.pop_word();
                self.pc = ret.wrapping_add(1);
                true
            }
            0x00 => {
                // BRK stops execution loop
                false
            }
            _ => {
                panic!(
                    "Unknown opcode {:02X} at PC {:04X}",
                    opcode, opcode_pc
                )
            }
        }
    }

    // Reset to vector at 0xFFFC/0xFFFD
    fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFD;
        self.p = 0x24;
        self.pc = self.read_word(0xFFFC);
    }

    // Memory read
    fn read(&self, addr: u16) -> u8 {
        self.mem_buffer[addr as usize]
    }

    // Memory write
    fn write(&mut self, addr: u16, data: u8) {
        self.mem_buffer[addr as usize] = data;
    }

    const FLAG_ZERO: u8 = 0b0000_0010;
    const FLAG_NEG: u8 = 0b1000_0000;

    // Update Z and N based on value
    fn set_zn(&mut self, val: u8) {
        if val == 0 {
            self.p |= Self::FLAG_ZERO;
        } else {
            self.p &= !Self::FLAG_ZERO;
        }

        if (val & 0x80) != 0 {
            self.p |= Self::FLAG_NEG;
        } else {
            self.p &= !Self::FLAG_NEG;
        }
    }
}

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

    // Program at 0x8000:
    // JSR $9000
    // LDA #$01
    // BRK
    cpu.write(start.wrapping_add(0), 0x20);
    cpu.write(start.wrapping_add(1), (sub & 0x00FF) as u8);
    cpu.write(start.wrapping_add(2), (sub >> 8) as u8);
    cpu.write(start.wrapping_add(3), 0xA9);
    cpu.write(start.wrapping_add(4), 0x01);
    cpu.write(start.wrapping_add(5), 0x00);

    // Subroutine at 0x9000:
    // LDA #$10
    // RTS
    cpu.write(sub.wrapping_add(0), 0xA9);
    cpu.write(sub.wrapping_add(1), 0x10);
    cpu.write(sub.wrapping_add(2), 0x60);

    cpu.reset();

    while cpu.step() {}
}
