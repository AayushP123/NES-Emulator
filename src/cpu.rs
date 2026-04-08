// Entire CPU data structure state
pub struct Cpu {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16, // pointer to next instruction
    pub p: u8,   // status flags
    pub sp: u8,  // stack pointer
    pub mem_buffer: [u8; 65536],
}

impl Cpu {
    // Fetch byte at PC, then increment PC
    // Immediate opcode
    fn fetch_byte(&mut self) -> u8 {
        let byte = self.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    // Addressing opcode
    // Fetches 1 byte address
    fn addr_zero_page(&mut self) -> u16 {
        self.fetch_byte() as u16
    }

    // Byte + X, wraps around the zero page to make sure it doesn't go outta bounds
    fn addr_zero_page_x(&mut self) -> u16 {
        self.fetch_byte().wrapping_add(self.x) as u16
    }

    // Byte + Y, same idea, used by LDX or STX
    fn addr_zero_page_y(&mut self) -> u16 {
        self.fetch_byte().wrapping_add(self.y) as u16
    }

    // Fetch the full 16-bit address (This is the actual 16-bit being pulled)
    fn addr_absolute(&mut self) -> u16 {
        self.fetch_word()
    }

    // Fetches full 2-byte address, adds X to it.
    fn addr_absolute_x(&mut self) -> u16 {
        self.fetch_word().wrapping_add(self.x as u16)
    }

    // Fetches full 2-byte address, adds Y to it
    fn addr_absolute_y (&mut self) -> u16 {
        self.fetch_word().wrapping_add(self.y as u16)
    }

    // Add X to byte, lookup the pointer
    fn addr_indirect_x(&mut self) -> u16 {
        let base = self.fetch_byte().wrapping_add(self.x);
        let lo = self.read(base as u16) as u16;
        let hi = self.read(base.wrapping_add(1) as u16) as u16;
        lo | (hi << 8)
    }

    // Read byte, lookup the pointer, THEN add Y
    fn addr_indirect_y(&mut self) -> u16 {
        let base = self.fetch_byte();
        let lo = self.read(base as u16) as u16;
        let hi = self.read(base.wrapping_add(1) as u16) as u16;
        let ptr = lo | (hi << 8);
        ptr.wrapping_add(self.y as u16)
    }
    // Fetch 16-bit word (little-endian) from instruction stream (This is the function)
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
    pub fn step(&mut self) -> bool {
        let opcode_pc = self.pc;
        let opcode = self.fetch_byte();

        match opcode {
            // Load Accumulator opcodes
            0xA9 => { // Immediate
                let value = self.fetch_byte();
                self.a = value;
                self.set_zn(self.a);
                true
            }
            0xA5 => {
                let addr = self.addr_zero_page();
                self.a = self.read(addr);
                self.set_zn(self.a);
                true
            }

            0xB5 => {
                let addr = self.addr_zero_page_x();
                self.a = self.read(addr);
                self.set_zn(self.a);
                true
            }

            0xAD => {
                let addr = self.addr_absolute();
                self.a = self.read(addr);
                self.set_zn(self.a);
                true
            }

            0xBD => {
                let addr = self.addr_absolute_x();
                self.a = self.read(addr);
                self.set_zn(self.a);
                true
            }

            0xB9 => {
                let addr = self.addr_absolute_y();
                self.a = self.read(addr);
                self.set_zn(self.a);
                true
            }

            0xA1 => {
                let addr = self.addr_indirect_x();
                self.a = self.read(addr);
                self.set_zn(self.a);
                true
            }

            0xB1 => {
                let addr = self.addr_indirect_y();
                self.a = self.read(addr);
                self.set_zn(self.a);
                true
            }

            // Store accumulator
            0x85 => {
                let addr = self.addr_zero_page();
                self.write(addr, self.a);
                true
            }

            0x95 => {
                let addr = self.addr_zero_page_x();
                self.write(addr, self.a);
                true
            }

            0x8D => {
                let addr = self.addr_absolute();
                self.write(addr, self.a);
                true
            }

            0x9D => {
                let addr = self.addr_absolute_x();
                self.write(addr, self.a);
                true
            }

            0x99 => {
                let addr = self.addr_absolute_y();
                self.write(addr, self.a);
                true
            }

            0x81 => {
                let addr = self.addr_indirect_x();
                self.write(addr, self.a);
                true
            }

            0x91 => {
                let addr = self.addr_indirect_y();
                self.write(addr, self.a);
                true
            }

            // Load Y Register
            0xA4 => { let addr = self.addr_zero_page();
                self.y = self.read(addr);
                self.set_zn(self.y);
                true
            }

            0xB4 => { let addr = self.addr_zero_page_x();
                self.y = self.read(addr);
                self.set_zn(self.y);
                true
            }

            0xAC => { let addr = self.addr_absolute();
                self.y = self.read(addr);
                self.set_zn(self.y);
                true
            }

            0xBC => { let addr = self.addr_absolute_x();
                self.y = self.read(addr);
                self.set_zn(self.y);
                true
            }

            // Store Y Register
            0x84 => {
                let addr = self.addr_zero_page();
                self.write(addr, self.y);
                true
            }

            0x94 => {
                let addr = self.addr_zero_page_x();
                self.write(addr, self.y);
                true
            }

            0x8C => {
                let addr = self.addr_absolute();
                self.write(addr, self.y);
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
    pub fn reset(&mut self) {
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
    pub fn write(&mut self, addr: u16, data: u8) {
        self.mem_buffer[addr as usize] = data;
    }

    // Addressing opcodes

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