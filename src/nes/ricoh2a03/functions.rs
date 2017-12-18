extern crate sdl2;

use nes::ricoh2a03::Ricoh2A03;
use sdl2::keyboard::*;

impl Ricoh2A03 {
	pub fn cross(&self, a: u16, b: u16) -> bool {
		(a.wrapping_add(b)) & 0xff00 != a & 0xff00
	}

    //pub fn draw_nametables(&mut self, texture: &mut sdl2::render::Texture) {
    //    self.bus.draw_nametables(texture);
    //}

    pub fn draw_screen(&self, texture: &mut sdl2::render::Texture) {
        self.bus.draw_screen(texture);
    }

    //pub fn draw_tiles(&mut self, texture: &mut sdl2::render::Texture) {
    //    self.bus.draw_tiles(texture);
    //}

	pub fn pop8(&mut self) -> u8 {
		self.s = self.s.wrapping_add(1);
        let s = self.s;
		self.read8(0x100 + (s as u16))
	}

	pub fn pop16(&mut self) -> u16 {
		let lo = self.pop8() as u16;
		let hi = self.pop8() as u16;
		lo | (hi << 8)
	}

    pub fn push8(&mut self, value: u8) {
		let s = self.s;
		self.write8(0x100 + (s as u16), value);
		self.s = s.wrapping_sub(1);
	}

	pub fn push16(&mut self, value: u16) {
		self.push8((value >> 8) as u8);
		self.push8(value as u8);
	}

    pub fn read8(&mut self, address: u16) -> u8 {
        self.tick();
        self.bus.read(address)
    }

    pub fn read16_d(&mut self, address1: u16, address2: u16) -> u16 {
        self.tick();
        (self.read8(address1) as u16) | ((self.read8(address2) as u16) << 8)
    }

    pub fn read16(&mut self, address: u16) -> u16 {
		self.read16_d(address, address + 1)
    }

	pub fn should_redraw(&mut self) -> bool {
		self.bus.should_redraw()
	}

    pub fn set_button(&mut self, keycode: Keycode, state: bool) {
        self.bus.set_button(keycode, state);
    }

	pub fn set_nz(&mut self, value: u8) {
		self.p.negative = (value & 0x80) != 0;
		self.p.zero = value == 0;
	}

	pub fn should_irq(&mut self) -> bool {
		false
	}

	pub fn should_nmi(&mut self) -> bool {
		self.bus.should_nmi()
	}

	pub fn tick(&mut self) {
        self.bus.tick();
	}

    pub fn write8(&mut self, address: u16, value: u8) {
        self.bus.write(address, value)
    }
}