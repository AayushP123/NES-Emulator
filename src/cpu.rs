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
    fn addr_absolute_y(&mut self) -> u16 {
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
            // ADC: Add with Carry opcodes
            0x69 => {
                let val = self.fetch_byte();
                self.adc(val);
                true
            }

            0x65 => {
                let addr = self.addr_zero_page();
                let val = self.read(addr);
                self.adc(val);
                true
            }

            0x75 => {
                let addr = self.addr_zero_page_x();
                let val = self.read(addr);
                self.adc(val);
                true
            }

            0x6D => {
                let addr = self.addr_absolute();
                let val = self.read(addr);
                self.adc(val);
                true
            }

            0x7D => {
                let addr = self.addr_absolute_x();
                let val = self.read(addr);
                self.adc(val);
                true
            }

            0x79 => {
                let addr = self.addr_absolute_y();
                let val = self.read(addr);
                self.adc(val);
                true
            }

            0x61 => {
                let addr = self.addr_indirect_x();
                let val = self.read(addr);
                self.adc(val);
                true
            }

            0x71 => {
                let addr = self.addr_indirect_y();
                let val = self.read(addr);
                self.adc(val);
                true
            }

            // SBC: Subtract with Carry opcodes
            0xE9 => { // Immediate
                let val = self.fetch_byte();
                self.sbc(val);
                true
            }

            0xE5 => {
                let addr = self.addr_zero_page();
                let val = self.read(addr);
                self.sbc(val);
                true
            }

            0xF5 => {
                let addr = self.addr_zero_page_x();
                let val = self.read(addr);
                self.sbc(val);
                true
            }

            0xED => {
                let addr = self.addr_absolute();
                let val = self.read(addr);
                self.sbc(val);
                true
            }

            0xFD => {
                let addr = self.addr_absolute_x();
                let val = self.read(addr);
                self.sbc(val);
                true
            }

            0xF9 => {
                let addr = self.addr_absolute_y();
                let val = self.read(addr);
                self.sbc(val);
                true
            }

            0xE1 => {
                let addr = self.addr_indirect_x();
                let val = self.read(addr);
                self.sbc(val);
                true
            }

            0xF1 => {
                let addr = self.addr_indirect_y();
                let val = self.read(addr);
                self.sbc(val);
                true
            }

            // Flag Toggles
            0x38 => { // Set Carry Flag
                self.p |= Self::FLAG_CARRY;
                true
            }

            0x18 => { // Clear Carry Flag
                self.p &= !Self::FLAG_CARRY;
                true
            }

            0x78 => { // Set Interrupt Disable
                self.p |= Self::FLAG_INTERRUPT;
                true
            }

            0x58 => { // Clear Interrupt Disable
                self.p &= !Self::FLAG_INTERRUPT;
                true
            }

            0xF8 => { // Set Decimal Flag
                self.p |= Self::FLAG_DECIMAL;
                true
            }

            0xD8 => { // Clear Decimal Flag
                self.p &= !Self::FLAG_DECIMAL;
                true
            }

            0xB8 => { // Clear Overflow Flag
                self.p &= !Self::FLAG_OVERFLOW;
                true
            }

            // Bitwise Logic (Immediate Mode)
            0x29 => { // Logical AND
                let value = self.fetch_byte();
                self.a &= value;
                self.set_zn(self.a);
                true
            }
            0x09 => { // Logical Inclusive OR
                let value = self.fetch_byte();
                self.a |= value;
                self.set_zn(self.a);
                true
            }
            0x49 => { // Exclusive OR (XOR)
                let value = self.fetch_byte();
                self.a ^= value;
                self.set_zn(self.a);
                true
            }

            // AND remaining modes
            0x25 => {
                let addr = self.addr_zero_page();
                self.a &= self.read(addr);
                self.set_zn(self.a);
                true
            }
            0x35 => {
                let addr = self.addr_zero_page_x();
                self.a &= self.read(addr);
                self.set_zn(self.a);
                true
            }
            0x2D => {
                let addr = self.addr_absolute();
                self.a &= self.read(addr);
                self.set_zn(self.a);
                true
            }
            0x3D => {
                let addr = self.addr_absolute_x();
                self.a &= self.read(addr);
                self.set_zn(self.a);
                true
            }
            0x39 => {
                let addr = self.addr_absolute_y();
                self.a &= self.read(addr);
                self.set_zn(self.a);
                true
            }
            0x21 => {
                let addr = self.addr_indirect_x();
                self.a &= self.read(addr);
                self.set_zn(self.a);
                true
            }
            0x31 => {
                let addr = self.addr_indirect_y();
                self.a &= self.read(addr);
                self.set_zn(self.a);
                true
            }

            // ORA remaining modes
            0x05 => {
                let addr = self.addr_zero_page();
                self.a |= self.read(addr);
                self.set_zn(self.a);
                true
            }
            0x15 => {
                let addr = self.addr_zero_page_x();
                self.a |= self.read(addr);
                self.set_zn(self.a);
                true
            }
            0x0D => {
                let addr = self.addr_absolute();
                self.a |= self.read(addr);
                self.set_zn(self.a);
                true
            }
            0x1D => {
                let addr = self.addr_absolute_x();
                self.a |= self.read(addr);
                self.set_zn(self.a);
                true
            }
            0x19 => {
                let addr = self.addr_absolute_y();
                self.a |= self.read(addr);
                self.set_zn(self.a);
                true
            }
            0x01 => {
                let addr = self.addr_indirect_x();
                self.a |= self.read(addr);
                self.set_zn(self.a);
                true
            }
            0x11 => {
                let addr = self.addr_indirect_y();
                self.a |= self.read(addr);
                self.set_zn(self.a);
                true
            }

            // EOR remaining modes
            0x45 => {
                let addr = self.addr_zero_page();
                self.a ^= self.read(addr);
                self.set_zn(self.a);
                true
            }
            0x55 => {
                let addr = self.addr_zero_page_x();
                self.a ^= self.read(addr);
                self.set_zn(self.a);
                true
            }
            0x4D => {
                let addr = self.addr_absolute();
                self.a ^= self.read(addr);
                self.set_zn(self.a);
                true
            }
            0x5D => {
                let addr = self.addr_absolute_x();
                self.a ^= self.read(addr);
                self.set_zn(self.a);
                true
            }
            0x59 => {
                let addr = self.addr_absolute_y();
                self.a ^= self.read(addr);
                self.set_zn(self.a);
                true
            }
            0x41 => {
                let addr = self.addr_indirect_x();
                self.a ^= self.read(addr);
                self.set_zn(self.a);
                true
            }
            0x51 => {
                let addr = self.addr_indirect_y();
                self.a ^= self.read(addr);
                self.set_zn(self.a);
                true
            }

            // BIT: test bits in memory against accumulator
            0x24 => {
                let addr = self.addr_zero_page();
                let val = self.read(addr);
                self.bit(val);
                true
            }
            0x2C => {
                let addr = self.addr_absolute();
                let val = self.read(addr);
                self.bit(val);
                true
            }

            // ASL: Arithmetic Shift Left
            0x0A => { // Accumulator
                self.a = self.asl(self.a);
                true
            }
            0x06 => {
                let addr = self.addr_zero_page();
                let r = self.asl(self.read(addr));
                self.write(addr, r);
                true
            }
            0x16 => {
                let addr = self.addr_zero_page_x();
                let r = self.asl(self.read(addr));
                self.write(addr, r);
                true
            }
            0x0E => {
                let addr = self.addr_absolute();
                let r = self.asl(self.read(addr));
                self.write(addr, r);
                true
            }
            0x1E => {
                let addr = self.addr_absolute_x();
                let r = self.asl(self.read(addr));
                self.write(addr, r);
                true
            }

            // LSR: Logical Shift Right
            0x4A => { // Accumulator
                self.a = self.lsr(self.a);
                true
            }
            0x46 => {
                let addr = self.addr_zero_page();
                let r = self.lsr(self.read(addr));
                self.write(addr, r);
                true
            }
            0x56 => {
                let addr = self.addr_zero_page_x();
                let r = self.lsr(self.read(addr));
                self.write(addr, r);
                true
            }
            0x4E => {
                let addr = self.addr_absolute();
                let r = self.lsr(self.read(addr));
                self.write(addr, r);
                true
            }
            0x5E => {
                let addr = self.addr_absolute_x();
                let r = self.lsr(self.read(addr));
                self.write(addr, r);
                true
            }

            // ROL: Rotate Left through carry
            0x2A => { // Accumulator
                self.a = self.rol(self.a);
                true
            }
            0x26 => {
                let addr = self.addr_zero_page();
                let r = self.rol(self.read(addr));
                self.write(addr, r);
                true
            }
            0x36 => {
                let addr = self.addr_zero_page_x();
                let r = self.rol(self.read(addr));
                self.write(addr, r);
                true
            }
            0x2E => {
                let addr = self.addr_absolute();
                let r = self.rol(self.read(addr));
                self.write(addr, r);
                true
            }
            0x3E => {
                let addr = self.addr_absolute_x();
                let r = self.rol(self.read(addr));
                self.write(addr, r);
                true
            }

            // ROR: Rotate Right through carry
            0x6A => { // Accumulator
                self.a = self.ror(self.a);
                true
            }
            0x66 => {
                let addr = self.addr_zero_page();
                let r = self.ror(self.read(addr));
                self.write(addr, r);
                true
            }
            0x76 => {
                let addr = self.addr_zero_page_x();
                let r = self.ror(self.read(addr));
                self.write(addr, r);
                true
            }
            0x6E => {
                let addr = self.addr_absolute();
                let r = self.ror(self.read(addr));
                self.write(addr, r);
                true
            }
            0x7E => {
                let addr = self.addr_absolute_x();
                let r = self.ror(self.read(addr));
                self.write(addr, r);
                true
            }

            // INC: Increment memory
            0xE6 => {
                let addr = self.addr_zero_page();
                let v = self.read(addr).wrapping_add(1);
                self.write(addr, v);
                self.set_zn(v);
                true
            }
            0xF6 => {
                let addr = self.addr_zero_page_x();
                let v = self.read(addr).wrapping_add(1);
                self.write(addr, v);
                self.set_zn(v);
                true
            }
            0xEE => {
                let addr = self.addr_absolute();
                let v = self.read(addr).wrapping_add(1);
                self.write(addr, v);
                self.set_zn(v);
                true
            }
            0xFE => {
                let addr = self.addr_absolute_x();
                let v = self.read(addr).wrapping_add(1);
                self.write(addr, v);
                self.set_zn(v);
                true
            }

            // DEC: Decrement memory
            0xC6 => {
                let addr = self.addr_zero_page();
                let v = self.read(addr).wrapping_sub(1);
                self.write(addr, v);
                self.set_zn(v);
                true
            }
            0xD6 => {
                let addr = self.addr_zero_page_x();
                let v = self.read(addr).wrapping_sub(1);
                self.write(addr, v);
                self.set_zn(v);
                true
            }
            0xCE => {
                let addr = self.addr_absolute();
                let v = self.read(addr).wrapping_sub(1);
                self.write(addr, v);
                self.set_zn(v);
                true
            }
            0xDE => {
                let addr = self.addr_absolute_x();
                let v = self.read(addr).wrapping_sub(1);
                self.write(addr, v);
                self.set_zn(v);
                true
            }

            // Branching opcode
            0x90 => {
                self.branch((self.p & Self::FLAG_CARRY) == 0);
                true
            }

            0xB0 => {
                self.branch((self.p & Self::FLAG_CARRY) != 0);
                true
            }

            0xD0 => {
                self.branch((self.p & Self::FLAG_ZERO) == 0);
                true
            }
            0xF0 => {
                self.branch((self.p & Self::FLAG_ZERO) != 0);
                true
            }

            0x10 => {
                self.branch((self.p & Self::FLAG_NEG) == 0);
                true
            }
            0x30 => {
                self.branch((self.p & Self::FLAG_NEG) != 0);
                true
            }

            0x50 => {
                self.branch((self.p & Self::FLAG_OVERFLOW) == 0);
                true
            }

            0x70 => {
                self.branch((self.p & Self::FLAG_OVERFLOW) != 0);
                true
            }

            // Compare Accumulator opcodes
            0xC9 => {
                let val = self.fetch_byte();
                self.compare(self.a, val);
                true
            }

            0xC5 => {
                let addr = self.addr_zero_page();
                let val = self.read(addr);
                self.compare(self.a, val);
                true
            }

            0xD5 => {
                let addr = self.addr_zero_page_x();
                let val = self.read(addr);
                self.compare(self.a, val);
                true
            }

            0xCD => {
                let addr = self.addr_absolute();
                let val = self.read(addr);
                self.compare(self.a, val);
                true
            }

            0xDD => {
                let addr = self.addr_absolute_x();
                let val = self.read(addr);
                self.compare(self.a, val);
                true
            }

            0xD9 => {
                let addr = self.addr_absolute_y();
                let val = self.read(addr);
                self.compare(self.a, val);
                true
            }

            0xC1 => {
                let addr = self.addr_indirect_x();
                let val = self.read(addr);
                self.compare(self.a, val);
                true
            }

            0xD1 => {
                let addr = self.addr_indirect_y();
                let val = self.read(addr);
                self.compare(self.a, val);
                true
            }

            // Compare X Register
            0xE0 => {
                let val = self.fetch_byte();
                self.compare(self.x, val);
                true
            }

            0xE4 => {
                let addr = self.addr_zero_page();
                let val = self.read(addr);
                self.compare(self.x, val);
                true
            }

            0xEC => {
                let addr = self.addr_absolute();
                let val = self.read(addr);
                self.compare(self.x, val);
                true
            }

            // Compare Y Register
            0xC0 => {
                let val = self.fetch_byte();
                self.compare(self.y, val);
                true
            }

            0xC4 => {
                let addr = self.addr_zero_page();
                let val = self.read(addr);
                self.compare(self.y, val);
                true
            }

            0xCC => {
                let addr = self.addr_absolute();
                let val = self.read(addr);
                self.compare(self.y, val);
                true
            }

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
            0xA0 => {
                // LDY immediate
                let value = self.fetch_byte();
                self.y = value;
                self.set_zn(self.y);
                true
            }
            0xA4 => {
                let addr = self.addr_zero_page();
                self.y = self.read(addr);
                self.set_zn(self.y);
                true
            }

            0xB4 => {
                let addr = self.addr_zero_page_x();
                self.y = self.read(addr);
                self.set_zn(self.y);
                true
            }

            0xAC => {
                let addr = self.addr_absolute();
                self.y = self.read(addr);
                self.set_zn(self.y);
                true
            }

            0xBC => {
                let addr = self.addr_absolute_x();
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

            // Load X Register
            0xA2 => {
                // LDX immediate
                let value = self.fetch_byte();
                self.x = value;
                self.set_zn(self.x);
                true
            }
            0xA6 => {
                let addr = self.addr_zero_page();
                self.x = self.read(addr);
                self.set_zn(self.x);
                true
            }
            0xB6 => {
                let addr = self.addr_zero_page_y();
                self.x = self.read(addr);
                self.set_zn(self.x);
                true
            }
            0xAE => {
                let addr = self.addr_absolute();
                self.x = self.read(addr);
                self.set_zn(self.x);
                true
            }
            0xBE => {
                let addr = self.addr_absolute_y();
                self.x = self.read(addr);
                self.set_zn(self.x);
                true
            }

            // Store X Register
            0x86 => {
                let addr = self.addr_zero_page();
                self.write(addr, self.x);
                true
            }
            0x96 => {
                let addr = self.addr_zero_page_y();
                self.write(addr, self.x);
                true
            }
            0x8E => {
                let addr = self.addr_absolute();
                self.write(addr, self.x);
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
            0xBA => {
                // TSX
                self.x = self.sp;
                self.set_zn(self.x);
                true
            }
            0x9A => {
                // TXS (no flags)
                self.sp = self.x;
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

            // Stack opcodes
            0x48 => {
                // PHA
                self.push_byte(self.a);
                true
            }
            0x68 => {
                // PLA
                self.a = self.pop_byte();
                self.set_zn(self.a);
                true
            }
            0x08 => {
                // PHP: B and unused bits always set when pushing
                self.push_byte(self.p | Self::FLAG_BREAK | Self::FLAG_BREAK2);
                true
            }
            0x28 => {
                // PLP: B cleared, unused set when pulling
                self.p = (self.pop_byte() & 0xCF) | 0x20;
                true
            }

            // Jump opcodes
            0x4C => {
                // JMP absolute
                self.pc = self.fetch_word();
                true
            }
            0x6C => {
                // JMP indirect, hardware page-wrap bug: high byte wraps within same page
                let ptr = self.fetch_word();
                let lo = self.read(ptr) as u16;
                let hi = self.read((ptr & 0xFF00) | ((ptr + 1) & 0x00FF)) as u16;
                self.pc = lo | (hi << 8);
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
            0x40 => {
                // RTI: pull P then PC
                self.p = (self.pop_byte() & 0xCF) | 0x20;
                self.pc = self.pop_word();
                true
            }

            0xEA => {
                // NOP
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

    // Trigger NMI: push PC and P, jump to vector at 0xFFFA
    pub fn trigger_nmi(&mut self) {
        self.push_word(self.pc);
        self.push_byte((self.p | Self::FLAG_BREAK2) & !Self::FLAG_BREAK);
        self.p |= Self::FLAG_INTERRUPT;
        self.pc = self.read_word(0xFFFA);
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

    const FLAG_CARRY: u8     = 0b0000_0001;
    const FLAG_ZERO: u8      = 0b0000_0010;
    const FLAG_INTERRUPT: u8 = 0b0000_0100;
    const FLAG_DECIMAL: u8   = 0b0000_1000;
    const FLAG_BREAK: u8     = 0b0001_0000;
    const FLAG_BREAK2: u8    = 0b0010_0000;
    const FLAG_OVERFLOW: u8  = 0b0100_0000;
    const FLAG_NEG: u8       = 0b1000_0000;

    // Compare function for funcs: CMP, CPX, CPY
    fn compare(&mut self, reg: u8, val: u8) {
        let res = reg.wrapping_sub(val);

        if reg >= val {
            self.p |= Self::FLAG_CARRY;
        } else {
            self.p &= !Self::FLAG_CARRY;
        }

        self.set_zn(res);
    }

    // Subtract with Carry function
    fn sbc(&mut self, data: u8) {
        self.adc(!data);
    }

    // Adder function for adding accumulator
    fn adc(&mut self, data: u8) {
        let a = self.a as u16;
        let b = data as u16;
        let c = if (self.p & Self::FLAG_CARRY) != 0 { 1 } else { 0 };

        let sum = a + b + c;
        let result = sum as u8;

        if sum > 0xFF {
            self.p |= Self::FLAG_CARRY;
        } else {
            self.p &= !Self::FLAG_CARRY;
        }

        if (self.a ^ result) & (data ^ result) & 0x80 != 0 {
            self.p |= Self::FLAG_OVERFLOW;
        } else {
            self.p &= !Self::FLAG_OVERFLOW;
        }

        self.a = result;
        self.set_zn(self.a);
    }

    // Old bit 7 goes to carry, shift left, bit 0 becomes 0
    fn asl(&mut self, val: u8) -> u8 {
        if val & 0x80 != 0 {
            self.p |= Self::FLAG_CARRY;
        } else {
            self.p &= !Self::FLAG_CARRY;
        }
        let result = val << 1;
        self.set_zn(result);
        result
    }

    // Old bit 0 goes to carry, shift right, bit 7 becomes 0
    fn lsr(&mut self, val: u8) -> u8 {
        if val & 0x01 != 0 {
            self.p |= Self::FLAG_CARRY;
        } else {
            self.p &= !Self::FLAG_CARRY;
        }
        let result = val >> 1;
        self.set_zn(result);
        result
    }

    // Rotate left through carry
    fn rol(&mut self, val: u8) -> u8 {
        let old_carry = (self.p & Self::FLAG_CARRY) != 0;
        if val & 0x80 != 0 {
            self.p |= Self::FLAG_CARRY;
        } else {
            self.p &= !Self::FLAG_CARRY;
        }
        let result = (val << 1) | (old_carry as u8);
        self.set_zn(result);
        result
    }

    // Rotate right through carry
    fn ror(&mut self, val: u8) -> u8 {
        let old_carry = (self.p & Self::FLAG_CARRY) != 0;
        if val & 0x01 != 0 {
            self.p |= Self::FLAG_CARRY;
        } else {
            self.p &= !Self::FLAG_CARRY;
        }
        let result = (val >> 1) | ((old_carry as u8) << 7);
        self.set_zn(result);
        result
    }

    // BIT: Z = !(A & val), N = val bit 7, V = val bit 6
    fn bit(&mut self, val: u8) {
        if self.a & val == 0 {
            self.p |= Self::FLAG_ZERO;
        } else {
            self.p &= !Self::FLAG_ZERO;
        }
        if val & 0x80 != 0 {
            self.p |= Self::FLAG_NEG;
        } else {
            self.p &= !Self::FLAG_NEG;
        }
        if val & 0x40 != 0 {
            self.p |= Self::FLAG_OVERFLOW;
        } else {
            self.p &= !Self::FLAG_OVERFLOW;
        }
    }

    // Helper for all branching instructions
    fn branch(&mut self, condition: bool) {
        let offset = self.fetch_byte() as i8;

        if condition {
            self.pc = self.pc.wrapping_add_signed(offset as i16);
        }
    }

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