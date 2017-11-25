extern crate sdl2;

use nes::mapper::Mapper;
use nes::rom::MirrorMode;

pub const PPU_OAM_SIZE: usize = 256;

pub const PPU_FRAMEBUFFER_WIDTH: usize = 256;
pub const PPU_FRAMEBUFFER_HEIGHT: usize = 240;
pub const PPU_FRAMEBUFFER_CHANNELS: usize = 3;
pub const PPU_FRAMEBUFFER_STRIDE: usize = PPU_FRAMEBUFFER_WIDTH * PPU_FRAMEBUFFER_CHANNELS;
pub const PPU_FRAMEBUFFER_SIZE: usize = PPU_FRAMEBUFFER_WIDTH * 
                                        PPU_FRAMEBUFFER_HEIGHT *
                                        PPU_FRAMEBUFFER_CHANNELS;

pub const PPU_NAMETABLE_SIZE: usize = 1024;
pub const PPU_PALETTE_SIZE: usize = 32;

pub const PPU_POSTRENDER_LINE: usize = 240;
pub const PPU_PRERENDER_LINE: usize = 261;

pub const PPU_STATUS_UPDATE_CYCLE: usize = 1;
pub const PPU_ODD_SKIP_CYCLE: usize = 339;

pub const PPU_START: usize = 0x2000;
pub const PPU_END: usize = 0x3fff;

pub const PPU_CTRL: usize = 0x2000;
pub const PPU_CTRL_NMI: u8 = 0x80;
pub const PPU_CTRL_PPU_MASTER_SLAVE: u8 = 0x40;
pub const PPU_CTRL_SPRITE_HEIGHT: u8 = 0x20;
pub const PPU_CTRL_BG_TILE: u8 = 0x10;
pub const PPU_CTRL_SPRITE_TILE: u8 = 0x08;
pub const PPU_CTRL_INCREMENT: u8 = 0x04;
pub const PPU_CTRL_NAMETABLE: u8 = 0x03;

pub const PPU_MASK: usize = 0x2001;
pub const PPU_MASK_COLOUR_EMPHASIS: u8 = 0xe0;
pub const PPU_MASK_SPRITE_ENABLE: u8 = 0x10;
pub const PPU_MASK_BG_ENABLE: u8 = 0x08;
pub const PPU_MASK_SPRITE_LC_ENABLE: u8 = 0x04;
pub const PPU_MASK_BG_LC_ENABLE: u8 = 0x02;
pub const PPU_MASK_GREYSCALE: u8 = 0x01;

pub const PPU_STATUS: usize = 0x2002;
pub const PPU_STATUS_VBLANK: u8 = 0x80;
pub const PPU_STATUS_SPRITE0_HIT: u8 = 0x40;
pub const PPU_STATUS_SPRITE_OVERFLOW: u8 = 0x20;
pub const PPU_STATUS_USED: u8 = 0xe0;
pub const PPU_STATUS_UNUSED: u8 = 0x1f;

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
    nametable_0: Box<[u8]>,
    nametable_1: Box<[u8]>,
    nametable_2: Box<[u8]>,
    nametable_3: Box<[u8]>,
    palette: Box<[u8]>,
    oam: Box<[u8]>,

    framebuffer: Box<[u8]>,
    redraw: bool,

    line: usize,
    cycle: usize,
    odd: bool,

    nmi_occured: bool,

    latch: u8,

    controller: u8,
    mask: u8,
    status: u8,
    oam_address: u8,

    scroll: u16,
    scroll_latch: bool,

    address: u16,
    address_latch: bool,

    oamdma: bool,
}

impl Ricoh2C02 {
    pub fn new(mapper: Box<Mapper+Send>) -> Ricoh2C02 {
        Ricoh2C02 {
            mapper: mapper,
            nametable_0: vec![0; PPU_NAMETABLE_SIZE].into_boxed_slice(),
            nametable_1: vec![0; PPU_NAMETABLE_SIZE].into_boxed_slice(),
            nametable_2: vec![0; PPU_NAMETABLE_SIZE].into_boxed_slice(),
            nametable_3: vec![0; PPU_NAMETABLE_SIZE].into_boxed_slice(),
            palette: vec![0; PPU_PALETTE_SIZE].into_boxed_slice(),
            oam: vec![0; PPU_OAM_SIZE].into_boxed_slice(),

            framebuffer: vec![0; PPU_FRAMEBUFFER_SIZE].into_boxed_slice(),
            redraw: false,

            line: 0,
            cycle: 0,
            odd: false,

            nmi_occured: false,

            latch: 0,

            controller: 0,
            mask: 0,
            status: 0,
            oam_address: 0,

            scroll: 0,
            scroll_latch: false,

            address: 0,
            address_latch: false,

            oamdma: false,
        }
    }

	pub fn check_nmi(&self) -> bool {
        if (self.controller & PPU_CTRL_NMI != 0) && (self.nmi_occured) {
            return true;
        }

        return false;
	}

    pub fn clear_nmi(&mut self) {
        self.nmi_occured = false;
    }

    pub fn in_range(&self, address: usize) -> bool {
        if address == PPU_OAMDMA || ((address >= PPU_START) && (address <= PPU_END)) {
            return true;
        }

        return false;
    }

    pub fn read(&mut self, address: usize) -> u8 {
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
                self.latch &= PPU_STATUS_UNUSED;
                self.latch |= self.status & PPU_STATUS_USED;
                self.status &= !PPU_STATUS_VBLANK;
                self.nmi_occured = false;
                self.scroll_latch = false;
		        self.address_latch = false;
            },
            PPU_OAMADDR => {
                println!("PPU_READ: PPU_OAMADDR");
            },
            PPU_OAMDATA => {
                println!("PPU_READ: PPU_OAMDATA");
                //self.latch = self.oam_read();
            },
            PPU_SCROLL => {
                println!("PPU_READ: PPU_SCROLL");
            },
            PPU_ADDR => {
                println!("PPU_READ: PPU_ADDR");
            },
            PPU_DATA => {
                println!("PPU_READ: PPU_DATA");
                let address = self.address as usize;
                self.latch = self.read_vram(address);

                if (self.controller & PPU_CTRL_INCREMENT) != 0 {
                    self.address += 32;
                } else {
                    self.address += 1;
                }
            },
            PPU_OAMADDR => {
                println!("PPU_READ: PPU_OAMADDR");
            },
            _ => unreachable!(),
        }

        return self.latch;
    }

	pub fn redraw(&mut self) -> bool {
		let redraw = self.redraw;
        self.redraw = false;
        redraw
	}

    pub fn scanline(&mut self) {
        for i in 0..256 {
            let x = i;
            let y = self.line as usize;

            let tile_address = ((y / 8) * 32) + (x / 8);
            let attribute_address = 0x3c0 + ((y / 32) * 8) + (x / 32); 

            let tile = self.nametable_0[tile_address];
            let attribute_byte = self.nametable_0[attribute_address];

            let xhalf = (x % 32) / 16;
            let yhalf = (y % 32) / 16;
            let tile_portion = (yhalf * 2) + xhalf;

            let palette = match tile_portion {
                0 => (attribute_byte >> 0) & 0x03,
                1 => (attribute_byte >> 2) & 0x03,
                2 => (attribute_byte >> 4) & 0x03,
                3 => (attribute_byte >> 6) & 0x03,
                _ => unreachable!(),
            };

            let pattern_low = self.mapper.read_chr(0x1000 + (tile as usize * 16) + (y % 8));
            let pattern_high = self.mapper.read_chr(0x1000 + (tile as usize * 16) + (y % 8) + 8);

            let palette_index_low = (pattern_low >> (7 - (x % 8))) & 0x1;
            let palette_index_high = (pattern_high >> (7 - (x % 8))) & 0x1;
            let palette_index = (palette_index_high << 2) | palette_index_low;

            let mut colour_index;

            if palette_index == 0 {
                colour_index = self.palette[0] as usize;
            } else {
                colour_index = self.palette[(palette as usize * 4) + palette_index as usize] as usize;
            }

            let colour_red = PALETTE[(colour_index * 3)];
            let colour_green = PALETTE[(colour_index * 3) + 1];
            let colour_blue = PALETTE[(colour_index * 3) + 2];

            self.framebuffer[(y * PPU_FRAMEBUFFER_STRIDE) + (x * PPU_FRAMEBUFFER_CHANNELS)] = colour_red;
            self.framebuffer[(y * PPU_FRAMEBUFFER_STRIDE) + (x * PPU_FRAMEBUFFER_CHANNELS) + 1] = colour_green;
            self.framebuffer[(y * PPU_FRAMEBUFFER_STRIDE) + (x * PPU_FRAMEBUFFER_CHANNELS) + 2] = colour_blue;
        }
    }

    pub fn copy_framebuffer(&self, texture: &mut sdl2::render::Texture) {
			texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
        		for y in 0..240 {
            		for x in 0..256 {
                	    let offset = (y * pitch) + (x * 3);
                	    buffer[offset] = self.framebuffer[(y * PPU_FRAMEBUFFER_STRIDE) + (x * PPU_FRAMEBUFFER_CHANNELS)];
                	    buffer[offset + 1] = self.framebuffer[(y * PPU_FRAMEBUFFER_STRIDE) + (x * PPU_FRAMEBUFFER_CHANNELS) + 1];
                	    buffer[offset + 2] = self.framebuffer[(y * PPU_FRAMEBUFFER_STRIDE) + (x * PPU_FRAMEBUFFER_CHANNELS) + 2];
            		}
        		}
    		}).unwrap();
    }

    pub fn step(&mut self) {
        match self.line {
            PPU_PRERENDER_LINE => {
                match self.cycle {
                    PPU_STATUS_UPDATE_CYCLE => {
                        self.status &= PPU_STATUS_UNUSED;
                        self.nmi_occured = false;
                        self.redraw = false;
                    }
                    PPU_ODD_SKIP_CYCLE => {
                        if self.odd {
                            self.cycle += 1;
                            self.odd = false;
                        } else {
                            self.odd = true;
                        }
                    }
                    _ => (),
                }
            },
            PPU_POSTRENDER_LINE => {
                match self.cycle {
                    PPU_STATUS_UPDATE_CYCLE => {
                        self.status |= PPU_STATUS_VBLANK;
                        self.nmi_occured = true;
                        self.redraw = true;
                    }
                    _ => (),
                }
            },
            _ => (),
        }

        self.cycle += 1;
        if self.cycle >= 341 {
            self.cycle = 0;

            if self.line < 240 {
                self.scanline();
            }

            self.line += 1;

            if self.line >= 262 {
                self.line = 0;
            }
        }
    }

    pub fn write(&mut self, address: usize, value: usize) {
        let address = PPU_START + (address % 8);

        self.latch = value as u8;

        match address {
            PPU_CTRL => {
                println!("PPU_WRITE: PPU_CTRL");
                self.controller = self.latch;
            },
            PPU_MASK => {
                println!("PPU_WRITE: PPU_MASK");
                self.mask = self.latch;
            },
            PPU_STATUS => {
                println!("PPU_WRITE: PPU_STATUS");
            },
            PPU_OAMADDR => {
                println!("PPU_WRITE: PPU_OAMADDR");
                self.oam_address = self.latch;
            },
            PPU_OAMDATA => {
                println!("PPU_WRITE: PPU_OAMDATA");
                //self.oam_write(self.latch);
            },
            PPU_SCROLL => {
                println!("PPU_WRITE: PPU_SCROLL");
                if !self.scroll_latch {
                    self.scroll &= 0x00ff;
                    self.scroll |= (self.latch as u16) << 8;
                    self.scroll_latch = true;
                } else {
                    self.scroll &= 0xff00;
                    self.scroll |= self.latch as u16;
                }
            },
            PPU_ADDR => {
                println!("PPU_WRITE: PPU_ADDR");
                if !self.address_latch {
                    self.address &= 0x00ff;
                    self.address |= (self.latch as u16) << 8;
                    self.address_latch = true;
                } else {
                    self.address &= 0xff00;
                    self.address |= self.latch as u16;
                }
            },
            PPU_DATA => {
                println!("PPU_WRITE: PPU_DATA");
                let address = self.address as usize;
                let latch = self.latch as usize;
                self.write_vram(address, latch);

                if (self.controller & PPU_CTRL_INCREMENT) != 0 {
                    self.address += 32;
                } else {
                    self.address += 1;
                }
            },
            PPU_OAMADDR => {
                println!("PPU_WRITE: PPU_OAMADDR");
                self.oamdma = true;
            },
            _ => unreachable!(),
        }
    }

    pub fn read_nametable(&mut self, address: usize) -> u8 {
        if address < 0x400 {
            return self.nametable_0[address];
        }

        if address < 0x800 {
            match self.mapper.mirroring() {
                MirrorMode::Horizontal => return self.nametable_0[address - 0x400],
                MirrorMode::Vertical => return self.nametable_1[address - 0x400],
                MirrorMode::FourScreen => return self.nametable_1[address - 0x400],
            }
        }

        if address < 0xc00 {
            match self.mapper.mirroring() {
                MirrorMode::Horizontal => return self.nametable_2[address - 0x800],
                MirrorMode::Vertical => return self.nametable_0[address - 0x800],
                MirrorMode::FourScreen => return self.nametable_2[address - 0x800],
            }
        }

        match self.mapper.mirroring() {
            MirrorMode::Horizontal => return self.nametable_2[address - 0xc00],
            MirrorMode::Vertical => return self.nametable_1[address - 0xc00],
            MirrorMode::FourScreen => return self.nametable_3[address - 0xc00],
        }
    }

    pub fn write_nametable(&mut self, address: usize, value: u8) {
        if address < 0x400 {
            self.nametable_0[address] = value;
            return;
        }

        if address < 0x800 {
            match self.mapper.mirroring() {
                MirrorMode::Horizontal => self.nametable_0[address - 0x400] = value,
                MirrorMode::Vertical => self.nametable_1[address - 0x400] = value,
                MirrorMode::FourScreen => self.nametable_1[address - 0x400] = value,
            }
            return;
        }

        if address < 0xc00 {
            match self.mapper.mirroring() {
                MirrorMode::Horizontal => self.nametable_2[address - 0x800] = value,
                MirrorMode::Vertical => self.nametable_0[address - 0x800] = value,
                MirrorMode::FourScreen => self.nametable_2[address - 0x800] = value,
            }
            return;
        }

        match self.mapper.mirroring() {
            MirrorMode::Horizontal => self.nametable_2[address - 0xc00] = value,
            MirrorMode::Vertical => self.nametable_1[address - 0xc00] = value,
            MirrorMode::FourScreen => self.nametable_3[address - 0xc00] = value,
        }
        return;
    }

    pub fn read_palette(&mut self, address: usize) -> u8 {
        if address == 0x10 || address == 0x14 || address == 0x18 || address == 0x1c {
            address & 0x0f;
        }

        self.palette[address]
    }

    pub fn write_palette(&mut self, address: usize, value: u8) {
        if address == 0x10 || address == 0x14 || address == 0x18 || address == 0x1c {
            address & 0x0f;
        }

        self.palette[address] = value;
    }

    pub fn read_vram(&mut self, address: usize) -> u8 {
        if address < 0x2000 {
            return self.mapper.read_chr(address);
        }

        if address < 0x3f00 {
            return self.read_nametable((address - 0x2000) % 0x1000);
        }

        if address < 0x4000 {
            return self.read_palette((address - 0x3f00) % 0x20)
        }

        panic!("read from unknown vram region 0x{:04x}", address)
    }

    pub fn write_vram(&mut self, address: usize, value: usize) {
        if address < 0x2000 {
            return self.mapper.write_chr(address, value);
        }

        if address < 0x3f00 {
            return self.write_nametable((address - 0x2000) % 0x1000, value as u8);
        }

        if address < 0x4000 {
            return self.write_palette((address - 0x3f00) % 0x20, value as u8)
        }

        panic!("write to unknown vram region 0x{:04x}", address)
    }
}