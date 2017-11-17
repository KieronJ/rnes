use nes::mapper::Mapper;

pub struct Bus {
    mapper: Box<Mapper + Send>,
}

impl Bus {
    pub fn read(&self, address: u16) -> u8 {
        if self.mapper.in_range(address as usize) {
            return self.mapper.read_prg((address - 0x6000) as usize);
        }

        panic!("read from unknown memory region 0x{:04x}", address)
    }

    pub fn write(&mut self, address: u16, value: u8) {
        if self.mapper.in_range(address as usize) {
            return self.mapper.write_prg(address as usize, value as usize);
        }

        panic!("write to unknown memory region 0x{:04x}", address)
    }
}

pub fn create_bus(mapper: Box<Mapper + Send>) -> Bus {
    Bus {
        mapper: mapper,
    }
}