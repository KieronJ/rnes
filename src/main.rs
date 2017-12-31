extern crate sdl2;

mod nes;
mod util;

use nes::bus::Bus;
use nes::mapper::create_mapper;
use nes::ricoh2c02::Ricoh2C02;
use nes::ricoh2a03::InterruptType;
use nes::ricoh2a03::Ricoh2A03;
use nes::rom::Rom;
use sdl2::event::*;
use sdl2::keyboard::*;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use std::env;
use std::thread;
use std::time;
use std::time::Instant;
use util::open_file;

pub const FRAME_TIME: f64 = (1.0 / 60.0) * 1000.0;

fn main() {
	let rom_filepath = env::args().nth(1).unwrap();
	let mut rom_file = open_file(rom_filepath).unwrap();
	let rom = Rom::new(&mut rom_file);

	let mapper = create_mapper(rom.clone());
	let ppu_mapper = create_mapper(rom);

	let ppu = Ricoh2C02::new(mapper);
	let bus = Bus::new(ppu_mapper, ppu);
	let mut cpu = Ricoh2A03::new(bus);
	cpu.reset();

	let sdl_context = sdl2::init().unwrap();
	let mut sdl_event = sdl_context.event_pump().unwrap();
	let sdl_video = sdl_context.video().unwrap();

	let sdl_window = sdl_video.window("rnes", 256, 240).build().unwrap();
	let mut sdl_canvas = sdl_window.into_canvas().build().unwrap();
	let sdl_texture_creator = sdl_canvas.texture_creator();
	let mut sdl_texture = sdl_texture_creator.create_texture_streaming(
							PixelFormatEnum::RGB24, 256, 240).unwrap();

	//let nt_window = sdl_video.window("rnes nametables", 512, 480).build().unwrap();
	//let mut nt_canvas = nt_window.into_canvas().build().unwrap();
	//let nt_texture_creator = nt_canvas.texture_creator();
	//let mut nt_texture = nt_texture_creator.create_texture_streaming(
	//						PixelFormatEnum::RGB24, 512, 480).unwrap();

	//let tile_window = sdl_video.window("rnes tiles", 256, 128).build().unwrap();
	//let mut tile_canvas = tile_window.into_canvas().build().unwrap();
	//let tile_texture_creator = tile_canvas.texture_creator();
	//let mut tile_texture = tile_texture_creator.create_texture_streaming(
	//						PixelFormatEnum::RGB24, 512, 480).unwrap();

	let mut running = true;

	let mut start_time = Instant::now();
	let mut end_time;
	let mut elapsed;
	let mut elapsed_ms;
	let mut sleep_time;

	while running {
		if cpu.should_nmi() {
			cpu.interrupt(InterruptType::NMI);
		}

		if cpu.should_irq() {
			cpu.interrupt(InterruptType::IRQ);
		}

		cpu.step();

		if cpu.should_redraw() {
			for event in sdl_event.poll_iter() {
				match event {
					Event::Quit {..} => {
						running = false;
					},

					Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
						running = false;
					},

					Event::KeyDown {keycode, ..} => {
						cpu.set_button(keycode.unwrap(), true);
					},

					Event::KeyUp {keycode, ..} => {
						cpu.set_button(keycode.unwrap(), false);
					},

					_ => {}
				}
			}

			sdl_canvas.clear();
			cpu.draw_screen(&mut sdl_texture);
			sdl_canvas.copy(&sdl_texture, None, Some(Rect::new(0, 0, 256, 240))).unwrap();
			sdl_canvas.present();

			//nt_canvas.clear();
			//cpu.draw_nametables(&mut nt_texture);
			//nt_canvas.copy(&nt_texture, None, Some(Rect::new(0, 0, 512, 480))).unwrap();
			//nt_canvas.present();

			end_time = Instant::now();
			
			elapsed = end_time.duration_since(start_time);
			elapsed_ms = (elapsed.as_secs() as f64 * 1000.0) + (elapsed.subsec_nanos() as f64 / 1000000.0);

			if elapsed_ms < FRAME_TIME {
				sleep_time = (FRAME_TIME - elapsed_ms) as u64;

				if sleep_time != 0 {
					thread::sleep(time::Duration::from_millis(sleep_time));
				}
			}

			start_time = Instant::now();
		}
	}
}