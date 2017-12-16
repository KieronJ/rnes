extern crate sdl2;

use nes::mapper::Mapper;
use nes::rom::MirrorMode;

pub const PPU_START: usize = 0x2000;
pub const PPU_END: usize = 0x2007;

pub const PPU_CTRL: usize = 0x2000;
pub const PPU_MASK: usize = 0x2001;
pub const PPU_STATUS: usize = 0x2002;
pub const PPU_OAMADDR: usize = 0x2003;
pub const PPU_OAMDATA: usize = 0x2004;
pub const PPU_SCROLL: usize = 0x2005;
pub const PPU_ADDR: usize = 0x2006;
pub const PPU_DATA: usize = 0x2007;
pub const PPU_OAMDMA: usize = 0x4014;

static PALETTE: [u8; 192] = [
    84, 84, 84,         0, 30, 116,         8, 16, 144,         48, 0, 136,         68, 0, 100,         92, 0, 48,          84, 4, 0,       60, 24, 0,
    32, 42, 0,          8, 58, 0,           0, 64, 0,           0, 60, 0,           0, 50, 60,          0, 0, 0,            0, 0, 0,        0, 0, 0,
    152, 150, 152,      8, 76, 196,         48, 50, 236,        92, 30, 228,        136, 20, 176,       160, 20, 100,       152,34, 32,     120, 60, 0,
    84, 90, 0,          40, 114, 0,         8, 124, 0,          0, 118, 40,         0, 102, 120,        0, 0, 0,            0, 0, 0,        0, 0, 0,
    236, 238, 236,      76, 154, 236,       120, 124, 236,      176, 98, 236,       228, 84, 236,       236, 88, 180,       236, 106, 100,  212, 136, 32,
    160, 170, 0,        116, 196, 0,        76, 208, 32,        56, 204, 108,       56, 180, 204,       60, 60, 60,         0, 0, 0,        0, 0, 0,
    236, 238, 236,      168, 204, 236,      188, 188, 236,      212, 178, 236,      236, 174, 236,      236, 174, 212,      236, 180, 176,  228, 196, 144,
    204, 210, 120,      180, 222, 120,      168, 226, 144,      152, 226, 180,      160, 214, 228,      160, 162, 160,      0, 0, 0,        0, 0, 0
];

pub struct Ricoh2C02 {
    mapper: Box<Mapper+Send>,

    framebuffer: Box<[u8]>,

    nametable_0: Box<[u8]>,
    nametable_1: Box<[u8]>,
    nametable_2: Box<[u8]>,
    nametable_3: Box<[u8]>,

    palette: Box<[u8]>,

    scanline: usize,
    cycle: usize,
    odd: bool,

    latch: u8,
    read_buffer: u8,

    nmi_enable: bool,
    bg_pattern_table: u16,
    vram_increment: u16,

    vblank: bool,

    nmi: bool,
    redraw: bool,

    vram_address: u16,
    temp_vram_address: u16,
    fine_x_scroll: u8,

    tile: u8,
    tile_low: u8,
    tile_high: u8,

    pattern_shift_low: u16,
    pattern_shift_high: u16,

    attribute_latch_high: u8,
    attribute_latch_low: u8,

    attribute_shift_high: u8,
    attribute_shift_low: u8,

    write_toggle: bool,
}

impl Ricoh2C02 {
    pub fn new(mapper: Box<Mapper+Send>) -> Ricoh2C02 {
        Ricoh2C02 {
            mapper: mapper,

            framebuffer: vec![0; 256 * 240 * 3].into_boxed_slice(),

            nametable_0: vec![0; 0x400].into_boxed_slice(),
            nametable_1: vec![0; 0x400].into_boxed_slice(),
            nametable_2: vec![0; 0x400].into_boxed_slice(),
            nametable_3: vec![0; 0x400].into_boxed_slice(),

            palette: vec![0; 0x20].into_boxed_slice(),

            scanline: 0,
            cycle: 0,
            odd: false,

            latch: 0,
            read_buffer: 0,

            nmi_enable: false,
            bg_pattern_table: 0,
            vram_increment: 1,

            vblank: false,

            nmi: false,
            redraw: false,

            vram_address: 0,
            temp_vram_address: 0,
            fine_x_scroll: 0,

            tile: 0,
            tile_low: 0,
            tile_high: 0,

            pattern_shift_low: 0,
            pattern_shift_high: 0,

            attribute_latch_high: 0,
            attribute_latch_low: 0,

            attribute_shift_high: 0,
            attribute_shift_low: 0,

            write_toggle: false,
        }
    }

    pub fn draw_screen(&self, texture: &mut sdl2::render::Texture) {
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..240 {
                for x in 0..256 {
                    let offset = y * pitch + x * 3;
                    buffer[offset] = self.framebuffer[((y * 256) + x) * 3];
                    buffer[offset + 1] = self.framebuffer[(((y * 256) + x) * 3) + 1];
                    buffer[offset + 2] = self.framebuffer[(((y * 256) + x) * 3) + 2];
                }
            }
        }).unwrap();
    }

    pub fn in_range(&self, address: u16) -> bool {
        let address = address as usize;

        if (address >= PPU_START) && (address <= PPU_END) ||
            address == PPU_OAMDMA {
            return true;
        }

        return false;
    }

    fn increment_x(&mut self) {
        if (self.vram_address & 0x001f) == 31 {
            self.vram_address &= !0x001f;
            self.vram_address ^= 0x0400;
        }
        else {
            self.vram_address += 1;
        }
    }

    fn increment_y(&mut self) {
        if (self.vram_address & 0x7000) != 0x7000 {
            self.vram_address += 0x1000;
        }
        else {
            self.vram_address &= !0x7000;
        }

        let mut coarse_y = (self.vram_address & 0x03e0) >> 5;

        if coarse_y == 29 {
            coarse_y = 0;
            self.vram_address ^= 0x0800;
        } else if coarse_y == 31 {
            coarse_y = 0;
        } else {
            coarse_y += 1;
        }

        self.vram_address &= !0x03e0;
        self.vram_address |= coarse_y << 5
    }

    pub fn io_read(&mut self, address: u16) -> u8 {
        let address = address as usize;

        if address == PPU_OAMDMA {
            println!("PPU_READ: OAM_DMA");

        } else {
            let address = PPU_START + (address % 8);

            match address {
                PPU_CTRL => {
                    println!("PPU_READ: PPU_CTRL");
                },

                PPU_MASK => {
                    println!("PPU_READ: PPU_MASK");                    
                },

                PPU_STATUS => {
                    println!("PPU_READ: PPU_STATUS");
                    self.latch |= (self.vblank as u8) << 7;
                    self.vblank = false;
                    self.write_toggle = false;
                },

                PPU_OAMADDR => {
                    println!("PPU_READ: PPU_OAMADDR");
                },

                PPU_OAMDATA => {
                    println!("PPU_READ: PPU_OAMDATA");
                },

                PPU_SCROLL => {
                    println!("PPU_READ: PPU_SCROLL");
                },

                PPU_ADDR => {
                    println!("PPU_READ: PPU_ADDR");
                },

                PPU_DATA => {
                    println!("PPU_READ: PPU_DATA");

                    let vram_address = self.vram_address;

                    if address >= 0x3f00 {
                        self.latch = self.vram_read(vram_address);
                        self.read_buffer = self.nametable_read(vram_address);
                    } else {
                        self.latch = self.read_buffer;
                        self.read_buffer = self.vram_read(vram_address);
                    }

                    self.vram_address += self.vram_increment;
                },

                _ => unreachable!()
            }
        }

        self.latch
    }

    pub fn io_write(&mut self, address: u16, value: u8) {
        let address = address as usize;

        self.latch = value;
        
        if address == PPU_OAMDMA {
            println!("PPU_WRITE: OAM_DMA");

        } else {
            let address = PPU_START + (address % 8);

            match address {
                PPU_CTRL => {
                    println!("PPU_WRITE: PPU_CTRL");
                    self.nmi_enable = (self.latch & 0x80) != 0;

                    if (self.latch & 0x10) != 0 {
                        self.bg_pattern_table = 0x1000;
                    }
                    else {
                        self.bg_pattern_table = 0;
                    }

                    if (self.latch & 0x04) != 0 {
                        self.vram_increment = 32;   
                    } else {
                        self.vram_increment = 1;
                    }

                    self.temp_vram_address &= !0x0c00;
                    self.temp_vram_address |= ((self.latch as u16) & 0x03) << 10;
                },

                PPU_MASK => {
                    println!("PPU_WRITE: PPU_MASK");                    
                },

                PPU_STATUS => {
                    println!("PPU_WRITE: PPU_STATUS");
                },

                PPU_OAMADDR => {
                    println!("PPU_WRITE: PPU_OAMADDR");
                },

                PPU_OAMDATA => {
                    println!("PPU_WRITE: PPU_OAMDATA");
                },

                PPU_SCROLL => {
                    println!("PPU_WRITE: PPU_SCROLL");

                    if self.write_toggle {
                        self.temp_vram_address &= !0x73e0;
                        self.temp_vram_address |= ((self.latch as u16) & 0xf8) << 2;
                        self.temp_vram_address |= ((self.latch as u16) & 0x07) << 12;
                    } else {
                        self.temp_vram_address &= !0x001f;
                        self.temp_vram_address |= (self.latch as u16) >> 3;
                        self.fine_x_scroll = self.latch & 0x7;
                    }

                    self.write_toggle = !self.write_toggle;
                },

                PPU_ADDR => {
                    println!("PPU_WRITE: PPU_ADDR");

                    if self.write_toggle {
                        self.temp_vram_address &= !0x00ff;
                        self.temp_vram_address |= self.latch as u16;
                        self.vram_address = self.temp_vram_address;
                    } else {
                        self.temp_vram_address &= !0x7f00;
                        self.temp_vram_address |= ((self.latch as u16) & 0x3f) << 8;
                    }

                    self.write_toggle = !self.write_toggle;                   
                },

                PPU_DATA => {
                    println!("PPU_WRITE: PPU_DATA");
                    let vram_address = self.vram_address;
                    let latch = self.latch;

                    self.vram_write(vram_address, latch);
                    self.vram_address += self.vram_increment;
                },

                _ => unreachable!()
            }
        } 
    }

    pub fn nametable_read(&mut self, address: u16) -> u8 {
        let address = address as usize;

        if address < 0x2400 {
            return self.nametable_0[address - 0x2000]
        }

        if address < 0x2800 {
            match self.mapper.mirroring() {
                MirrorMode::Horizontal => return self.nametable_0[address - 0x2400],
                MirrorMode::Vertical => return self.nametable_2[address - 0x2400],
                MirrorMode::FourScreen => return self.nametable_1[address - 0x2400],
            }
        }

        if address < 0x2c00 {
            match self.mapper.mirroring() {
                MirrorMode::Horizontal => return self.nametable_1[address - 0x2800],
                MirrorMode::Vertical => return self.nametable_0[address - 0x2800],
                MirrorMode::FourScreen => return self.nametable_2[address - 0x2800],
            }
        }

        match self.mapper.mirroring() {
            MirrorMode::Horizontal => self.nametable_2[address - 0x2c00],
            MirrorMode::Vertical => self.nametable_1[address - 0x2c00],
            MirrorMode::FourScreen => self.nametable_3[address - 0x2c00],
        }
    }

    pub fn nametable_write(&mut self, address: u16, value: u8) {
        let address = address as usize;

        println!("{:x}", address);

        if address < 0x2400 {
            return self.nametable_0[address - 0x2000] = value;
        }

        if address < 0x2800 {
            return match self.mapper.mirroring() {
                MirrorMode::Horizontal => self.nametable_0[address - 0x2400] = value,
                MirrorMode::Vertical => self.nametable_2[address - 0x2400] = value,
                MirrorMode::FourScreen => self.nametable_1[address - 0x2400] = value,
            };
        }

        if address < 0x2c00 {
            return match self.mapper.mirroring() {
                MirrorMode::Horizontal => self.nametable_1[address - 0x2800] = value,
                MirrorMode::Vertical => self.nametable_0[address - 0x2800] = value,
                MirrorMode::FourScreen => self.nametable_2[address - 0x2800] = value,
            };
        }

        return match self.mapper.mirroring() {
            MirrorMode::Horizontal => self.nametable_2[address - 0x2c00] = value,
            MirrorMode::Vertical => self.nametable_1[address - 0x2c00] = value,
            MirrorMode::FourScreen => self.nametable_3[address - 0x2c00] = value,
        };
    }

    pub fn palette_read(&mut self, address: u16) -> u8 {
        let mut address = address - 0x3f00;

        if address == 0x10 || address == 0x14 || address == 0x18 || address == 0x1c {
            address &= 0x0f;
        }

        self.palette[address as usize] % 0x40
    }

    pub fn palette_write(&mut self, address: u16, value: u8) {
        let mut address = address - 0x3f00;

        if address == 0x10 || address == 0x14 || address == 0x18 || address == 0x1c {
            address &= 0x0f;
        }

        self.palette[address as usize] = value % 0x40;
    }

    pub fn should_nmi(&mut self) -> bool {
        let nmi_status = self.nmi_enable & self.nmi;

        if nmi_status {
            self.nmi = false;
        }

        nmi_status
    }

    pub fn should_redraw(&mut self) -> bool {
        let redraw = self.redraw;
        self.redraw = false;
        redraw
    }

    pub fn tick(&mut self) {
        if self.scanline == 0 && self.cycle == 0 {
            if self.odd {
                self.cycle += 1;
            }

            self.odd = !self.odd;
        }

        if self.scanline == 241 && self.cycle == 1 {
            self.vblank = true;
            self.nmi = true;
            self.redraw = true;
        }

        if self.scanline == 261 {
            if self.cycle == 1 {
                self.vblank = false;
            }

            else if self.cycle >= 280 && self.cycle <= 304 {
                self.vram_address &= !0x7be0;
                self.vram_address |= self.temp_vram_address & 0x7be0;
            }
        }

        if self.scanline < 240 || self.scanline == 261 {
            if self.scanline 

            if self.cycle < 256 {
                let fetch = self.cycle % 8;

                if self.cycle != 0 && fetch == 0 {
                    self.increment_x();
                }

                else if fetch == 1 {
                    self.pattern_shift_low &= 0x00ff;
                    self.pattern_shift_low |= (self.tile_low as u16) << 8;

                    self.pattern_shift_high &= 0x00ff;
                    self.pattern_shift_high |= (self.tile_high as u16) << 8;

                    let mut tile_address = 0x2000;
                    tile_address |= self.vram_address & 0x0fff;

                    self.tile = self.vram_read(tile_address);
                }
                    
                else if fetch == 3 {
                    let mut at_address = 0x23c0;
                    at_address |= self.vram_address & 0x0c00;
                    at_address |= (self.vram_address & 0x0380) >> 4;
                    at_address |= (self.vram_address & 0x001c) >> 2;

                    let mut attribute_byte = self.vram_read(at_address);

                    if self.vram_address & 0x02 != 0 {
                        attribute_byte >>= 2;
                    }
                    
                    if self.vram_address & 0x40 != 0 {
                        attribute_byte >>= 4;
                    }

                    self.attribute_latch_low = attribute_byte & 0x1;
                    self.attribute_latch_high = (attribute_byte & 0x2) >> 1;
                }

                else if fetch == 5 {
                    let tile_address = self.bg_pattern_table | (self.tile as u16 * 0x10);
                    self.tile_low = self.vram_read(tile_address);
                }

                else if fetch == 7 {
                    let tile_address = self.bg_pattern_table | (self.tile as u16 * 0x10) | 0x8;
                    self.tile_high = self.vram_read(tile_address);
                }
            }
                    
            else if self.cycle == 256 {
                self.increment_x();
                self.increment_y();
            }

            else if self.cycle == 257 {
                self.vram_address &= !0x041f;
                self.vram_address |= self.temp_vram_address & 0x041f;
            }

            else if self.cycle == 328 || self.cycle == 336 {
                self.increment_x();
            }
        }

    if self.scanline < 240 && self.cycle != 0 && self.cycle < 257 {
                let pattern_low = self.pattern_shift_low & (1 << self.fine_x_scroll) as u16;
                let pattern_high = self.pattern_shift_high & (1 << self.fine_x_scroll) as u16;

                let palette_index_low = pattern_low >> self.fine_x_scroll;
                let palette_index_high = (pattern_high >> self.fine_x_scroll) << 1;

                let palette_index = palette_index_high | palette_index_low;

                self.pattern_shift_low >>= 1;
                self.pattern_shift_high >>= 1;

                let attribute_bit_low = self.attribute_shift_low & (1 << self.fine_x_scroll);
                let attribute_bit_high = self.attribute_shift_high & (1 << self.fine_x_scroll);

                let attribute_low = attribute_bit_low >> self.fine_x_scroll;
                let attribute_high = (attribute_bit_high >> self.fine_x_scroll) << 1;

                let palette = (attribute_high | attribute_low) as u16;

                self.attribute_shift_low >>= 1;
                self.attribute_shift_low |= self.attribute_latch_low << 7;

                self.attribute_shift_high >>= 1;
                self.attribute_shift_high |= self.attribute_latch_high << 7;

                let colour_index = self.vram_read(0x3f00 + (palette * 4) + palette_index) as usize;
                let framebuffer_address = (((self.scanline * 256) + self.cycle - 1) * 3) as usize;

                self.framebuffer[framebuffer_address] = PALETTE[colour_index * 3];
                self.framebuffer[framebuffer_address + 1] = PALETTE[(colour_index * 3) + 1];
                self.framebuffer[framebuffer_address + 2] = PALETTE[(colour_index * 3) + 2];
        }

        self.cycle += 1;

        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline += 1;

            if self.scanline >= 262 {
                self.scanline = 0;
            }
        }
    }

    pub fn vram_read(&mut self, address: u16) -> u8 {
        let address = address % 0x4000;

        if address < 0x2000 {
            return self.mapper.read_chr(address)
        }

        if address < 0x3f00 {
            return self.nametable_read(0x2000 + (address % 0x1000))
        }

        if address < 0x3fff {
            return self.palette_read(0x3f00 + (address % 0x20))
        }

        panic!("PPU_READ_ERROR: unknown address 0x{:04x}", address);
    }

    pub fn vram_write(&mut self, address: u16, value: u8) {
        let address = address % 0x4000;

        if address < 0x2000 {
            return self.mapper.write_chr(address, value);
        }

        if address < 0x3f00 {
            return self.nametable_write(0x2000 + (address % 0x1000), value);
        }

        if address < 0x3fff {
            return self.palette_write(0x3f00 + (address % 0x20), value);
        }

        panic!("PPU_WRITE_ERROR: unknown address 0x{:04x}", address)
    }
}