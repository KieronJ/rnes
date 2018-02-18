use nes::mapper::Mapper;
use nes::rom::MirrorMode;
use nes::rom::Rom;
use nes::rom::ROM_PRG_BANK_SIZE;

pub struct Nrom {
    rom: Rom,
    ram: Box<[u8]>,
}

impl Nrom {
    pub fn new(rom: Rom) -> Nrom {
        Nrom {
            rom: rom,
            ram: vec![0; 0x2000].into_boxed_slice()
        }
    }
}

impl Mapper for Nrom {
    fn mirroring(&self) -> MirrorMode {
        self.rom.mirroring()
    }

    fn in_range(&self, address: u16) -> bool {
        return address >= 0x4020;
    }

    fn read_chr(&self, address: u16) -> u8 {
        self.rom.read_chr(address as usize)
    }

    fn read_prg(&self, address: u16) -> u8 {
        if address < 0x6000 {
            return 0xff;
        }

        if address < 0x8000 {
            return self.ram[address as usize - 0x6000]
        }

        let prg_address = address - 0x8000;

        if self.rom.prg_banks() == 1 {
            return self.rom.read_prg((prg_address % (ROM_PRG_BANK_SIZE as u16 * 1)) as usize);
        } else {
            return self.rom.read_prg((prg_address % (ROM_PRG_BANK_SIZE as u16 * 2)) as usize);
        }
    }

    fn write_chr(&mut self, address: u16, _: u8) {
        println!("unsupported write to CHR 0x{:04x}", address)
    }

    fn write_prg(&mut self, address: u16, value: u8) {
        if address >= 0x6000 && address < 0x8000 {
            self.ram[address as usize - 0x6000] = value;
            return;
        }

        panic!("unsupported write to PRG")
    }
}