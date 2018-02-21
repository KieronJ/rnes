extern crate sdl2;

use nes::mapper::Mapper;
use nes::rom::MirrorMode;
use std::cell::RefCell;
use std::rc::Rc;

pub const PPU_START: u16 = 0x2000;
pub const PPU_END: u16 = 0x3fff;

pub const PPU_CTRL: u16 = 0x2000;
pub const PPU_MASK: u16 = 0x2001;
pub const PPU_STATUS: u16 = 0x2002;
pub const PPU_OAMADDR: u16 = 0x2003;
pub const PPU_OAMDATA: u16 = 0x2004;
pub const PPU_SCROLL: u16 = 0x2005;
pub const PPU_ADDR: u16 = 0x2006;
pub const PPU_DATA: u16 = 0x2007;

pub const PPU_PRERENDER: isize = -1;
pub const PPU_POSTRENDER: isize = 240;
pub const PPU_VBLANK_START: isize = 241;
pub const PPU_VBLANK_END: isize = 260;

pub const PPU_LAST_CYCLE: usize = 340;

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
    mapper: Rc<RefCell<Box<Mapper+Send>>>,

    framebuffer: Box<[u8]>,

    nametable_0: Box<[u8]>,
    nametable_1: Box<[u8]>,
    nametable_2: Box<[u8]>,
    nametable_3: Box<[u8]>,

    palette: Box<[u8]>,

    scanline: isize,
    cycle: usize,
    odd: bool,

    latch: u8,
    read_buffer: u8,

    nmi_enable: bool,
    sprite_size: usize,
    bg_pattern_table: u16,
    sprite_pattern_table: u16,
    vram_increment: u16,

    sprite_enable: bool,
    background_enable: bool,
    lc_sprite_enable: bool,
    lc_background_enable: bool,
    greyscale: bool,

    vblank: bool,
    sprite_0_hit: bool,
    sprite_overflow: bool,

    should_nmi: bool,

    redraw: bool,

    vram_address: u16,
    temp_vram_address: u16,
    fine_x_scroll: u8,

    write_toggle: bool,

    tile_address: u16,
    tile_low: u8,
    tile_high: u8,

    tile_shift_low: u16,
    tile_shift_high: u16,

    attribute_shift_low: u8,
    attribute_shift_high: u8,

    attribute_latch_low: u8,
    attribute_latch_high: u8,

    next_attribute_latch_low: u8,
    next_attribute_latch_high: u8,

    oam: Box<[u8]>,
    secondary_oam: Box<[u8]>,

    oam_addr: u8,
    oam_2_addr: u8,
    oam_buffer: u8,
    oam_overflow: bool,

    sprite_fill_count: usize,

    sprite_shift_low: [u8; 8],
    sprite_shift_high: [u8; 8],

    sprite_latch: [u8; 8],
    sprite_counter: [u8; 8],
}

impl Ricoh2C02 {
    pub fn new(mapper: Rc<RefCell<Box<Mapper+Send>>>) -> Ricoh2C02 {
        Ricoh2C02 {
            mapper: mapper,

            framebuffer: vec![0; 256 * 240].into_boxed_slice(),

            nametable_0: vec![0; 0x400].into_boxed_slice(),
            nametable_1: vec![0; 0x400].into_boxed_slice(),
            nametable_2: vec![0; 0x400].into_boxed_slice(),
            nametable_3: vec![0; 0x400].into_boxed_slice(),

            palette: vec![0; 0x20].into_boxed_slice(),

            scanline: -1,
            cycle: 0,
            odd: false,

            latch: 0,
            read_buffer: 0,

            nmi_enable: false,
            sprite_size: 8,
            bg_pattern_table: 0,
            sprite_pattern_table: 0,
            vram_increment: 0,

            sprite_enable: false,
            background_enable: false,
            lc_sprite_enable: false,
            lc_background_enable: false,
            greyscale: false,

            vblank: false,
            sprite_0_hit: false,
            sprite_overflow: false,

            should_nmi: false,

            redraw: false,

            vram_address: 0,
            temp_vram_address: 0,
            fine_x_scroll: 0,

            write_toggle: false,

            tile_address: 0,
            tile_low: 0,
            tile_high: 0,

            tile_shift_low: 0,
            tile_shift_high: 0,

            attribute_shift_low: 0,
            attribute_shift_high: 0,

            attribute_latch_low: 0,
            attribute_latch_high: 0,

            next_attribute_latch_low: 0,
            next_attribute_latch_high: 0,

            oam: vec![0; 256].into_boxed_slice(),
            secondary_oam: vec![0; 32].into_boxed_slice(),

            oam_addr: 0,
            oam_2_addr: 0,
            oam_buffer: 0,
            oam_overflow: false,

            sprite_fill_count: 0,

            sprite_shift_low: [0; 8],
            sprite_shift_high: [0; 8],

            sprite_latch: [0; 8],
            sprite_counter: [0; 8],
        }
    }

    pub fn odd(&self) -> bool {
        return self.odd;
    }

    fn copy_horizontal_bits(&mut self) {
        self.vram_address &= !0x041f;
        self.vram_address |= self.temp_vram_address & 0x041f;
    }

    fn copy_vertical_bits(&mut self) {
        self.vram_address &= !0x7be0;
        self.vram_address |= self.temp_vram_address & 0x7be0;
    }

    fn priority_select(&mut self) -> u16 {
        if !self.rendering_enabled() && (self.vram_address & 0x3f00) == 0x3f00 {
            return self.vram_address & 0x1f;
        }

        let mut colour = 0;
        if self.background_enable {
            colour = self.get_pixel_colour() as u16;
        }

        let mut priority_colour = 0;
        for sprite in 0..8 {
            let mut sprite_colour = 0;

            if self.sprite_enable {
                sprite_colour = self.get_sprite_colour(sprite) as u16;
            }

            if sprite == 0 && colour != 0 && sprite_colour != 0 && self.cycle < 255 {
                self.sprite_0_hit = true;
            }

            if (sprite_colour != 0) && ((self.sprite_latch[sprite] & 0x20) != 0) {
                if colour != 0 {
                    return colour;
                } else {
                    return sprite_colour;
                }
            } else {
                if sprite_colour != 0 {
                    return sprite_colour;
                } else if colour != 0 {
                    priority_colour = colour
                } else {
                    priority_colour = 0;
                }
            }
        }

        priority_colour
    }

    fn draw_pixel(&mut self) {
        let framebuffer_address = ((self.scanline as usize) << 8) + self.cycle - 1;

        let pixel_colour = self.priority_select();

        if self.scanline < 8 || self.scanline > 231 {
            self.framebuffer[framebuffer_address] = self.palette_read(0x3f00);
        } else {
            self.framebuffer[framebuffer_address] = self.palette_read(pixel_colour);
        }
    }

    fn get_pixel_colour(&self) -> u8 {
        let attribute_high = ((self.attribute_shift_high << self.fine_x_scroll) & 0x80) >> 4;
        let attribute_low = ((self.attribute_shift_low << self.fine_x_scroll) & 0x80) >> 5;

        let tile_high = ((self.tile_shift_high << self.fine_x_scroll) & 0x8000) >> 14;
        let tile_low = ((self.tile_shift_low << self.fine_x_scroll) & 0x8000) >> 15;

        let mut colour = 0;
        colour |= attribute_high;
        colour |= attribute_low;
        colour |= tile_high as u8;
        colour |= tile_low as u8;

        if colour & 0x03 == 0 {
            colour = 0;
        }

        if (self.cycle < 8) && !self.lc_background_enable {
            colour = 0;
        }

        colour
    }

    fn get_sprite_colour(&mut self, index: usize) -> u8 {
        let pattern_high;
        let pattern_low;

        let attribute = (self.sprite_latch[index] & 0x03) << 2;

        if self.sprite_latch[index] & 0x40 != 0 {
            pattern_high = (self.sprite_shift_high[index] & 0x01) << 1;
            pattern_low = self.sprite_shift_low[index] & 0x01;
        } else {
            pattern_high = (self.sprite_shift_high[index] & 0x80) >> 6;
            pattern_low = (self.sprite_shift_low[index] & 0x80) >> 7;
        }

        let mut colour = 0;
        if self.sprite_counter[index] == 0 {
            colour |= 0x10;
            colour |= attribute;
            colour |= pattern_high;
            colour |= pattern_low;

            if (self.cycle <= 8) && !self.lc_sprite_enable {
                colour = 0;
            }
        }

        if colour & 0x03 == 0 {
            colour = 0;
        }

        colour
    }

    //pub fn draw_nametables(&mut self, texture: &mut sdl2::render::Texture) {
    //    texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
    //        for y in 0..480 {
    //            for x in 0..512 {
    //                let offset = (y * pitch) + (x * 3);
    //                let mut tile_address = 0x2000;
    //                if x >= 256 {
    //                    tile_address += 0x400;
    //                }
    //                if y >= 240 {
    //                    tile_address += 0x800;
    //                }
    //                tile_address += (((y % 240) / 8) * 32) + ((x % 256) / 8);
    //                let tile = self.vram_read(tile_address as u16);
    //                let pattern_address = self.bg_pattern_table + (tile as u16 * 0x10);
    //                let tile_low = self.vram_read(pattern_address + (y as u16 % 8));
    //                let tile_high = self.vram_read(pattern_address + (y as u16 % 8) + 8);
    //                let bit = (7 - (x % 8)) as u8;
    //                let mut colour_low = (tile_low & (1 << bit)) >> bit;
    //                let mut colour_high = (tile_high & (1 << bit)) >> bit;
    //                let colour = match colour_low | (colour_high << 1) {
    //                    0 => 0x00,
    //                    1 => 0x55,
    //                    2 => 0xaa,
    //                    3 => 0xff,
    //                    _ => unreachable!()
    //                };
    //                buffer[offset] = colour;
    //                buffer[offset + 1] = colour;
    //                buffer[offset + 2] = colour;
    //            }
    //        }
    //    }).unwrap();
    //}

 //pub fn draw_tiles(&mut self, texture: &mut sdl2::render::Texture) {
    //    texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
    //        for y in 0..128 {
    //            for x in 0..128 {
    //                let offset = (y * pitch) + (x * 3);
    //                let tile = ((y / 8) * 16) + (x / 8);
    //                let address = (tile * 0x10) + (y % 8);
//
    //                let tile_low = self.mapper.read_chr(address as u16);
    //                let tile_high = self.mapper.read_chr((address + 8) as u16);
//
    //                let bit = (7 - (x % 8)) as u8;
    //                let mut colour_low = (tile_low & (1 << bit)) >> bit;
    //                let mut colour_high = (tile_high & (1 << bit)) >> bit;
//
    //                let colour = match colour_low | (colour_high << 1) {
    //                    0 => 0x00,
    //                    1 => 0x55,
    //                    2 => 0xaa,
    //                    3 => 0xff,
    //                    _ => unreachable!()
    //                };
//
    //                buffer[offset] = colour;
    //                buffer[offset + 1] = colour;
    //                buffer[offset + 2] = colour;
    //            }
    //        }
//
    //        for y in 0..128 {
    //            for x in 0..128 {
    //                let offset = (y * pitch) + ((x + 128) * 3);
    //                let tile = ((y / 8) * 16) + (x / 8);
    //                let address = 0x1000 + (tile * 0x10) + (y % 8);
//
    //                let tile_low = self.mapper.read_chr(address as u16);
    //                let tile_high = self.mapper.read_chr((address + 8) as u16);
//
    //                let bit = (7 - (x % 8)) as u8;
    //                let mut colour_low = (tile_low & (1 << bit)) >> bit;
    //                let mut colour_high = (tile_high & (1 << bit)) >> bit;
//
    //                let colour = match colour_low | (colour_high << 1) {
    //                    0 => 0x00,
    //                    1 => 0x55,
    //                    2 => 0xaa,
    //                    3 => 0xff,
    //                    _ => unreachable!()
    //                };
//
    //                buffer[offset] = colour;
    //                buffer[offset + 1] = colour;
    //                buffer[offset + 2] = colour;
    //            }
    //        }
    //    }).unwrap();
    //}

    pub fn draw_screen(&self, texture: &mut sdl2::render::Texture) {
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..240 {
                for x in 0..256 {
                    let texture_address = (y * pitch) + (x * 3);
                    let framebuffer_address = (y << 8) + x;

                    let pixel_colour = self.framebuffer[framebuffer_address];

                    let palette_address = (pixel_colour * 3) as usize;

                    buffer[texture_address] = PALETTE[palette_address];
                    buffer[texture_address + 1] = PALETTE[palette_address + 1];
                    buffer[texture_address + 2] = PALETTE[palette_address + 2];
                }
            }
        }).unwrap();
    }

    pub fn in_range(&self, address: u16) -> bool {
        if (address >= PPU_START) && (address <= PPU_END) {
            return true;
        }

        return false;
    }

    fn increment_horizontal_scroll(&mut self) {
        if (self.vram_address & 0x001f) == 31 {
            self.vram_address &= !0x001f;
            self.vram_address ^= 0x0400;
        }
        else {
            self.vram_address += 1;
        }
    }

    fn increment_vertical_scroll(&mut self) {
        if (self.vram_address & 0x7000) != 0x7000 {
            self.vram_address += 0x1000;
        }
        else {
            self.vram_address &= !0x7000;

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
            self.vram_address |= coarse_y << 5;
        }
    }

    pub fn io_read(&mut self, address: u16) -> u8 {
        match address {
            PPU_CTRL => {
                //println!("PPU_CTRL");
            },

            PPU_MASK => {
                //println!("PPU_MASK");                    
            },

            PPU_STATUS => {
                self.latch &= 0x1f;
                self.latch |= (self.vblank as u8) << 7;
                self.latch |= (self.sprite_0_hit as u8) << 6;
                self.latch |= (self.sprite_overflow as u8) << 5;

                self.vblank = false;
                self.write_toggle = false;

                //println!("PPU_STATUS -> 0x{:02x}", self.latch);
            },

            PPU_OAMADDR => {
                //println!("PPU_OAMADDR");
            },

            PPU_OAMDATA => {
                self.latch = self.oam[self.oam_addr as usize];
            },

            PPU_SCROLL => {
                //println!("PPU_SCROLL");
            },

            PPU_ADDR => {
                //println!("PPU_ADDR");
            },

            PPU_DATA => {
                let vram_address = self.vram_address;

                self.latch = self.read_buffer;
                self.read_buffer = self.vram_read(vram_address);

                if (vram_address & 0x3fff) >= 0x3f00 {
                    self.latch = self.vram_read(vram_address);
                    self.read_buffer = self.nametable_read(vram_address);
                }

                if self.rendering_enabled()
                && (self.scanline < PPU_POSTRENDER) {
                    self.increment_horizontal_scroll();
                    self.increment_vertical_scroll();
                } else {
                    self.vram_address += self.vram_increment;
                }

                //println!("PPU_DATA -> 0x{:02x}", self.latch);
            },
            _ => unreachable!()
        }

        self.latch
    }

    pub fn io_write(&mut self, address: u16, value: u8) {
        self.latch = value;

        match address {
            PPU_CTRL => {
                self.nmi_enable = (self.latch & 0x80) != 0;

                if (self.latch & 0x20) != 0 {
                    self.sprite_size = 16;
                } else {
                    self.sprite_size = 8;
                }

                if (self.latch & 0x10) != 0 {
                    self.bg_pattern_table = 0x1000;
                }
                else {
                    self.bg_pattern_table = 0;
                }

                if (self.latch & 0x08) != 0 {
                    self.sprite_pattern_table = 0x1000;
                }
                else {
                    self.sprite_pattern_table = 0;
                }

                if (self.latch & 0x04) != 0 {
                    self.vram_increment = 32;   
                } else {
                    self.vram_increment = 1;
                }

                self.temp_vram_address &= !0x0c00;
                self.temp_vram_address |= ((self.latch as u16) & 0x03) << 10;

                //println!("0x{:02x} -> PPU_CTRL", self.latch);
            },

            PPU_MASK => {
                self.sprite_enable = (self.latch & 0x10) != 0;
                self.background_enable = (self.latch & 0x08) != 0;
                self.lc_sprite_enable = (self.latch & 0x04) != 0;
                self.lc_background_enable = (self.latch & 0x02) != 0;
                self.greyscale = (self.latch & 0x01) != 0;

                //println!("0x{:02x} -> PPU_MASK", self.latch);
            },

            PPU_STATUS => {
                //println!("PPU_STATUS");
            },

            PPU_OAMADDR => {
                self.oam_addr = self.latch;
            },

            PPU_OAMDATA => {
                self.oam[self.oam_addr as usize] = self.latch;
                self.oam_addr = self.oam_addr.wrapping_add(1);
            },

            PPU_SCROLL => {
                if self.write_toggle {
                    self.temp_vram_address &= !0x73e0;
                    self.temp_vram_address |= (self.latch as u16 & 0xf8) << 2;
                    self.temp_vram_address |= (self.latch as u16 & 0x07) << 12;
                } else {
                    self.temp_vram_address &= !0x001f;
                    self.temp_vram_address |= (self.latch as u16) >> 3;
                    self.fine_x_scroll = self.latch & 0x7;
                }

                self.write_toggle = !self.write_toggle;

                //println!("0x{:02x} -> PPU_SCROLL", self.latch);
            },

            PPU_ADDR => {
                if self.write_toggle {
                    self.temp_vram_address &= !0x00ff;
                    self.temp_vram_address |= self.latch as u16;
                    self.vram_address = self.temp_vram_address;
                } else {
                    self.temp_vram_address &= !0xff00;
                    self.temp_vram_address |= (self.latch as u16 & 0x3f) << 8;
                }

                self.write_toggle = !self.write_toggle;

                //println!("0x{:02x} -> PPU_ADDR", self.latch);            
            },

            PPU_DATA => {
                let vram_address = self.vram_address;
                let latch = self.latch;

                self.vram_write(vram_address, latch);

                if self.rendering_enabled()
                && (self.scanline < PPU_PRERENDER) {
                    self.increment_horizontal_scroll();
                    self.increment_vertical_scroll();
                } else {
                    self.vram_address += self.vram_increment;
                }

                //println!("0x{:02x} -> PPU_DATA", self.latch);   
            },

            _ => unreachable!()
        }
    }

    pub fn nametable_read(&mut self, address: u16) -> u8 {
        let address = (address & 0xfff) as usize;

        if address < 0x400 {
            return self.nametable_0[address]
        }

        let mapper = self.mapper.borrow_mut();

        if address < 0x800 {
            match mapper.mirroring() {
                MirrorMode::Horizontal => return self.nametable_0[address - 0x400],
                MirrorMode::Vertical => return self.nametable_1[address - 0x400],
                MirrorMode::FourScreen => return self.nametable_1[address - 0x400],
                MirrorMode::OneScreen => return self.nametable_0[address - 0x400],
            }
        }

        if address < 0xc00 {
            match mapper.mirroring() {
                MirrorMode::Horizontal => return self.nametable_2[address - 0x800],
                MirrorMode::Vertical => return self.nametable_0[address - 0x800],
                MirrorMode::FourScreen => return self.nametable_2[address - 0x800],
                MirrorMode::OneScreen => return self.nametable_0[address - 0x800]
            }
        }

        match mapper.mirroring() {
            MirrorMode::Horizontal => self.nametable_2[address - 0xc00],
            MirrorMode::Vertical => self.nametable_1[address - 0xc00],
            MirrorMode::FourScreen => self.nametable_3[address - 0xc00],
            MirrorMode::OneScreen => return self.nametable_0[address - 0xc00]
        }
    }

    pub fn nametable_write(&mut self, address: u16, value: u8) {
        let address = (address & 0xfff) as usize;

        if address < 0x400 {
            return self.nametable_0[address] = value;
        }

        let mapper = self.mapper.borrow_mut();

        if address < 0x800 {
            return match mapper.mirroring() {
                MirrorMode::Horizontal => self.nametable_0[address - 0x400] = value,
                MirrorMode::Vertical => self.nametable_1[address - 0x400] = value,
                MirrorMode::FourScreen => self.nametable_1[address - 0x400] = value,
                MirrorMode::OneScreen => self.nametable_0[address - 0x400] = value,
            };
        }

        if address < 0xc00 {
            return match mapper.mirroring() {
                MirrorMode::Horizontal => self.nametable_2[address - 0x800] = value,
                MirrorMode::Vertical => self.nametable_0[address - 0x800] = value,
                MirrorMode::FourScreen => self.nametable_2[address - 0x800] = value,
                MirrorMode::OneScreen => self.nametable_2[address - 0x800] = value,
            };
        }

        return match mapper.mirroring() {
            MirrorMode::Horizontal => self.nametable_2[address - 0xc00] = value,
            MirrorMode::Vertical => self.nametable_1[address - 0xc00] = value,
            MirrorMode::FourScreen => self.nametable_3[address - 0xc00] = value,
            MirrorMode::OneScreen => self.nametable_3[address - 0xc00] = value,
        };
    }

    pub fn palette_read(&mut self, address: u16) -> u8 {
        let mut address = (address & 0x1f) as usize;

        if address == 0x10 || address == 0x14
        || address == 0x18 || address == 0x1c {
            address &= 0x0f;
        }

        let mut value = self.palette[address];

        if self.greyscale {
            value &= 0x30;
        }

        value
    }

    pub fn palette_write(&mut self, address: u16, value: u8) {
        let mut address = (address & 0x1f) as usize;
        let value = value & 0x3f;

        if address == 0x10 || address == 0x14
        || address == 0x18 || address == 0x1c {
            address &= 0x0f;
        }

        self.palette[address] = value;
    }

    pub fn rendering_enabled(&self) -> bool {
        self.background_enable || self.sprite_enable
    }

    pub fn should_nmi(&mut self) -> bool {
        if self.should_nmi {
            self.should_nmi = false;
            return true;
        }

        false
    }

    pub fn should_redraw(&mut self) -> bool {
        if self.redraw {
            self.redraw = false;
            return true;
        }

        false
    }

    pub fn tick(&mut self) {
        if self.cycle == 0 {
            self.update_cycle();
            return;
        }

        if self.scanline < PPU_POSTRENDER {
            self.process_scanline();

            if self.scanline == PPU_PRERENDER {
                self.process_prerender();
            }
        }

        else if self.scanline == PPU_VBLANK_START {
            self.process_vblank();
        }

        self.update_cycle();
    }

    pub fn get_tile_address(&mut self) -> u16 {
        0x2000 | (self.vram_address & 0x0fff)
    }

    pub fn get_attribute_address(&mut self) -> u16 {
        let mut address = 0x23c0;
        address |= self.vram_address & 0x0c00;
        address |= (self.vram_address >> 4) & 0x38;
        address |= (self.vram_address >> 2) & 0x07;
        address
    }

    pub fn load_tile_info(&mut self) {
        if !self.rendering_enabled() {
            return;
        }

        match self.cycle & 0x07 {
            1 => {
                self.tile_shift_low |= self.tile_low as u16;
                self.tile_shift_high |= self.tile_high as u16;

                self.attribute_latch_low = self.next_attribute_latch_low;
                self.attribute_latch_high = self.next_attribute_latch_high;

                let tile_address = self.get_tile_address();
                let tile = self.vram_read(tile_address);
                self.tile_address = self.bg_pattern_table;
                self.tile_address |= self.vram_address >> 12;
                self.tile_address |= (tile as u16) << 4;
            },

            3 => {
                let mut shift = (self.vram_address >> 4) & 0x4;
                shift |= self.vram_address & 0x2;

                let attribute_address = self.get_attribute_address();
                let mut bits = self.vram_read(attribute_address);
                bits >>= shift;

                self.next_attribute_latch_low = bits & 0x1;
                self.next_attribute_latch_high = (bits & 0x2) >> 1;
            },

            5 => {
                let tile_address = self.tile_address;
                self.tile_low = self.vram_read(tile_address);
            },

            7 => {
                let tile_address = self.tile_address;
                self.tile_high = self.vram_read(tile_address + 8);
            },
            _ => ()
        }
    }

    pub fn shift_registers(&mut self) {
        self.tile_shift_low <<= 1;
        self.tile_shift_high <<= 1;

        self.attribute_shift_low <<= 1;
        self.attribute_shift_low |= self.attribute_latch_low;

        self.attribute_shift_high <<= 1;
        self.attribute_shift_high |= self.attribute_latch_high;

        for i in 0..8 {
            if self.sprite_counter[i] == 0 {
                if self.sprite_latch[i] & 0x40 != 0 {
                    self.sprite_shift_low[i] >>= 1;
                    self.sprite_shift_high[i] >>= 1;
                } else {
                    self.sprite_shift_low[i] <<= 1;
                    self.sprite_shift_high[i] <<= 1;
                }
            }
        }
    }

    fn sprite_inrange(&self, sprite: u8, scanline: isize) -> bool {
        let sprite = sprite as isize;

        scanline >= sprite && scanline < sprite + self.sprite_size as isize
    }

    fn sprite_evaluation(&mut self) {
        if self.cycle < 65 {
            self.secondary_oam[(self.cycle - 1) >> 1] = 0xff;
            return;
        }

        if self.cycle == 65 {
            self.oam_addr = 0;
            self.oam_2_addr = 0;
            self.oam_overflow = false;
        }

        if self.oam_overflow {
            return;
        }

        if self.cycle % 2 == 1 {
            self.oam_buffer = self.oam[self.oam_addr as usize];
        } else {
            if self.oam_2_addr < 32 {
                self.secondary_oam[self.oam_2_addr as usize] = self.oam_buffer;

                if self.sprite_inrange(self.oam_buffer, self.scanline) {
                    self.secondary_oam[(self.oam_2_addr as usize) + 1] = self.oam[self.oam_addr.wrapping_add(1) as usize];
                    self.secondary_oam[(self.oam_2_addr as usize) + 2] = self.oam[self.oam_addr.wrapping_add(2) as usize];
                    self.secondary_oam[(self.oam_2_addr as usize) + 3] = self.oam[self.oam_addr.wrapping_add(3) as usize];

                    self.oam_2_addr += 4;
                }

                self.oam_addr = self.oam_addr.wrapping_add(4);

                if self.oam_addr < 4 {
                    self.oam_overflow = true;
                }

            } else {
                if self.sprite_inrange(self.oam_buffer, self.scanline) {
                    self.sprite_overflow = true;
                } else {
                    self.oam_addr = self.oam_addr.wrapping_add(5);

                    if self.oam_addr < 4 {
                        self.oam_overflow = true;
                    }
                }
            }
        }
    }

    pub fn process_scanline(&mut self) {
        if self.cycle <= 256 {
            self.load_tile_info();

            if self.rendering_enabled() && (self.cycle & 0x7) == 0 {
                self.increment_horizontal_scroll();

                if self.cycle == 256 {
                    self.increment_vertical_scroll();
                }
            }

            if self.scanline != PPU_PRERENDER {
                for i in 0..8 {
                    if self.sprite_counter[i] > 0 {
                        self.sprite_counter[i] -= 1;
                    }
                }

                self.draw_pixel();
            }

            self.shift_registers();
            self.sprite_evaluation();
        }

        else if self.cycle >= 257 && self.cycle <= 320 {
            if self.cycle == 257 {
                self.sprite_fill_count = 0;

                if self.rendering_enabled() {
                    self.copy_horizontal_bits();
                }  
            }

            match self.cycle & 0x07 {
                1 => {
                    self.sprite_latch[self.sprite_fill_count] = self.secondary_oam[(self.sprite_fill_count * 4) + 2];
                    self.sprite_counter[self.sprite_fill_count] = self.secondary_oam[(self.sprite_fill_count * 4) + 3];
                },

                5 => {
                    let tile_index = self.secondary_oam[(self.sprite_fill_count * 4) + 1] as u16;

                    let mut pattern_address;

                    if self.sprite_size == 8 {
                        pattern_address = self.sprite_pattern_table;
                        pattern_address |= (tile_index) << 4;
                    } else {
                        pattern_address = (tile_index & 0x01) << 12;
                        pattern_address |= (tile_index & 0xfe) << 4;
                    }

                    if self.sprite_latch[self.sprite_fill_count] & 0x80 != 0 {
                        let mut line = (self.sprite_size - 1) as u16 - ((self.scanline as u16) - self.secondary_oam[(self.sprite_fill_count * 4)] as u16);

                        if line >= 8 {
                            pattern_address += 1 << 4;
                            line -= 8;
                        }

                        pattern_address |= line;
                    } else {
                        let mut line = (self.scanline as u16) - self.secondary_oam[(self.sprite_fill_count * 4)] as u16;

                        if line >= 8 {
                            pattern_address += 1 << 4;
                            line -= 8;
                        }

                        pattern_address |= line;
                    }

                    self.sprite_shift_low[self.sprite_fill_count] = self.vram_read(pattern_address);
                },

                7 => {
                    let tile_index = self.secondary_oam[(self.sprite_fill_count * 4) + 1] as u16;

                    let mut pattern_address;

                    if self.sprite_size == 8 {
                        pattern_address = self.sprite_pattern_table;
                        pattern_address |= (tile_index) << 4;
                    } else {
                        pattern_address = (tile_index & 0x01) << 12;
                        pattern_address |= (tile_index & 0xfe) << 4;
                    }

                    if self.sprite_latch[self.sprite_fill_count] & 0x80 != 0 {
                        let mut line = (self.sprite_size - 1) as u16 - ((self.scanline as u16) - self.secondary_oam[(self.sprite_fill_count * 4)] as u16);

                        if line >= 8 {
                            pattern_address += 1 << 4;
                            line -= 8;
                        }

                        pattern_address |= line;
                    } else {
                        let mut line = (self.scanline as u16) - self.secondary_oam[(self.sprite_fill_count * 4)] as u16;

                        if line >= 8 {
                            pattern_address += 1 << 4;
                            line -= 8;
                        }

                        pattern_address |= line;
                    }

                    self.sprite_shift_high[self.sprite_fill_count] = self.vram_read(pattern_address.wrapping_add(8));

                    self.sprite_fill_count += 1;
                },
                _ => ()
            }
        }

        else if self.cycle >= 321 && self.cycle <= 336 {
            self.load_tile_info();

            if self.scanline != PPU_PRERENDER {
                self.shift_registers();
            }

            if self.cycle == 328 || self.cycle == 336 {
                if self.rendering_enabled() {
                    self.increment_horizontal_scroll();
                }
            }
        }
    }

    pub fn process_prerender(&mut self) {
        if self.cycle == 1 {
            self.vblank = false;
            self.sprite_0_hit = false;
            self.sprite_overflow = false;
        }

        else if self.cycle >= 280 && self.cycle <= 304 {
            if self.rendering_enabled() {
                self.copy_vertical_bits();
            }
        }
    }

    pub fn process_vblank(&mut self) {
        if self.cycle == 1 {
            self.vblank = true;
            self.redraw = true;

            if self.nmi_enable {
                self.should_nmi = true;
            }
        }
    }

    pub fn update_cycle(&mut self) {
        self.cycle += 1;

        if self.cycle > PPU_LAST_CYCLE {
            self.cycle = 0;
            self.scanline += 1;

            if self.scanline > PPU_VBLANK_END {
                self.scanline = PPU_PRERENDER;

                if self.odd {
                    self.cycle += 1;
                }

                self.odd = !self.odd;
            }
        }
    }

    pub fn vram_read(&mut self, address: u16) -> u8 {
        let address = address & 0x3fff;

        if address < 0x2000 {
            let mapper = self.mapper.borrow_mut();
            return mapper.read_chr(address)
        }

        if address < 0x3f00 {
            return self.nametable_read(address)
        }

        return self.palette_read(address)
        }

    pub fn vram_write(&mut self, address: u16, value: u8) {
        let address = address & 0x3fff;

        if address < 0x2000 {
            let mut mapper = self.mapper.borrow_mut();
            return mapper.write_chr(address, value);
        }

        if address < 0x3f00 {
            return self.nametable_write(address, value);
        }

        return self.palette_write(address, value);
    }
}