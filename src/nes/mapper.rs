use nes::rom::MirrorMode;
use nes::rom::Rom;
use nes::mappers::nrom::Nrom;
use nes::mappers::mmc1::Mmc1;
use nes::mappers::unrom::Unrom;

pub trait Mapper {
    fn in_range(&self, address: u16) -> bool;
    fn mirroring(&self) -> MirrorMode;
    fn read_chr(&self, address: u16) -> u8;
    fn read_prg(&self, address: u16) -> u8;
    fn write_chr(&mut self, address: u16, value: u8);
    fn write_prg(&mut self, address: u16, value: u8);
}

pub fn create_mapper(rom: Rom) -> Box<Mapper + Send> {
    let mapper = rom.mapper();

    println!("Mapper #{}", mapper);

    match mapper {
        0 => Box::new(Nrom::new(rom)) as Box<Mapper + Send>,
        //1 => Box::new(Mmc1::new(rom)) as Box<Mapper + Send>,
        2 => Box::new(Unrom::new(rom)) as Box<Mapper + Send>,
        _ => panic!("unsupported mapper {}", mapper)
    }
}