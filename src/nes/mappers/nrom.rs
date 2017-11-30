use nes::mapper::Mapper;
use nes::rom::MirrorMode;
use nes::rom::Rom;
use nes::rom::ROM_PRG_BANK_SIZE;

pub struct Nrom {
    rom: Rom,
}

impl Nrom {
    pub fn new(rom: Rom) -> Nrom {
        Nrom {
            rom: rom,
        }
    }
}

impl Mapper for Nrom {
    fn mirroring(&self) -> MirrorMode {
        self.rom.mirroring()
    }

    fn in_range(&self, address: usize) -> bool {
        if (address >= 0x6000) && (address <= 0xffff) {
            return true;
        }

        return false;
    }

    fn read_chr(&self, address: usize) -> u8 {
        self.rom.read_chr(address as u16)
    }

    fn read_prg(&self, address: usize) -> u8 {
        if address < 0x8000 {
            return 0xff;
        }

        let prg_address = address - 0x8000;

        if self.rom.prg_banks() == 1 {
            return self.rom.read_prg((prg_address % (ROM_PRG_BANK_SIZE * 1)) as u16);
        } else {
            return self.rom.read_prg((prg_address % (ROM_PRG_BANK_SIZE * 2)) as u16);
        }
    }

    fn write_chr(&self, _: usize, _: usize) {
        println!("unsupported write to CHR")
    }

    fn write_prg(&self, _: usize, _: usize) {
        panic!("unsupported write to PRG")
    }
}