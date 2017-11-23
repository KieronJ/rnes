extern crate sdl2;

mod nes;
mod util;

use nes::bus::Bus;
use nes::mapper::create_mapper;
use nes::ricoh2c02::Ricoh2C02;
use nes::ricoh2a03::Ricoh2A03;
use nes::rom::Rom;
use sdl2::event::*;
use sdl2::keyboard::*;
use util::open_file;

fn main() {
	let mut rom_file = open_file("nestest.nes").unwrap();
	let rom = Rom::new(&mut rom_file);

	let mapper = create_mapper(rom.clone());
	let ppu_mapper = create_mapper(rom);

	let ppu = Ricoh2C02::new(mapper);
	let bus = Bus::new(ppu_mapper, ppu);
	let mut cpu = Ricoh2A03::new(bus);

	let sdl_context = sdl2::init().unwrap();
	let mut sdl_event = sdl_context.event_pump().unwrap();
	let sdl_video = sdl_context.video().unwrap();
	let _sdl_window = sdl_video.window("rnes", 256, 240).build().unwrap();

	let mut running = true;

	while running {
		for event in sdl_event.poll_iter() {
			match event {
				Event::Quit {..} => {
					running = false;
				},
				Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
					running = false;
				},
				_ => {},
			}
		}

		cpu.step();
	}
}