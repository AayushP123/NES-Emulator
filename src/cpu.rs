// Entire CPU data structure state
use crate::bus::Bus;

pub struct Cpu {
    pub a:  u8,
    pub x:  u8,
    pub y:  u8,
    pub pc: u16, // pointer to next instruction
    pub p:  u8,  // status flags
    pub sp: u8,  // stack pointer
}

impl Cpu {
    // Fetch byte at PC, then increment PC
    fn fetch_byte(&mut self, bus: &mut Bus) -> u8 {
        let byte = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    // Fetch 16-bit word little-endian from instruction stream
    fn fetch_word(&mut self, bus: &mut Bus) -> u16 {
        let lo = self.fetch_byte(bus);
        let hi = self.fetch_byte(bus);
        (lo as u16) | ((hi as u16) << 8)
    }

    // Read 16-bit word little-endian from memory at addr
    fn read_word(&self, bus: &mut Bus, addr: u16) -> u16 {
        let lo = bus.read(addr) as u16;
        let hi = bus.read(addr.wrapping_add(1)) as u16;
        lo | (hi << 8)
    }

    // Addressing mode: fetch 1 byte address
    fn addr_zero_page(&mut self, bus: &mut Bus) -> u16 {
        self.fetch_byte(bus) as u16
    }

    // Byte + X, wraps inside zero page to stay in bounds
    fn addr_zero_page_x(&mut self, bus: &mut Bus) -> u16 {
        self.fetch_byte(bus).wrapping_add(self.x) as u16
    }

    // Byte + Y, used by LDX / STX
    fn addr_zero_page_y(&mut self, bus: &mut Bus) -> u16 {
        self.fetch_byte(bus).wrapping_add(self.y) as u16
    }

    // Fetch the full 16-bit address
    fn addr_absolute(&mut self, bus: &mut Bus) -> u16 {
        self.fetch_word(bus)
    }

    // Fetches full 2-byte address, adds X to it
    fn addr_absolute_x(&mut self, bus: &mut Bus) -> u16 {
        self.fetch_word(bus).wrapping_add(self.x as u16)
    }

    // Fetches full 2-byte address, adds Y to it
    fn addr_absolute_y(&mut self, bus: &mut Bus) -> u16 {
        self.fetch_word(bus).wrapping_add(self.y as u16)
    }

    // Add X to byte, lookup the pointer
    fn addr_indirect_x(&mut self, bus: &mut Bus) -> u16 {
        let base = self.fetch_byte(bus).wrapping_add(self.x);
        let lo   = bus.read(base as u16) as u16;
        let hi   = bus.read(base.wrapping_add(1) as u16) as u16;
        lo | (hi << 8)
    }

    // Read byte, lookup the pointer, THEN add Y
    fn addr_indirect_y(&mut self, bus: &mut Bus) -> u16 {
        let base = self.fetch_byte(bus);
        let lo   = bus.read(base as u16) as u16;
        let hi   = bus.read(base.wrapping_add(1) as u16) as u16;
        let ptr  = lo | (hi << 8);
        ptr.wrapping_add(self.y as u16)
    }

    // Compute stack address (page 0x01 + SP)
    fn stack_addr(&self) -> u16 {
        0x0100u16 | (self.sp as u16)
    }

    // Push one byte onto stack
    fn push_byte(&mut self, bus: &mut Bus, v: u8) {
        let addr = self.stack_addr();
        bus.write(addr, v);
        self.sp = self.sp.wrapping_sub(1);
    }

    // Pop one byte from stack
    fn pop_byte(&mut self, bus: &mut Bus) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let addr = self.stack_addr();
        bus.read(addr)
    }

    // Push 16-bit word onto stack (high byte first)
    fn push_word(&mut self, bus: &mut Bus, v: u16) {
        let hi = (v >> 8) as u8;
        let lo = (v & 0x00FF) as u8;
        self.push_byte(bus, hi);
        self.push_byte(bus, lo);
    }

    // Pop 16-bit word from stack (low byte first)
    fn pop_word(&mut self, bus: &mut Bus) -> u16 {
        let lo = self.pop_byte(bus) as u16;
        let hi = self.pop_byte(bus) as u16;
        lo | (hi << 8)
    }

    // Execute one instruction. Returns false to stop (BRK).
    pub fn step(&mut self, bus: &mut Bus) -> bool {
        let opcode_pc = self.pc;
        let opcode    = self.fetch_byte(bus);

        match opcode {
            // ADC: Add with Carry opcodes
            0x69 => {
                let val = self.fetch_byte(bus);
                self.adc(val);
                true
            }
            0x65 => {
                let addr = self.addr_zero_page(bus);
                let val  = bus.read(addr);
                self.adc(val);
                true
            }
            0x75 => {
                let addr = self.addr_zero_page_x(bus);
                let val  = bus.read(addr);
                self.adc(val);
                true
            }
            0x6D => {
                let addr = self.addr_absolute(bus);
                let val  = bus.read(addr);
                self.adc(val);
                true
            }
            0x7D => {
                let addr = self.addr_absolute_x(bus);
                let val  = bus.read(addr);
                self.adc(val);
                true
            }
            0x79 => {
                let addr = self.addr_absolute_y(bus);
                let val  = bus.read(addr);
                self.adc(val);
                true
            }
            0x61 => {
                let addr = self.addr_indirect_x(bus);
                let val  = bus.read(addr);
                self.adc(val);
                true
            }
            0x71 => {
                let addr = self.addr_indirect_y(bus);
                let val  = bus.read(addr);
                self.adc(val);
                true
            }

            // SBC: Subtract with Carry opcodes
            0xE9 => {
                let val = self.fetch_byte(bus);
                self.sbc(val);
                true
            }
            0xE5 => {
                let addr = self.addr_zero_page(bus);
                let val  = bus.read(addr);
                self.sbc(val);
                true
            }
            0xF5 => {
                let addr = self.addr_zero_page_x(bus);
                let val  = bus.read(addr);
                self.sbc(val);
                true
            }
            0xED => {
                let addr = self.addr_absolute(bus);
                let val  = bus.read(addr);
                self.sbc(val);
                true
            }
            0xFD => {
                let addr = self.addr_absolute_x(bus);
                let val  = bus.read(addr);
                self.sbc(val);
                true
            }
            0xF9 => {
                let addr = self.addr_absolute_y(bus);
                let val  = bus.read(addr);
                self.sbc(val);
                true
            }
            0xE1 => {
                let addr = self.addr_indirect_x(bus);
                let val  = bus.read(addr);
                self.sbc(val);
                true
            }
            0xF1 => {
                let addr = self.addr_indirect_y(bus);
                let val  = bus.read(addr);
                self.sbc(val);
                true
            }

            // Flag Toggles
            0x38 => { self.p |= Self::FLAG_CARRY;      true } // Set Carry
            0x18 => { self.p &= !Self::FLAG_CARRY;     true } // Clear Carry
            0x78 => { self.p |= Self::FLAG_INTERRUPT;  true } // Set Interrupt Disable
            0x58 => { self.p &= !Self::FLAG_INTERRUPT; true } // Clear Interrupt Disable
            0xF8 => { self.p |= Self::FLAG_DECIMAL;    true } // Set Decimal
            0xD8 => { self.p &= !Self::FLAG_DECIMAL;   true } // Clear Decimal
            0xB8 => { self.p &= !Self::FLAG_OVERFLOW;  true } // Clear Overflow

            // AND: Logical AND with accumulator
            0x29 => {
                let val = self.fetch_byte(bus);
                self.a &= val;
                self.set_zn(self.a);
                true
            }
            0x25 => {
                let addr = self.addr_zero_page(bus);
                self.a &= bus.read(addr);
                self.set_zn(self.a);
                true
            }
            0x35 => {
                let addr = self.addr_zero_page_x(bus);
                self.a &= bus.read(addr);
                self.set_zn(self.a);
                true
            }
            0x2D => {
                let addr = self.addr_absolute(bus);
                self.a &= bus.read(addr);
                self.set_zn(self.a);
                true
            }
            0x3D => {
                let addr = self.addr_absolute_x(bus);
                self.a &= bus.read(addr);
                self.set_zn(self.a);
                true
            }
            0x39 => {
                let addr = self.addr_absolute_y(bus);
                self.a &= bus.read(addr);
                self.set_zn(self.a);
                true
            }
            0x21 => {
                let addr = self.addr_indirect_x(bus);
                self.a &= bus.read(addr);
                self.set_zn(self.a);
                true
            }
            0x31 => {
                let addr = self.addr_indirect_y(bus);
                self.a &= bus.read(addr);
                self.set_zn(self.a);
                true
            }

            // ORA: Logical Inclusive OR with accumulator
            0x09 => {
                let val = self.fetch_byte(bus);
                self.a |= val;
                self.set_zn(self.a);
                true
            }
            0x05 => {
                let addr = self.addr_zero_page(bus);
                self.a |= bus.read(addr);
                self.set_zn(self.a);
                true
            }
            0x15 => {
                let addr = self.addr_zero_page_x(bus);
                self.a |= bus.read(addr);
                self.set_zn(self.a);
                true
            }
            0x0D => {
                let addr = self.addr_absolute(bus);
                self.a |= bus.read(addr);
                self.set_zn(self.a);
                true
            }
            0x1D => {
                let addr = self.addr_absolute_x(bus);
                self.a |= bus.read(addr);
                self.set_zn(self.a);
                true
            }
            0x19 => {
                let addr = self.addr_absolute_y(bus);
                self.a |= bus.read(addr);
                self.set_zn(self.a);
                true
            }
            0x01 => {
                let addr = self.addr_indirect_x(bus);
                self.a |= bus.read(addr);
                self.set_zn(self.a);
                true
            }
            0x11 => {
                let addr = self.addr_indirect_y(bus);
                self.a |= bus.read(addr);
                self.set_zn(self.a);
                true
            }

            // EOR: Exclusive OR with accumulator
            0x49 => {
                let val = self.fetch_byte(bus);
                self.a ^= val;
                self.set_zn(self.a);
                true
            }
            0x45 => {
                let addr = self.addr_zero_page(bus);
                self.a ^= bus.read(addr);
                self.set_zn(self.a);
                true
            }
            0x55 => {
                let addr = self.addr_zero_page_x(bus);
                self.a ^= bus.read(addr);
                self.set_zn(self.a);
                true
            }
            0x4D => {
                let addr = self.addr_absolute(bus);
                self.a ^= bus.read(addr);
                self.set_zn(self.a);
                true
            }
            0x5D => {
                let addr = self.addr_absolute_x(bus);
                self.a ^= bus.read(addr);
                self.set_zn(self.a);
                true
            }
            0x59 => {
                let addr = self.addr_absolute_y(bus);
                self.a ^= bus.read(addr);
                self.set_zn(self.a);
                true
            }
            0x41 => {
                let addr = self.addr_indirect_x(bus);
                self.a ^= bus.read(addr);
                self.set_zn(self.a);
                true
            }
            0x51 => {
                let addr = self.addr_indirect_y(bus);
                self.a ^= bus.read(addr);
                self.set_zn(self.a);
                true
            }

            // BIT: test bits in memory against accumulator
            0x24 => {
                let addr = self.addr_zero_page(bus);
                let val  = bus.read(addr);
                self.bit(val);
                true
            }
            0x2C => {
                let addr = self.addr_absolute(bus);
                let val  = bus.read(addr);
                self.bit(val);
                true
            }

            // ASL: Arithmetic Shift Left
            0x0A => {
                // Accumulator mode
                self.a = self.asl(self.a);
                true
            }
            0x06 => {
                let addr = self.addr_zero_page(bus);
                let r    = self.asl(bus.read(addr));
                bus.write(addr, r);
                true
            }
            0x16 => {
                let addr = self.addr_zero_page_x(bus);
                let r    = self.asl(bus.read(addr));
                bus.write(addr, r);
                true
            }
            0x0E => {
                let addr = self.addr_absolute(bus);
                let r    = self.asl(bus.read(addr));
                bus.write(addr, r);
                true
            }
            0x1E => {
                let addr = self.addr_absolute_x(bus);
                let r    = self.asl(bus.read(addr));
                bus.write(addr, r);
                true
            }

            // LSR: Logical Shift Right
            0x4A => {
                // Accumulator mode
                self.a = self.lsr(self.a);
                true
            }
            0x46 => {
                let addr = self.addr_zero_page(bus);
                let r    = self.lsr(bus.read(addr));
                bus.write(addr, r);
                true
            }
            0x56 => {
                let addr = self.addr_zero_page_x(bus);
                let r    = self.lsr(bus.read(addr));
                bus.write(addr, r);
                true
            }
            0x4E => {
                let addr = self.addr_absolute(bus);
                let r    = self.lsr(bus.read(addr));
                bus.write(addr, r);
                true
            }
            0x5E => {
                let addr = self.addr_absolute_x(bus);
                let r    = self.lsr(bus.read(addr));
                bus.write(addr, r);
                true
            }

            // ROL: Rotate Left through carry
            0x2A => {
                // Accumulator mode
                self.a = self.rol(self.a);
                true
            }
            0x26 => {
                let addr = self.addr_zero_page(bus);
                let r    = self.rol(bus.read(addr));
                bus.write(addr, r);
                true
            }
            0x36 => {
                let addr = self.addr_zero_page_x(bus);
                let r    = self.rol(bus.read(addr));
                bus.write(addr, r);
                true
            }
            0x2E => {
                let addr = self.addr_absolute(bus);
                let r    = self.rol(bus.read(addr));
                bus.write(addr, r);
                true
            }
            0x3E => {
                let addr = self.addr_absolute_x(bus);
                let r    = self.rol(bus.read(addr));
                bus.write(addr, r);
                true
            }

            // ROR: Rotate Right through carry
            0x6A => {
                // Accumulator mode
                self.a = self.ror(self.a);
                true
            }
            0x66 => {
                let addr = self.addr_zero_page(bus);
                let r    = self.ror(bus.read(addr));
                bus.write(addr, r);
                true
            }
            0x76 => {
                let addr = self.addr_zero_page_x(bus);
                let r    = self.ror(bus.read(addr));
                bus.write(addr, r);
                true
            }
            0x6E => {
                let addr = self.addr_absolute(bus);
                let r    = self.ror(bus.read(addr));
                bus.write(addr, r);
                true
            }
            0x7E => {
                let addr = self.addr_absolute_x(bus);
                let r    = self.ror(bus.read(addr));
                bus.write(addr, r);
                true
            }

            // INC: Increment memory
            0xE6 => {
                let addr = self.addr_zero_page(bus);
                let v    = bus.read(addr).wrapping_add(1);
                bus.write(addr, v);
                self.set_zn(v);
                true
            }
            0xF6 => {
                let addr = self.addr_zero_page_x(bus);
                let v    = bus.read(addr).wrapping_add(1);
                bus.write(addr, v);
                self.set_zn(v);
                true
            }
            0xEE => {
                let addr = self.addr_absolute(bus);
                let v    = bus.read(addr).wrapping_add(1);
                bus.write(addr, v);
                self.set_zn(v);
                true
            }
            0xFE => {
                let addr = self.addr_absolute_x(bus);
                let v    = bus.read(addr).wrapping_add(1);
                bus.write(addr, v);
                self.set_zn(v);
                true
            }

            // DEC: Decrement memory
            0xC6 => {
                let addr = self.addr_zero_page(bus);
                let v    = bus.read(addr).wrapping_sub(1);
                bus.write(addr, v);
                self.set_zn(v);
                true
            }
            0xD6 => {
                let addr = self.addr_zero_page_x(bus);
                let v    = bus.read(addr).wrapping_sub(1);
                bus.write(addr, v);
                self.set_zn(v);
                true
            }
            0xCE => {
                let addr = self.addr_absolute(bus);
                let v    = bus.read(addr).wrapping_sub(1);
                bus.write(addr, v);
                self.set_zn(v);
                true
            }
            0xDE => {
                let addr = self.addr_absolute_x(bus);
                let v    = bus.read(addr).wrapping_sub(1);
                bus.write(addr, v);
                self.set_zn(v);
                true
            }

            // Branching opcodes
            0x90 => { self.branch(bus, (self.p & Self::FLAG_CARRY)    == 0); true } // BCC
            0xB0 => { self.branch(bus, (self.p & Self::FLAG_CARRY)    != 0); true } // BCS
            0xD0 => { self.branch(bus, (self.p & Self::FLAG_ZERO)     == 0); true } // BNE
            0xF0 => { self.branch(bus, (self.p & Self::FLAG_ZERO)     != 0); true } // BEQ
            0x10 => { self.branch(bus, (self.p & Self::FLAG_NEG)      == 0); true } // BPL
            0x30 => { self.branch(bus, (self.p & Self::FLAG_NEG)      != 0); true } // BMI
            0x50 => { self.branch(bus, (self.p & Self::FLAG_OVERFLOW) == 0); true } // BVC
            0x70 => { self.branch(bus, (self.p & Self::FLAG_OVERFLOW) != 0); true } // BVS

            // CMP: Compare Accumulator
            0xC9 => {
                let val = self.fetch_byte(bus);
                self.compare(self.a, val);
                true
            }
            0xC5 => {
                let addr = self.addr_zero_page(bus);
                let val  = bus.read(addr);
                self.compare(self.a, val);
                true
            }
            0xD5 => {
                let addr = self.addr_zero_page_x(bus);
                let val  = bus.read(addr);
                self.compare(self.a, val);
                true
            }
            0xCD => {
                let addr = self.addr_absolute(bus);
                let val  = bus.read(addr);
                self.compare(self.a, val);
                true
            }
            0xDD => {
                let addr = self.addr_absolute_x(bus);
                let val  = bus.read(addr);
                self.compare(self.a, val);
                true
            }
            0xD9 => {
                let addr = self.addr_absolute_y(bus);
                let val  = bus.read(addr);
                self.compare(self.a, val);
                true
            }
            0xC1 => {
                let addr = self.addr_indirect_x(bus);
                let val  = bus.read(addr);
                self.compare(self.a, val);
                true
            }
            0xD1 => {
                let addr = self.addr_indirect_y(bus);
                let val  = bus.read(addr);
                self.compare(self.a, val);
                true
            }

            // CPX: Compare X Register
            0xE0 => {
                let val = self.fetch_byte(bus);
                self.compare(self.x, val);
                true
            }
            0xE4 => {
                let addr = self.addr_zero_page(bus);
                let val  = bus.read(addr);
                self.compare(self.x, val);
                true
            }
            0xEC => {
                let addr = self.addr_absolute(bus);
                let val  = bus.read(addr);
                self.compare(self.x, val);
                true
            }

            // CPY: Compare Y Register
            0xC0 => {
                let val = self.fetch_byte(bus);
                self.compare(self.y, val);
                true
            }
            0xC4 => {
                let addr = self.addr_zero_page(bus);
                let val  = bus.read(addr);
                self.compare(self.y, val);
                true
            }
            0xCC => {
                let addr = self.addr_absolute(bus);
                let val  = bus.read(addr);
                self.compare(self.y, val);
                true
            }

            // LDA: Load Accumulator
            0xA9 => {
                self.a = self.fetch_byte(bus);
                self.set_zn(self.a);
                true
            }
            0xA5 => {
                let addr = self.addr_zero_page(bus);
                self.a   = bus.read(addr);
                self.set_zn(self.a);
                true
            }
            0xB5 => {
                let addr = self.addr_zero_page_x(bus);
                self.a   = bus.read(addr);
                self.set_zn(self.a);
                true
            }
            0xAD => {
                let addr = self.addr_absolute(bus);
                self.a   = bus.read(addr);
                self.set_zn(self.a);
                true
            }
            0xBD => {
                let addr = self.addr_absolute_x(bus);
                self.a   = bus.read(addr);
                self.set_zn(self.a);
                true
            }
            0xB9 => {
                let addr = self.addr_absolute_y(bus);
                self.a   = bus.read(addr);
                self.set_zn(self.a);
                true
            }
            0xA1 => {
                let addr = self.addr_indirect_x(bus);
                self.a   = bus.read(addr);
                self.set_zn(self.a);
                true
            }
            0xB1 => {
                let addr = self.addr_indirect_y(bus);
                self.a   = bus.read(addr);
                self.set_zn(self.a);
                true
            }

            // STA: Store Accumulator
            0x85 => {
                let addr = self.addr_zero_page(bus);
                bus.write(addr, self.a);
                true
            }
            0x95 => {
                let addr = self.addr_zero_page_x(bus);
                bus.write(addr, self.a);
                true
            }
            0x8D => {
                let addr = self.addr_absolute(bus);
                bus.write(addr, self.a);
                true
            }
            0x9D => {
                let addr = self.addr_absolute_x(bus);
                bus.write(addr, self.a);
                true
            }
            0x99 => {
                let addr = self.addr_absolute_y(bus);
                bus.write(addr, self.a);
                true
            }
            0x81 => {
                let addr = self.addr_indirect_x(bus);
                bus.write(addr, self.a);
                true
            }
            0x91 => {
                let addr = self.addr_indirect_y(bus);
                bus.write(addr, self.a);
                true
            }

            // LDX: Load X Register
            0xA2 => {
                self.x = self.fetch_byte(bus);
                self.set_zn(self.x);
                true
            }
            0xA6 => {
                let addr = self.addr_zero_page(bus);
                self.x   = bus.read(addr);
                self.set_zn(self.x);
                true
            }
            0xB6 => {
                let addr = self.addr_zero_page_y(bus);
                self.x   = bus.read(addr);
                self.set_zn(self.x);
                true
            }
            0xAE => {
                let addr = self.addr_absolute(bus);
                self.x   = bus.read(addr);
                self.set_zn(self.x);
                true
            }
            0xBE => {
                let addr = self.addr_absolute_y(bus);
                self.x   = bus.read(addr);
                self.set_zn(self.x);
                true
            }

            // STX: Store X Register
            0x86 => {
                let addr = self.addr_zero_page(bus);
                bus.write(addr, self.x);
                true
            }
            0x96 => {
                let addr = self.addr_zero_page_y(bus);
                bus.write(addr, self.x);
                true
            }
            0x8E => {
                let addr = self.addr_absolute(bus);
                bus.write(addr, self.x);
                true
            }

            // LDY: Load Y Register
            0xA0 => {
                self.y = self.fetch_byte(bus);
                self.set_zn(self.y);
                true
            }
            0xA4 => {
                let addr = self.addr_zero_page(bus);
                self.y   = bus.read(addr);
                self.set_zn(self.y);
                true
            }
            0xB4 => {
                let addr = self.addr_zero_page_x(bus);
                self.y   = bus.read(addr);
                self.set_zn(self.y);
                true
            }
            0xAC => {
                let addr = self.addr_absolute(bus);
                self.y   = bus.read(addr);
                self.set_zn(self.y);
                true
            }
            0xBC => {
                let addr = self.addr_absolute_x(bus);
                self.y   = bus.read(addr);
                self.set_zn(self.y);
                true
            }

            // STY: Store Y Register
            0x84 => {
                let addr = self.addr_zero_page(bus);
                bus.write(addr, self.y);
                true
            }
            0x94 => {
                let addr = self.addr_zero_page_x(bus);
                bus.write(addr, self.y);
                true
            }
            0x8C => {
                let addr = self.addr_absolute(bus);
                bus.write(addr, self.y);
                true
            }

            // Register transfers
            0xAA => { self.x = self.a;  self.set_zn(self.x); true } // TAX
            0x8A => { self.a = self.x;  self.set_zn(self.a); true } // TXA
            0xA8 => { self.y = self.a;  self.set_zn(self.y); true } // TAY
            0x98 => { self.a = self.y;  self.set_zn(self.a); true } // TYA
            0xBA => { self.x = self.sp; self.set_zn(self.x); true } // TSX
            0x9A => { self.sp = self.x;                      true } // TXS (no flags)

            // Register increments / decrements
            0xE8 => { self.x = self.x.wrapping_add(1); self.set_zn(self.x); true } // INX
            0xCA => { self.x = self.x.wrapping_sub(1); self.set_zn(self.x); true } // DEX
            0xC8 => { self.y = self.y.wrapping_add(1); self.set_zn(self.y); true } // INY
            0x88 => { self.y = self.y.wrapping_sub(1); self.set_zn(self.y); true } // DEY

            // Stack opcodes
            0x48 => {
                // PHA: push accumulator
                self.push_byte(bus, self.a);
                true
            }
            0x68 => {
                // PLA: pull accumulator
                self.a = self.pop_byte(bus);
                self.set_zn(self.a);
                true
            }
            0x08 => {
                // PHP: B and unused bits always set when pushing
                self.push_byte(bus, self.p | Self::FLAG_BREAK | Self::FLAG_BREAK2);
                true
            }
            0x28 => {
                // PLP: B cleared, unused set when pulling
                self.p = (self.pop_byte(bus) & 0xCF) | 0x20;
                true
            }

            // Jump opcodes
            0x4C => {
                // JMP absolute
                self.pc = self.fetch_word(bus);
                true
            }
            0x6C => {
                // JMP indirect: hardware page-wrap bug, high byte wraps within same page
                let ptr = self.fetch_word(bus);
                let lo  = bus.read(ptr) as u16;
                let hi  = bus.read((ptr & 0xFF00) | ((ptr + 1) & 0x00FF)) as u16;
                self.pc = lo | (hi << 8);
                true
            }
            0x20 => {
                // JSR: push return address (PC - 1) then jump
                let target = self.fetch_word(bus);
                let ret    = self.pc.wrapping_sub(1);
                self.push_word(bus, ret);
                self.pc = target;
                true
            }
            0x60 => {
                // RTS: pull return address and add 1
                let ret = self.pop_word(bus);
                self.pc = ret.wrapping_add(1);
                true
            }
            0x40 => {
                // RTI: pull P then PC
                self.p  = (self.pop_byte(bus) & 0xCF) | 0x20;
                self.pc = self.pop_word(bus);
                true
            }
            // LAX: Load A and X from memory simultaneously (unofficial)
            0xA3 => {
                let addr = self.addr_indirect_x(bus);
                let val  = bus.read(addr);
                self.a   = val;
                self.x   = val;
                self.set_zn(val);
                true
            }
            0xA7 => {
                let addr = self.addr_zero_page(bus);
                let val  = bus.read(addr);
                self.a   = val;
                self.x   = val;
                self.set_zn(val);
                true
            }
            0xAF => {
                let addr = self.addr_absolute(bus);
                let val  = bus.read(addr);
                self.a   = val;
                self.x   = val;
                self.set_zn(val);
                true
            }
            0xB3 => {
                let addr = self.addr_indirect_y(bus);
                let val  = bus.read(addr);
                self.a   = val;
                self.x   = val;
                self.set_zn(val);
                true
            }
            0xB7 => {
                let addr = self.addr_zero_page_y(bus);
                let val  = bus.read(addr);
                self.a   = val;
                self.x   = val;
                self.set_zn(val);
                true
            }
            0xBF => {
                let addr = self.addr_absolute_y(bus);
                let val  = bus.read(addr);
                self.a   = val;
                self.x   = val;
                self.set_zn(val);
                true
            }
            // SAX: Store A AND X into memory (unofficial)
            0x83 => {
                let addr = self.addr_indirect_x(bus);
                bus.write(addr, self.a & self.x);
                true
            }
            0x87 => {
                let addr = self.addr_zero_page(bus);
                bus.write(addr, self.a & self.x);
                true
            }
            0x8F => {
                let addr = self.addr_absolute(bus);
                bus.write(addr, self.a & self.x);
                true
            }
            0x97 => {
                let addr = self.addr_zero_page_y(bus);
                bus.write(addr, self.a & self.x);
                true
            }
            // Unofficial SBC immediate (identical to 0xE9)
            0xEB => {
                let val = self.fetch_byte(bus);
                self.sbc(val);
                true
            }
            // DCP: DEC memory then CMP with A (unofficial)
            0xC3 => {
                let addr = self.addr_indirect_x(bus);
                let val  = bus.read(addr).wrapping_sub(1);
                bus.write(addr, val);
                self.compare(self.a, val);
                true
            }
            0xC7 => {
                let addr = self.addr_zero_page(bus);
                let val  = bus.read(addr).wrapping_sub(1);
                bus.write(addr, val);
                self.compare(self.a, val);
                true
            }
            0xCF => {
                let addr = self.addr_absolute(bus);
                let val  = bus.read(addr).wrapping_sub(1);
                bus.write(addr, val);
                self.compare(self.a, val);
                true
            }
            0xD3 => {
                let addr = self.addr_indirect_y(bus);
                let val  = bus.read(addr).wrapping_sub(1);
                bus.write(addr, val);
                self.compare(self.a, val);
                true
            }
            0xD7 => {
                let addr = self.addr_zero_page_x(bus);
                let val  = bus.read(addr).wrapping_sub(1);
                bus.write(addr, val);
                self.compare(self.a, val);
                true
            }
            0xDB => {
                let addr = self.addr_absolute_y(bus);
                let val  = bus.read(addr).wrapping_sub(1);
                bus.write(addr, val);
                self.compare(self.a, val);
                true
            }
            0xDF => {
                let addr = self.addr_absolute_x(bus);
                let val  = bus.read(addr).wrapping_sub(1);
                bus.write(addr, val);
                self.compare(self.a, val);
                true
            }
            // ISB: INC memory then SBC with A (unofficial)
            0xE3 => {
                let addr = self.addr_indirect_x(bus);
                let val  = bus.read(addr).wrapping_add(1);
                bus.write(addr, val);
                self.sbc(val);
                true
            }
            0xE7 => {
                let addr = self.addr_zero_page(bus);
                let val  = bus.read(addr).wrapping_add(1);
                bus.write(addr, val);
                self.sbc(val);
                true
            }
            0xEF => {
                let addr = self.addr_absolute(bus);
                let val  = bus.read(addr).wrapping_add(1);
                bus.write(addr, val);
                self.sbc(val);
                true
            }
            0xF3 => {
                let addr = self.addr_indirect_y(bus);
                let val  = bus.read(addr).wrapping_add(1);
                bus.write(addr, val);
                self.sbc(val);
                true
            }
            0xF7 => {
                let addr = self.addr_zero_page_x(bus);
                let val  = bus.read(addr).wrapping_add(1);
                bus.write(addr, val);
                self.sbc(val);
                true
            }
            0xFB => {
                let addr = self.addr_absolute_y(bus);
                let val  = bus.read(addr).wrapping_add(1);
                bus.write(addr, val);
                self.sbc(val);
                true
            }
            0xFF => {
                let addr = self.addr_absolute_x(bus);
                let val  = bus.read(addr).wrapping_add(1);
                bus.write(addr, val);
                self.sbc(val);
                true
            }
            // SLO: ASL memory then ORA with A (unofficial)
            0x03 => {
                let addr = self.addr_indirect_x(bus);
                let val  = self.asl(bus.read(addr));
                bus.write(addr, val);
                self.a |= val;
                self.set_zn(self.a);
                true
            }
            0x07 => {
                let addr = self.addr_zero_page(bus);
                let val  = self.asl(bus.read(addr));
                bus.write(addr, val);
                self.a |= val;
                self.set_zn(self.a);
                true
            }
            0x0F => {
                let addr = self.addr_absolute(bus);
                let val  = self.asl(bus.read(addr));
                bus.write(addr, val);
                self.a |= val;
                self.set_zn(self.a);
                true
            }
            0x13 => {
                let addr = self.addr_indirect_y(bus);
                let val  = self.asl(bus.read(addr));
                bus.write(addr, val);
                self.a |= val;
                self.set_zn(self.a);
                true
            }
            0x17 => {
                let addr = self.addr_zero_page_x(bus);
                let val  = self.asl(bus.read(addr));
                bus.write(addr, val);
                self.a |= val;
                self.set_zn(self.a);
                true
            }
            0x1B => {
                let addr = self.addr_absolute_y(bus);
                let val  = self.asl(bus.read(addr));
                bus.write(addr, val);
                self.a |= val;
                self.set_zn(self.a);
                true
            }
            0x1F => {
                let addr = self.addr_absolute_x(bus);
                let val  = self.asl(bus.read(addr));
                bus.write(addr, val);
                self.a |= val;
                self.set_zn(self.a);
                true
            }
            // RLA: ROL memory then AND with A (unofficial)
            0x23 => {
                let addr = self.addr_indirect_x(bus);
                let val  = self.rol(bus.read(addr));
                bus.write(addr, val);
                self.a &= val;
                self.set_zn(self.a);
                true
            }
            0x27 => {
                let addr = self.addr_zero_page(bus);
                let val  = self.rol(bus.read(addr));
                bus.write(addr, val);
                self.a &= val;
                self.set_zn(self.a);
                true
            }
            0x2F => {
                let addr = self.addr_absolute(bus);
                let val  = self.rol(bus.read(addr));
                bus.write(addr, val);
                self.a &= val;
                self.set_zn(self.a);
                true
            }
            0x33 => {
                let addr = self.addr_indirect_y(bus);
                let val  = self.rol(bus.read(addr));
                bus.write(addr, val);
                self.a &= val;
                self.set_zn(self.a);
                true
            }
            0x37 => {
                let addr = self.addr_zero_page_x(bus);
                let val  = self.rol(bus.read(addr));
                bus.write(addr, val);
                self.a &= val;
                self.set_zn(self.a);
                true
            }
            0x3B => {
                let addr = self.addr_absolute_y(bus);
                let val  = self.rol(bus.read(addr));
                bus.write(addr, val);
                self.a &= val;
                self.set_zn(self.a);
                true
            }
            0x3F => {
                let addr = self.addr_absolute_x(bus);
                let val  = self.rol(bus.read(addr));
                bus.write(addr, val);
                self.a &= val;
                self.set_zn(self.a);
                true
            }
            // SRE: LSR memory then EOR with A (unofficial)
            0x43 => {
                let addr = self.addr_indirect_x(bus);
                let val  = self.lsr(bus.read(addr));
                bus.write(addr, val);
                self.a ^= val;
                self.set_zn(self.a);
                true
            }
            0x47 => {
                let addr = self.addr_zero_page(bus);
                let val  = self.lsr(bus.read(addr));
                bus.write(addr, val);
                self.a ^= val;
                self.set_zn(self.a);
                true
            }
            0x4F => {
                let addr = self.addr_absolute(bus);
                let val  = self.lsr(bus.read(addr));
                bus.write(addr, val);
                self.a ^= val;
                self.set_zn(self.a);
                true
            }
            0x53 => {
                let addr = self.addr_indirect_y(bus);
                let val  = self.lsr(bus.read(addr));
                bus.write(addr, val);
                self.a ^= val;
                self.set_zn(self.a);
                true
            }
            0x57 => {
                let addr = self.addr_zero_page_x(bus);
                let val  = self.lsr(bus.read(addr));
                bus.write(addr, val);
                self.a ^= val;
                self.set_zn(self.a);
                true
            }
            0x5B => {
                let addr = self.addr_absolute_y(bus);
                let val  = self.lsr(bus.read(addr));
                bus.write(addr, val);
                self.a ^= val;
                self.set_zn(self.a);
                true
            }
            0x5F => {
                let addr = self.addr_absolute_x(bus);
                let val  = self.lsr(bus.read(addr));
                bus.write(addr, val);
                self.a ^= val;
                self.set_zn(self.a);
                true
            }
            // RRA: ROR memory then ADC with A (unofficial)
            0x63 => {
                let addr = self.addr_indirect_x(bus);
                let val  = self.ror(bus.read(addr));
                bus.write(addr, val);
                self.adc(val);
                true
            }
            0x67 => {
                let addr = self.addr_zero_page(bus);
                let val  = self.ror(bus.read(addr));
                bus.write(addr, val);
                self.adc(val);
                true
            }
            0x6F => {
                let addr = self.addr_absolute(bus);
                let val  = self.ror(bus.read(addr));
                bus.write(addr, val);
                self.adc(val);
                true
            }
            0x73 => {
                let addr = self.addr_indirect_y(bus);
                let val  = self.ror(bus.read(addr));
                bus.write(addr, val);
                self.adc(val);
                true
            }
            0x77 => {
                let addr = self.addr_zero_page_x(bus);
                let val  = self.ror(bus.read(addr));
                bus.write(addr, val);
                self.adc(val);
                true
            }
            0x7B => {
                let addr = self.addr_absolute_y(bus);
                let val  = self.ror(bus.read(addr));
                bus.write(addr, val);
                self.adc(val);
                true
            }
            0x7F => {
                let addr = self.addr_absolute_x(bus);
                let val  = self.ror(bus.read(addr));
                bus.write(addr, val);
                self.adc(val);
                true
            }
            0xEA => true, // NOP

            0x00 => false, // BRK: stop execution loop

            _ => {
                // Unofficial/undocumented opcodes — treat as NOP variants
                // Consume operand bytes so PC advances correctly
                match opcode {
                    // 2-byte unofficial NOPs (read and discard 1 byte)
                    0x04 | 0x44 | 0x64 |        // NOP zp
                    0x14 | 0x34 | 0x54 | 0x74 | 0xD4 | 0xF4 | // NOP zp,x
                    0x80 | 0x82 | 0x89 | 0xC2 | 0xE2 => {      // NOP imm
                        self.fetch_byte(bus);
                    }
                    // 3-byte unofficial NOPs (read and discard 2 bytes)
                    0x0C |                               // NOP abs
                    0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => { // NOP abs,x
                        self.fetch_word(bus);
                    }
                    // 1-byte unofficial NOPs (no operand)
                    0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xFA => {}
                    _ => {
                        panic!("Unknown opcode {:02X} at PC {:04X}", opcode, opcode_pc)
                    }
                }
                true
            }
        }
    }

    // Reset: load PC from reset vector at $FFFC/$FFFD
    pub fn reset(&mut self, bus: &mut Bus) {
        self.a  = 0;
        self.x  = 0;
        self.y  = 0;
        self.sp = 0xFD;
        self.p  = 0x24;
        self.pc = self.read_word(bus, 0xFFFC);
    }

    // Trigger NMI: push PC and P, jump to vector at $FFFA/$FFFB
    pub fn trigger_nmi(&mut self, bus: &mut Bus) {
        self.push_word(bus, self.pc);
        self.push_byte(bus, (self.p | Self::FLAG_BREAK2) & !Self::FLAG_BREAK);
        self.p |= Self::FLAG_INTERRUPT;
        self.pc = self.read_word(bus, 0xFFFA);
    }

    // Status flag bit positions
    const FLAG_CARRY:     u8 = 0b0000_0001;
    const FLAG_ZERO:      u8 = 0b0000_0010;
    const FLAG_INTERRUPT: u8 = 0b0000_0100;
    const FLAG_DECIMAL:   u8 = 0b0000_1000;
    const FLAG_BREAK:     u8 = 0b0001_0000;
    const FLAG_BREAK2:    u8 = 0b0010_0000;
    const FLAG_OVERFLOW:  u8 = 0b0100_0000;
    const FLAG_NEG:       u8 = 0b1000_0000;

    // Compare function for CMP, CPX, CPY
    fn compare(&mut self, reg: u8, val: u8) {
        let res = reg.wrapping_sub(val);
        if reg >= val { self.p |= Self::FLAG_CARRY; }
        else          { self.p &= !Self::FLAG_CARRY; }
        self.set_zn(res);
    }

    // Subtract with Carry: invert operand then add
    fn sbc(&mut self, data: u8) {
        self.adc(!data);
    }

    // Add with Carry
    fn adc(&mut self, data: u8) {
        let a   = self.a as u16;
        let b   = data as u16;
        let c   = if (self.p & Self::FLAG_CARRY) != 0 { 1 } else { 0 };
        let sum = a + b + c;
        let res = sum as u8;

        if sum > 0xFF                              { self.p |= Self::FLAG_CARRY; }
        else                                       { self.p &= !Self::FLAG_CARRY; }
        if (self.a ^ res) & (data ^ res) & 0x80 != 0 { self.p |= Self::FLAG_OVERFLOW; }
        else                                       { self.p &= !Self::FLAG_OVERFLOW; }

        self.a = res;
        self.set_zn(self.a);
    }

    // Old bit 7 goes to carry, shift left, bit 0 becomes 0
    fn asl(&mut self, val: u8) -> u8 {
        if val & 0x80 != 0 { self.p |= Self::FLAG_CARRY; }
        else               { self.p &= !Self::FLAG_CARRY; }
        let res = val << 1;
        self.set_zn(res);
        res
    }

    // Old bit 0 goes to carry, shift right, bit 7 becomes 0
    fn lsr(&mut self, val: u8) -> u8 {
        if val & 0x01 != 0 { self.p |= Self::FLAG_CARRY; }
        else               { self.p &= !Self::FLAG_CARRY; }
        let res = val >> 1;
        self.set_zn(res);
        res
    }

    // Rotate left through carry
    fn rol(&mut self, val: u8) -> u8 {
        let old_c = (self.p & Self::FLAG_CARRY) != 0;
        if val & 0x80 != 0 { self.p |= Self::FLAG_CARRY; }
        else               { self.p &= !Self::FLAG_CARRY; }
        let res = (val << 1) | (old_c as u8);
        self.set_zn(res);
        res
    }

    // Rotate right through carry
    fn ror(&mut self, val: u8) -> u8 {
        let old_c = (self.p & Self::FLAG_CARRY) != 0;
        if val & 0x01 != 0 { self.p |= Self::FLAG_CARRY; }
        else               { self.p &= !Self::FLAG_CARRY; }
        let res = (val >> 1) | ((old_c as u8) << 7);
        self.set_zn(res);
        res
    }

    // BIT: Z = !(A & val), N = val bit 7, V = val bit 6
    fn bit(&mut self, val: u8) {
        if self.a & val == 0 { self.p |= Self::FLAG_ZERO; }
        else                 { self.p &= !Self::FLAG_ZERO; }
        if val & 0x80 != 0   { self.p |= Self::FLAG_NEG; }
        else                 { self.p &= !Self::FLAG_NEG; }
        if val & 0x40 != 0   { self.p |= Self::FLAG_OVERFLOW; }
        else                 { self.p &= !Self::FLAG_OVERFLOW; }
    }

    // Helper for all branching instructions
    fn branch(&mut self, bus: &mut Bus, condition: bool) {
        let offset = self.fetch_byte(bus) as i8;
        if condition {
            self.pc = self.pc.wrapping_add_signed(offset as i16);
        }
    }

    // Update Z and N flags based on value
    fn set_zn(&mut self, val: u8) {
        if val == 0          { self.p |= Self::FLAG_ZERO; }
        else                 { self.p &= !Self::FLAG_ZERO; }
        if (val & 0x80) != 0 { self.p |= Self::FLAG_NEG; }
        else                 { self.p &= !Self::FLAG_NEG; }
    }
}