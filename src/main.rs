mod nes;
mod util;

use nes::rom::create_rom;
use util::open_file;

fn main() {
	let mut rom_file = open_file("donkey_kong.nes");
	let rom = create_rom(&mut rom_file);
}