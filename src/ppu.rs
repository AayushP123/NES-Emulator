// Screen dimensions in pixels
pub const SCREEN_WIDTH: usize  = 256;
pub const SCREEN_HEIGHT: usize = 240;

// Standard NES 64-colour system palette (ARGB 0xAARRGGBB)
#[rustfmt::skip]
const NES_PALETTE: [u32; 64] = [
    0xFF626262, 0xFF001FB2, 0xFF2404C8, 0xFF5200B2,
    0xFF730076, 0xFF800024, 0xFF730B00, 0xFF522800,
    0xFF244400, 0xFF005700, 0xFF005C00, 0xFF005324,
    0xFF003C76, 0xFF000000, 0xFF000000, 0xFF000000,
    0xFFABABAB, 0xFF0D57FF, 0xFF4B30FF, 0xFF8A13FF,
    0xFFBC08D6, 0xFFD21269, 0xFFC72E00, 0xFF9D5400,
    0xFF607B00, 0xFF209800, 0xFF00A300, 0xFF009942,
    0xFF007DB4, 0xFF000000, 0xFF000000, 0xFF000000,
    0xFFFFFFFF, 0xFF53AEFF, 0xFF9085FF, 0xFFD365FF,
    0xFFFF57FF, 0xFFFF5DCF, 0xFFFF7757, 0xFFFA9E00,
    0xFFBDC700, 0xFF7AE700, 0xFF43F611, 0xFF26EF7E,
    0xFF2CD5F6, 0xFF4E4E4E, 0xFF000000, 0xFF000000,
    0xFFFFFFFF, 0xFFB6DEFB, 0xFFC9CAFF, 0xFFE2C3FF,
    0xFFF8C0FF, 0xFFFEC0E7, 0xFFFECCC5, 0xFFF7D8A5,
    0xFFE4E594, 0xFFCFEF96, 0xFFBDF4AB, 0xFFB3F3CC,
    0xFFB5EBF2, 0xFFB8B8B8, 0xFF000000, 0xFF000000,
];

// Nametable mirroring modes
#[derive(Clone, Copy, PartialEq)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    SingleScreenA,
    SingleScreenB,
}

pub struct Ppu {
    // CHR ROM/RAM: pattern tables (8 KB)
    pub chr: [u8; 8192],
    // Nametable RAM (2 KB, mirrored internally to 4 KB)
    vram: [u8; 2048],
    // Palette RAM (32 bytes)
    palette: [u8; 32],
    // Primary OAM: 64 sprites x 4 bytes
    pub oam: [u8; 256],
    // Secondary OAM: up to 8 sprites for the current scanline
    secondary_oam: [u8; 32],

    // CPU-facing registers ($2000-$2007)
    ctrl:     u8, // $2000 PPUCTRL
    mask:     u8, // $2001 PPUMASK
    status:   u8, // $2002 PPUSTATUS
    oam_addr: u8, // $2003 OAMADDR

    // Loopy internal registers
    // Current VRAM address (15-bit): yyy NN YYYYY XXXXX
    //   bits 4-0:  coarse X
    //   bits 9-5:  coarse Y
    //   bits 11-10: nametable select
    //   bits 14-12: fine Y
    v: u16,
    // Temporary VRAM address (same layout as v)
    t: u16,
    // Fine X scroll (3-bit)
    x: u8,
    // First/second write toggle for $2005/$2006
    w: bool,
    // Read buffer for $2007
    data_buf: u8,

    // Timing
    pub scanline:  i16, // -1 (pre-render) to 260
    pub dot:       u16, // 0 to 340
    pub frame:     u64,
    odd_frame:     bool,

    // Interrupts
    pub nmi_pending: bool,

    // Background shift registers (16-bit, MSB first)
    bg_shift_lo: u16,
    bg_shift_hi: u16,
    // Single-bit attribute latches, replicated each cycle into the attr shifters
    at_latch_lo: bool,
    at_latch_hi: bool,
    // 8-bit attribute shift registers: LSB fed from the latches each cycle
    at_shift_lo: u8,
    at_shift_hi: u8,

    // Tile-fetch latches (filled over 8-dot pipeline)
    nt_latch:    u8,
    at_latch:    u8, // 2-bit attribute for the tile being fetched
    bg_lo_latch: u8,
    bg_hi_latch: u8,

    // Sprite data for the current scanline
    sprite_count:    usize,
    sprite_shift_lo: [u8; 8],
    sprite_shift_hi: [u8; 8],
    sprite_attr:     [u8; 8],
    sprite_x:        [u8; 8],
    // True when sprite 0 is in secondary OAM for this scanline
    sprite0_on_line: bool,

    // ARGB pixels, row-major, 256 x 240
    pub framebuffer: Vec<u32>,

    pub mirroring: Mirroring,
}

impl Ppu {
    pub fn new(mirroring: Mirroring) -> Self {
        Ppu {
            chr:           [0; 8192],
            vram:          [0; 2048],
            palette:       [0; 32],
            oam:           [0; 256],
            secondary_oam: [0xFF; 32],
            ctrl:     0,
            mask:     0,
            status:   0,
            oam_addr: 0,
            v: 0, t: 0, x: 0, w: false,
            data_buf: 0,
            scanline:  -1,
            dot:       0,
            frame:     0,
            odd_frame: false,
            nmi_pending: false,
            bg_shift_lo: 0,
            bg_shift_hi: 0,
            at_latch_lo: false,
            at_latch_hi: false,
            at_shift_lo: 0,
            at_shift_hi: 0,
            nt_latch: 0, at_latch: 0, bg_lo_latch: 0, bg_hi_latch: 0,
            sprite_count:    0,
            sprite_shift_lo: [0; 8],
            sprite_shift_hi: [0; 8],
            sprite_attr:     [0; 8],
            sprite_x:        [0xFF; 8],
            sprite0_on_line: false,
            framebuffer: vec![0u32; SCREEN_WIDTH * SCREEN_HEIGHT],
            mirroring,
        }
    }

    // Read from CPU address in $2000-$2007 range
    pub fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr & 0x7 {
            0x2 => {
                // PPUSTATUS: return top 3 bits + stale data_buf low 5, then clear vblank + w
                let s = (self.status & 0xE0) | (self.data_buf & 0x1F);
                self.status &= !0x80;
                self.w = false;
                s
            }
            0x4 => self.oam[self.oam_addr as usize], // OAMDATA
            0x7 => {
                // PPUDATA: reads are buffered except palette
                let v = self.v & 0x3FFF;
                let prev = self.data_buf;
                self.data_buf = self.ppu_read(v);
                let result = if v >= 0x3F00 { self.data_buf } else { prev };
                let inc = if self.ctrl & 0x04 != 0 { 32 } else { 1 };
                self.v = (self.v + inc) & 0x7FFF;
                result
            }
            _ => 0,
        }
    }

    // Write to CPU address in $2000-$2007 range
    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr & 0x7 {
            0x0 => {
                // PPUCTRL: also update t nametable bits
                self.ctrl = data;
                self.t = (self.t & 0xF3FF) | ((data as u16 & 0x03) << 10);
            }
            0x1 => { self.mask = data; }
            0x3 => { self.oam_addr = data; }
            0x4 => {
                self.oam[self.oam_addr as usize] = data;
                self.oam_addr = self.oam_addr.wrapping_add(1);
            }
            0x5 => {
                // PPUSCROLL (two writes)
                if !self.w {
                    // First write: coarse X into t[4:0], fine X into x
                    self.t = (self.t & 0xFFE0) | ((data as u16) >> 3);
                    self.x =  data & 0x07;
                } else {
                    // Second write: coarse Y into t[9:5], fine Y into t[14:12]
                    self.t = (self.t & 0x8FFF) | (((data as u16) & 0x07) << 12);
                    self.t = (self.t & 0xFC1F) | (((data as u16) >> 3) << 5);
                }
                self.w = !self.w;
            }
            0x6 => {
                // PPUADDR (two writes)
                if !self.w {
                    // High byte: t[13:8] = data[5:0], clear t[14]
                    self.t = (self.t & 0x80FF) | (((data as u16) & 0x3F) << 8);
                } else {
                    // Low byte: t[7:0] = data, then v = t
                    self.t = (self.t & 0xFF00) | data as u16;
                    self.v =  self.t;
                }
                self.w = !self.w;
            }
            0x7 => {
                // PPUDATA
                self.ppu_write(self.v & 0x3FFF, data);
                let inc = if self.ctrl & 0x04 != 0 { 32 } else { 1 };
                self.v = (self.v + inc) & 0x7FFF;
            }
            _ => {}
        }
    }

    // OAM DMA: called when CPU writes to $4014
    // page is a 256-byte slice from CPU memory at address (data << 8)
    pub fn oam_dma(&mut self, page: &[u8; 256]) {
        for (i, &byte) in page.iter().enumerate() {
            self.oam[(self.oam_addr as usize + i) & 0xFF] = byte;
        }
    }

    // Advance one PPU clock (~1/3 CPU cycle)
    // Returns true once per completed frame (end of post-render scanline 260)
    pub fn tick(&mut self) -> bool {
        let rendering = self.mask & 0x18 != 0;
        let mut frame_done = false;

        match self.scanline {
            // Pre-render scanline
            -1 => {
                if self.dot == 1 {
                    // Clear VBlank, sprite-0 hit, sprite overflow
                    self.status &= !0xE0;
                    self.nmi_pending = false;
                }
                if rendering {
                    if self.dot >= 280 && self.dot <= 304 {
                        self.copy_v_vert();
                    }
                    self.bg_fetch_and_shift();
                }
            }

            // Visible scanlines
            0..=239 => {
                if rendering {
                    self.bg_fetch_and_shift();
                    // Sprite evaluation happens mid-scanline but we batch it at dot 257
                    if self.dot == 257 {
                        self.evaluate_sprites();
                    }
                }
                // Output pixel during dots 1-256
                if self.dot >= 1 && self.dot <= 256 {
                    self.render_pixel();
                }
            }

            // Post-render (scanline 240): PPU idles

            // VBlank start
            241 => {
                if self.dot == 1 {
                    self.status |= 0x80; // set VBlank flag
                    if self.ctrl & 0x80 != 0 {
                        self.nmi_pending = true; // trigger NMI
                    }
                }
            }
            _ => {}
        }

        // Advance dot / scanline
        self.dot += 1;
        if self.dot > 340 {
            self.dot = 0;
            self.scanline += 1;

            if self.scanline > 260 {
                self.scanline = -1;
                self.frame += 1;
                frame_done = true;
                self.odd_frame = !self.odd_frame;
                // Odd frames skip dot 0 of the pre-render scanline
                if self.odd_frame && rendering {
                    self.dot = 1;
                }
            }
        }

        frame_done
    }

    // Handles BG shift + fetch for the current dot on visible/pre-render lines
    fn bg_fetch_and_shift(&mut self) {
        // Shift registers clock every dot in the active window
        let active = (self.dot >= 1 && self.dot <= 256) || (self.dot >= 321 && self.dot <= 336);
        if active {
            self.shift_bg();
        }

        // The PPU runs an 8-cycle fetch pipeline at dots 1-256 and 321-336
        if active || self.dot == 338 || self.dot == 340 {
            match self.dot & 0x7 {
                1 => {
                    // Reload shift registers from latches filled in the previous 8 cycles
                    self.bg_shift_lo = (self.bg_shift_lo & 0xFF00) | self.bg_lo_latch as u16;
                    self.bg_shift_hi = (self.bg_shift_hi & 0xFF00) | self.bg_hi_latch as u16;
                    self.at_latch_lo  = self.at_latch & 0x01 != 0;
                    self.at_latch_hi  = self.at_latch & 0x02 != 0;
                    // Fetch nametable byte for next tile
                    let nt_addr = 0x2000 | (self.v & 0x0FFF);
                    self.nt_latch = self.ppu_read(nt_addr);
                }
                3 => {
                    // Fetch attribute byte
                    let at_addr = 0x23C0
                        | (self.v & 0x0C00)
                        | ((self.v >> 4) & 0x38)
                        | ((self.v >> 2) & 0x07);
                    let at_byte = self.ppu_read(at_addr);
                    // Pick the 2-bit palette for the current tile quadrant
                    let shift = ((self.v >> 4) & 0x04) | (self.v & 0x02);
                    self.at_latch = (at_byte >> shift) & 0x03;
                }
                5 => {
                    // Fetch low pattern byte
                    let fine_y = (self.v >> 12) & 0x07;
                    let base   = if self.ctrl & 0x10 != 0 { 0x1000u16 } else { 0x0000 };
                    self.bg_lo_latch = self.ppu_read(base + self.nt_latch as u16 * 16 + fine_y);
                }
                7 => {
                    // Fetch high pattern byte, then increment coarse X
                    let fine_y = (self.v >> 12) & 0x07;
                    let base   = if self.ctrl & 0x10 != 0 { 0x1000u16 } else { 0x0000 };
                    self.bg_hi_latch = self.ppu_read(base + self.nt_latch as u16 * 16 + fine_y + 8);
                    self.inc_v_x();
                }
                _ => {}
            }
        }

        if self.dot == 256 { self.inc_v_y(); }       // end of line: Y++
        if self.dot == 257 { self.copy_v_horiz(); }  // reload horizontal scroll
        // Sprite tiles fetched during dots 257-320, batched at dot 260
        if self.dot == 260 && self.scanline >= 0 && self.scanline < 240 {
            self.load_sprite_tiles();
        }
    }

    // Shift all background and attribute registers left by one
    fn shift_bg(&mut self) {
        self.bg_shift_lo <<= 1;
        self.bg_shift_hi <<= 1;
        self.at_shift_lo  = (self.at_shift_lo << 1) | self.at_latch_lo as u8;
        self.at_shift_hi  = (self.at_shift_hi << 1) | self.at_latch_hi as u8;
    }

    // Increment coarse X, wrapping into the adjacent horizontal nametable
    fn inc_v_x(&mut self) {
        if (self.v & 0x001F) == 31 {
            self.v &= !0x001F;
            self.v ^=  0x0400; // flip horizontal nametable
        } else {
            self.v += 1;
        }
    }

    // Increment fine Y, wrapping coarse Y and flipping vertical nametable when needed
    fn inc_v_y(&mut self) {
        if (self.v & 0x7000) != 0x7000 {
            self.v += 0x1000; // fine Y++
        } else {
            self.v &= !0x7000; // fine Y = 0
            let mut coarse_y = (self.v & 0x03E0) >> 5;
            if coarse_y == 29 {
                coarse_y = 0;
                self.v ^= 0x0800; // flip vertical nametable
            } else if coarse_y == 31 {
                coarse_y = 0; // wrap without flipping (out-of-range behaviour)
            } else {
                coarse_y += 1;
            }
            self.v = (self.v & !0x03E0) | (coarse_y << 5);
        }
    }

    // Copy horizontal bits from t into v
    fn copy_v_horiz(&mut self) {
        // v: ....A.. ...BCDEF = t: ....A.. ...BCDEF
        self.v = (self.v & 0xFBE0) | (self.t & 0x041F);
    }

    // Copy vertical bits from t into v (dots 280-304 of pre-render)
    fn copy_v_vert(&mut self) {
        // v: .IHGF.ED CBA..... = t: .IHGF.ED CBA.....
        self.v = (self.v & 0x841F) | (self.t & 0x7BE0);
    }

    // Sprite evaluation: fill secondary OAM with sprites visible on scanline+1
    fn evaluate_sprites(&mut self) {
        let next = self.scanline + 1;
        let height = if self.ctrl & 0x20 != 0 { 16i16 } else { 8i16 };
        let mut count = 0usize;
        self.sprite0_on_line = false;
        self.secondary_oam   = [0xFF; 32];

        for i in 0..64usize {
            // Sprite Y: sprite appears starting one scanline AFTER its Y value
            let y   = self.oam[i * 4] as i16;
            let row = next - y - 1;
            if row >= 0 && row < height {
                if count < 8 {
                    if i == 0 { self.sprite0_on_line = true; }
                    let dst = count * 4;
                    self.secondary_oam[dst..dst + 4]
                        .copy_from_slice(&self.oam[i * 4..i * 4 + 4]);
                    count += 1;
                } else {
                    self.status |= 0x20; // sprite overflow
                    break;
                }
            }
        }
        self.sprite_count = count;
    }

    // Fetch pattern tiles for sprites in secondary OAM
    // Called once per scanline at dot 260 in our simplified model
    fn load_sprite_tiles(&mut self) {
        let height = if self.ctrl & 0x20 != 0 { 16i16 } else { 8i16 };

        for i in 0..self.sprite_count {
            let y    = self.secondary_oam[i * 4]     as i16;
            let tile = self.secondary_oam[i * 4 + 1] as u16;
            let attr = self.secondary_oam[i * 4 + 2];
            let x    = self.secondary_oam[i * 4 + 3];

            let mut row = self.scanline - y - 1; // row within the sprite (0-7 or 0-15)
            let flip_v  = attr & 0x80 != 0;
            let flip_h  = attr & 0x40 != 0;

            if flip_v { row = height - 1 - row; }

            let (addr_lo, addr_hi) = if height == 8 {
                // 8x8: PPUCTRL bit 3 selects pattern table
                let base = if self.ctrl & 0x08 != 0 { 0x1000u16 } else { 0x0000 };
                let a = base + tile * 16 + row as u16;
                (a, a + 8)
            } else {
                // 8x16: tile bit 0 selects pattern table, top/bottom halves in consecutive tiles
                let base      = if tile & 1 != 0 { 0x1000u16 } else { 0x0000 };
                let tile_base = tile & 0xFE;
                let half_off  = if row >= 8 { 16u16 } else { 0 };
                let a = base + tile_base * 16 + half_off + (row & 7) as u16;
                (a, a + 8)
            };

            let mut lo = self.ppu_read(addr_lo);
            let mut hi = self.ppu_read(addr_hi);

            if flip_h {
                lo = lo.reverse_bits();
                hi = hi.reverse_bits();
            }

            self.sprite_shift_lo[i] = lo;
            self.sprite_shift_hi[i] = hi;
            self.sprite_attr[i]     = attr;
            self.sprite_x[i]        = x;
        }
        // Zero out unused slots so they never match
        for i in self.sprite_count..8 {
            self.sprite_shift_lo[i] = 0;
            self.sprite_shift_hi[i] = 0;
            self.sprite_attr[i]     = 0;
            self.sprite_x[i]        = 0xFF;
        }
    }

    // Composite background and sprite pixels and write to the framebuffer
    fn render_pixel(&mut self) {
        // dot 1 = pixel column 0, so subtract 1
        let col = (self.dot - 1) as usize;
        let row = self.scanline as usize;

        let show_bg = self.mask & 0x08 != 0;
        let show_sp = self.mask & 0x10 != 0;
        let clip_bg = self.mask & 0x02 == 0; // hide leftmost 8px of BG
        let clip_sp = self.mask & 0x04 == 0; // hide leftmost 8px of sprites

        // Background pixel
        let (bg_pix, bg_pal) = if show_bg && !(clip_bg && col < 8) {
            let mux = 0x8000u16 >> self.x;
            let lo  = ((self.bg_shift_lo & mux) != 0) as u8;
            let hi  = ((self.bg_shift_hi & mux) != 0) as u8;
            let pix = lo | (hi << 1);
            let m8  = 0x80u8 >> self.x;
            let al  = ((self.at_shift_lo & m8) != 0) as u8;
            let ah  = ((self.at_shift_hi & m8) != 0) as u8;
            (pix, al | (ah << 1))
        } else {
            (0, 0)
        };

        // Sprite pixel: iterate front-to-back, first non-transparent wins
        let (sp_pix, sp_pal, sp_front, sp_zero) =
            if show_sp && !(clip_sp && col < 8) {
                let mut result = (0u8, 0u8, false, false);
                for i in 0..self.sprite_count {
                    let sx = self.sprite_x[i] as usize;
                    if col < sx || col >= sx + 8 { continue; }
                    let bit = 7 - (col - sx);
                    let lo  = (self.sprite_shift_lo[i] >> bit) & 1;
                    let hi  = (self.sprite_shift_hi[i] >> bit) & 1;
                    let pix = lo | (hi << 1);
                    if pix != 0 {
                        result = (
                            pix,
                            (self.sprite_attr[i] & 0x03) + 4, // palettes 4-7 for sprites
                            self.sprite_attr[i] & 0x20 == 0,  // priority: 0 = in front of BG
                            i == 0 && self.sprite0_on_line,
                        );
                        break;
                    }
                }
                result
            } else {
                (0, 0, false, false)
            };

        // Sprite-0 hit: both BG and sprite-0 are opaque on this dot
        if sp_zero && bg_pix != 0 && sp_pix != 0 && col < 255 {
            self.status |= 0x40;
        }

        // Palette lookup: pick address based on what pixels are opaque and priority
        let palette_addr: u8 = match (bg_pix, sp_pix) {
            (0, 0) => 0,                                // universal background
            (0, _) => sp_pal * 4 + sp_pix,             // only sprite visible
            (_, 0) => bg_pal * 4 + bg_pix,             // only BG visible
            (_, _) => if sp_front { sp_pal * 4 + sp_pix } else { bg_pal * 4 + bg_pix },
        };

        let colour_idx = (self.palette_read(palette_addr) & 0x3F) as usize;
        let colour     = NES_PALETTE[colour_idx];

        if col < SCREEN_WIDTH && row < SCREEN_HEIGHT {
            self.framebuffer[row * SCREEN_WIDTH + col] = colour;
        }
    }

    // PPU memory map read
    fn ppu_read(&self, addr: u16) -> u8 {
        let addr = addr & 0x3FFF;
        match addr {
            0x0000..=0x1FFF => self.chr[addr as usize],
            0x2000..=0x3EFF => self.vram[self.mirror_nt(addr)],
            0x3F00..=0x3FFF => self.palette_read(addr as u8 & 0x1F),
            _               => 0,
        }
    }

    // PPU memory map write
    fn ppu_write(&mut self, addr: u16, data: u8) {
        let addr = addr & 0x3FFF;
        match addr {
            0x0000..=0x1FFF => self.chr[addr as usize] = data,
            0x2000..=0x3EFF => {
                let idx = self.mirror_nt(addr);
                self.vram[idx] = data;
            }
            0x3F00..=0x3FFF => {
                let mut idx = addr as u8 & 0x1F;
                // Mirror sprite palette backgrounds onto universal background
                if matches!(idx, 0x10 | 0x14 | 0x18 | 0x1C) { idx &= 0x0F; }
                self.palette[idx as usize] = data;
            }
            _ => {}
        }
    }

    // Read palette RAM with sprite-background mirroring applied
    fn palette_read(&self, idx: u8) -> u8 {
        let mut i = idx & 0x1F;
        if matches!(i, 0x10 | 0x14 | 0x18 | 0x1C) { i &= 0x0F; }
        self.palette[i as usize]
    }

    // Map a PPU nametable address ($2000-$2FFF) to a 0-2047 VRAM index
    fn mirror_nt(&self, addr: u16) -> usize {
        let addr   = (addr & 0x2FFF) as usize - 0x2000; // 0x000-0xFFF
        let table  = addr / 0x400;   // which of the 4 logical nametables (0-3)
        let offset = addr % 0x400;
        let mapped = match self.mirroring {
            Mirroring::Horizontal    => if table < 2 { 0 } else { 1 },
            Mirroring::Vertical      => table & 1,
            Mirroring::SingleScreenA => 0,
            Mirroring::SingleScreenB => 1,
        };
        mapped * 0x400 + offset
    }
}