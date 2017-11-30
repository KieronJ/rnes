extern crate sdl2;

use nes::mapper::Mapper;
use nes::ricoh2c02::Ricoh2C02;

pub const RAM_SIZE: usize = 0x800;

pub struct Bus {
    ram: Box<[u8]>,
    mapper: Box<Mapper+Send>,
    ppu: Ricoh2C02,
}

impl Bus {
    pub fn new(mapper: Box<Mapper+Send>, ppu: Ricoh2C02) -> Bus {
        Bus {
            ram: vec![0; RAM_SIZE].into_boxed_slice(),
            mapper: mapper,
            ppu: ppu,
        }
    }

    pub fn clear_nmi(&mut self) {
        self.ppu.clear_nmi();
    }

    pub fn draw_screen(&self, texture: &mut sdl2::render::Texture) {
        self.ppu.draw_screen(texture);
    }

    pub fn read(&mut self, address: u16) -> u8 {
        let address = address as usize;

        if address < 0x2000 {
            return self.ram[address % RAM_SIZE];
        }

        if self.ppu.in_range(address) {
            return self.ppu.read(address);
        }

        if self.mapper.in_range(address) {
            return self.mapper.read_prg(address);
        }

        if (address >= 0x4000) && (address <= 0x4020) {
            return 0;
        }

        panic!("read from unknown memory region 0x{:04x}", address)
    }

	pub fn redraw(&mut self) -> bool {
		self.ppu.redraw()
	}

	pub fn should_nmi(&self) -> bool {
		self.ppu.should_nmi()
	}

	pub fn tick(&mut self) {
		self.ppu.step();
	}

    pub fn write(&mut self, address: u16, value: u8) {
        let address = address as usize;

        if address < 0x2000 {
            return self.ram[address % RAM_SIZE] = value;
        }

        if self.ppu.in_range(address) {
            return self.ppu.write(address, value as usize);
        }

        if self.mapper.in_range(address) {
            return self.mapper.write_prg(address, value as usize);
        }

        if (address >= 0x4000) && (address <= 0x4020) {
            return;
        }

        panic!("write to unknown memory region 0x{:04x}", address)
    }
}