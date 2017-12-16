use nes::rom::MirrorMode;
use nes::rom::Rom;
use nes::mappers::nrom::Nrom;

pub trait Mapper {
    fn in_range(&self, address: u16) -> bool;
    fn mirroring(&self) -> MirrorMode;
    fn read_chr(&self, address: u16) -> u8;
    fn read_prg(&self, address: u16) -> u8;
    fn write_chr(&self, address: u16, value: u8);
    fn write_prg(&self, address: u16, value: u8);
}

pub fn create_mapper(rom: Rom) -> Box<Mapper + Send> {
    let mapper = rom.mapper();

    match mapper {
        0 => Box::new(Nrom::new(rom)) as Box<Mapper + Send>,
        _ => panic!("unsupported mapper {}", mapper)
    }
}