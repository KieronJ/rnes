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

    pub fn draw_screen(&self, texture: &mut sdl2::render::Texture) {
        self.ppu.draw_screen(texture);
    }

    pub fn read(&mut self, address: u16) -> u8 {
        if address < 0x2000 {
            return self.ram[address as usize % RAM_SIZE];
        }

        if self.ppu.in_range(address) {
            return self.ppu.io_read(address);
        }

        if self.mapper.in_range(address) {
            return self.mapper.read_prg(address);
        }

        if (address >= 0x4000) && (address <= 0x4020) {
            return 0;
        }

        panic!("read from unknown memory region 0x{:04x}", address)
    }

    pub fn should_redraw(&mut self) -> bool {
        self.ppu.should_redraw()
    }

    pub fn should_nmi(&mut self) -> bool {
        self.ppu.should_nmi()
    }

    pub fn tick(&mut self) {
        self.ppu.tick();
        self.ppu.tick();
        self.ppu.tick();
    }

    pub fn write(&mut self, address: u16, value: u8) {
        if address < 0x2000 {
            return self.ram[address as usize % RAM_SIZE] = value;
        }

        if self.ppu.in_range(address) {
            return self.ppu.io_write(address, value);
        }

        if self.mapper.in_range(address) {
            return self.mapper.write_prg(address, value);
        }

        println!("write to unknown memory region 0x{:04x}", address)
    }
}