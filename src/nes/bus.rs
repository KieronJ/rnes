extern crate sdl2;

use nes::controller::Controller;
use nes::mapper::Mapper;
use nes::ricoh2c02::Ricoh2C02;
use sdl2::keyboard::*;
use std::cell::RefCell;
use std::rc::Rc;

pub const RAM_SIZE: usize = 0x800;

pub struct Bus {
    controller: Controller,
    mapper: Rc<RefCell<Box<Mapper+Send>>>,
    ppu: Ricoh2C02,
    ram: Box<[u8]>,
}

impl Bus {
    pub fn new(mapper: Rc<RefCell<Box<Mapper+Send>>>, ppu: Ricoh2C02) -> Bus {
        Bus {
            controller: Controller::new(),
            mapper: mapper,
            ppu: ppu,
            ram: vec![0; RAM_SIZE].into_boxed_slice(),
        }
    }

    //pub fn draw_nametables(&mut self, texture: &mut sdl2::render::Texture) {
    //    self.ppu.draw_nametables(texture);
    //}

    //pub fn draw_tiles(&mut self, texture: &mut sdl2::render::Texture) {
    //    self.ppu.draw_tiles(texture);
    //}

    pub fn draw_screen(&self, texture: &mut sdl2::render::Texture) {
        self.ppu.draw_screen(texture);
    }

    pub fn read(&mut self, address: u16) -> u8 {
        if address < 0x2000 {
            return self.ram[address as usize % RAM_SIZE];
        }

        if self.ppu.in_range(address) {
            return self.ppu.io_read(0x2000 + (address % 8));
        }

        if self.controller.in_range(address) {
            return self.controller.io_read();
        }

        let mapper = self.mapper.borrow_mut();
        if mapper.in_range(address) {
            return mapper.read_prg(address);
        }

        if address >= 0x4000 && address <= 0x4020 {
            return 0;
        }

        panic!("read from unknown memory region 0x{:04x}", address)
    }

    pub fn set_button(&mut self, keycode: Keycode, state: bool) {
        self.controller.set_button(keycode, state);
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
            return self.ppu.io_write(0x2000 + (address % 8), value);
        }

        {
            let mut mapper = self.mapper.borrow_mut();
            if mapper.in_range(address) {
                return mapper.write_prg(address, value);
        }

        }
        if self.controller.in_range(address) {
            self.controller.io_write(value);
        }

        if address == 0x4014 {
            for i in 0..256 {
                let transfer_address = ((value as u16) << 8) | i;

                let data = self.read(transfer_address);
                self.tick();

                self.write(0x2004, data);
                self.tick();

                self.tick();

                if self.ppu.odd() {
                    self.tick();
                }
            }
        }

        if address >= 0x4000 && address <= 0x4020 {
            return;
        }

        panic!("write to unknown memory region 0x{:04x}", address)
    }
}