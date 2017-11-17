use nes::rom::Rom;
use nes::mappers::nrom::Nrom;

pub trait Mapper {
    fn in_range(&self, address: usize) -> bool;
    fn read_chr(&self, address: usize) -> u8;
    fn read_prg(&self, address: usize) -> u8;
    fn write_chr(&self, address: usize, value: usize);
    fn write_prg(&self, address: usize, value: usize);
}

pub fn create_mapper(rom: Rom) -> Box<Mapper + Send> {
    let mapper = rom.mapper();

    match mapper {
        0 => Box::new(Nrom::new(rom)) as Box<Mapper + Send>,
        _ => panic!("unsupported mapper {}", mapper)
    }
}