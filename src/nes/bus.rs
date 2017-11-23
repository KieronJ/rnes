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

    pub fn read(&self, address: u16) -> u8 {
        let address = address as usize;

        if address < 0x2000 {
            return self.ram[address % RAM_SIZE];
        }

        if self.mapper.in_range(address) {
            return self.mapper.read_prg(address - 0x6000);
        }

        panic!("read from unknown memory region 0x{:04x}", address)
    }

    pub fn write(&mut self, address: u16, value: u8) {
        let address = address as usize;

        if address < 0x2000 {
            return self.ram[address % RAM_SIZE] = value;
        }

        if self.mapper.in_range(address) {
            return self.mapper.write_prg(address - 0x6000, value as usize);
        }

        panic!("write to unknown memory region 0x{:04x}", address)
    }
}