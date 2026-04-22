// Bus: connects the CPU to all hardware (RAM, PPU registers, cartridge)
// Every CPU read/write routes through here instead of a flat mem_buffer

use crate::cart::Cart;
use crate::ppu::Ppu;

pub struct Bus {
    // 2 KB internal CPU RAM, mirrored 4x across $0000-$1FFF
    ram: [u8; 2048],
    pub ppu:  Ppu,
    pub cart: Cart,

    // OAM DMA state
    dma_page:   u8,
    dma_active: bool,
    dma_cycle:  u16, // counts 0-511 (256 read + 256 write cycles)
}

impl Bus {
    pub fn new(cart: Cart) -> Self {
        let mirroring = cart.mirroring;
        let mut bus = Bus {
            ram:        [0; 2048],
            ppu:        Ppu::new(mirroring),
            cart,
            dma_page:   0,
            dma_active: false,
            dma_cycle:  0,
        };
        // Load CHR data into PPU pattern table memory
        let len = bus.cart.chr.len().min(8192);
        bus.ppu.chr[..len].copy_from_slice(&bus.cart.chr[..len]);
        bus
    }

    // CPU memory read: routes address to the correct device
    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            // Internal RAM with mirroring ($0000-$07FF repeated 4x)
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize],

            // PPU registers ($2000-$2007), mirrored every 8 bytes through $3FFF
            0x2000..=0x3FFF => self.ppu.cpu_read(addr & 0x2007),

            // APU and IO registers (stubbed out, return open bus)
            0x4000..=0x4013 => 0,
            0x4015           => 0, // APU status stub
            0x4016           => 0, // Controller 1 stub
            0x4017           => 0, // Controller 2 / APU frame counter stub
            0x4018..=0x7FFF  => 0, // Expansion / unused

            // Cartridge PRG ROM
            0x8000..=0xFFFF => self.cart.prg_read(addr),

            _               => 0,
        }
    }

    // CPU memory write: routes address to the correct device
    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize] = data,
            0x2000..=0x3FFF => self.ppu.cpu_write(addr & 0x2007, data),

            // OAM DMA: kicks off a 512-cycle transfer of 256 bytes into OAM
            0x4014 => {
                self.dma_page   = data;
                self.dma_active = true;
                self.dma_cycle  = 0;
            }

            0x4016 => {} // Controller strobe stub
            0x8000..=0xFFFF => self.cart.prg_write(addr, data),
            _      => {}
        }
    }

    // Tick the PPU one cycle; returns true when a full frame is complete
    pub fn ppu_tick(&mut self) -> bool {
        self.ppu.tick()
    }

    // Check and clear the NMI signal the PPU raises at VBlank
    pub fn poll_nmi(&mut self) -> bool {
        if self.ppu.nmi_pending {
            self.ppu.nmi_pending = false;
            true
        } else {
            false
        }
    }

    pub fn dma_active(&self) -> bool {
        self.dma_active
    }

    // One DMA clock tick: on even cycles read from CPU page, on odd write to OAM
    pub fn dma_tick(&mut self) {
        if self.dma_cycle & 1 == 1 {
            let oam_idx  = (self.dma_cycle >> 1) as usize;
            let src_addr = ((self.dma_page as u16) << 8) | oam_idx as u16;
            // Read directly from RAM (DMA bypasses the bus mapper)
            let byte = self.ram[(src_addr & 0x07FF) as usize];
            self.ppu.oam[oam_idx] = byte;
        }
        self.dma_cycle += 1;
        if self.dma_cycle >= 512 {
            self.dma_active = false;
            self.dma_cycle  = 0;
        }
    }
}