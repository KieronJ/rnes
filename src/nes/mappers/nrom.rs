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

    fn in_range(&self, address: u16) -> bool {
        return address >= 0x6000;
    }

    fn read_chr(&self, address: u16) -> u8 {
        self.rom.read_chr(address)
    }

    fn read_prg(&self, address: u16) -> u8 {
        if address < 0x8000 {
            return 0xff;
        }

        let prg_address = address - 0x8000;

        if self.rom.prg_banks() == 1 {
            return self.rom.read_prg((prg_address % (ROM_PRG_BANK_SIZE as u16 * 1)) as u16);
        } else {
            return self.rom.read_prg((prg_address % (ROM_PRG_BANK_SIZE as u16 * 2)) as u16);
        }
    }

    fn write_chr(&self, address: u16, _: u8) {
        println!("unsupported write to CHR 0x{:04x}", address)
    }

    fn write_prg(&self, _: u16, _: u8) {
        panic!("unsupported write to PRG")
    }
}