// Cartridge: parses iNES header and holds PRG/CHR ROM
// Supports Mapper 0 (NROM) only for now

pub struct Cart {
    pub prg: Vec<u8>,              // Program ROM (16 KB or 32 KB)
    pub chr: Vec<u8>,              // Character ROM (8 KB); allocated as RAM if absent
    pub mapper: u8,
    pub mirroring: crate::ppu::Mirroring,
}

impl Cart {
    // Parse raw .nes file bytes into a Cart
    pub fn from_bytes(data: &[u8]) -> Self {
        // iNES header is 16 bytes: NES\x1A + sizes + flags
        assert!(&data[0..4] == b"NES\x1a", "Not a valid iNES file");

        let prg_banks = data[4] as usize; // 16 KB units
        let chr_banks = data[5] as usize; // 8 KB units
        let flags6    = data[6];
        let flags7    = data[7];

        let mapper = (flags7 & 0xF0) | (flags6 >> 4);

        let mirroring = if flags6 & 0x08 != 0 {
            crate::ppu::Mirroring::SingleScreenA // four-screen: treat as single for now
        } else if flags6 & 0x01 != 0 {
            crate::ppu::Mirroring::Vertical
        } else {
            crate::ppu::Mirroring::Horizontal
        };

        // Trainer is 512 bytes if bit 2 of flags6 is set; skip it
        let trainer_offset = if flags6 & 0x04 != 0 { 512 } else { 0 };
        let prg_start = 16 + trainer_offset;
        let prg_size  = prg_banks * 16384;
        let chr_start = prg_start + prg_size;

        let prg = data[prg_start..prg_start + prg_size].to_vec();

        // If no CHR ROM banks, allocate 8 KB of CHR RAM instead
        let chr = if chr_banks > 0 {
            data[chr_start..chr_start + chr_banks * 8192].to_vec()
        } else {
            vec![0u8; 8192]
        };

        assert!(mapper == 0, "Only Mapper 0 (NROM) is supported right now");

        Cart { prg, chr, mapper, mirroring }
    }

    // CPU reads from cartridge address space ($8000-$FFFF)
    // NROM-128 (16 KB): mirrors the single bank at both $8000 and $C000
    // NROM-256 (32 KB): maps directly with no mirroring needed
    pub fn prg_read(&self, addr: u16) -> u8 {
        let offset = (addr - 0x8000) as usize % self.prg.len();
        self.prg[offset]
    }

    // CPU writes to PRG space (ignored on NROM, hook for future mappers)
    pub fn prg_write(&mut self, _addr: u16, _data: u8) {}
}