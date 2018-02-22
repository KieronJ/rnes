use nes::mapper::Mapper;
use nes::rom::MirrorMode;
use nes::rom::Rom;
use nes::rom::ROM_CHR_BANK_SIZE;
use nes::rom::ROM_PRG_BANK_SIZE;

pub struct Cnrom {
    rom: Rom,
    chr_bank: u8
}

impl Cnrom {
    pub fn new(rom: Rom) -> Cnrom {
        Cnrom {
            rom: rom,
            chr_bank: 0
        }
    }
}

impl Mapper for Cnrom {
    fn mirroring(&self) -> MirrorMode {
        self.rom.mirroring()
    }

    fn in_range(&self, address: u16) -> bool {
        return address >= 0x4020;
    }

    fn read_chr(&self, address: u16) -> u8 {
        let bank_address = self.chr_bank as usize * ROM_CHR_BANK_SIZE;
        self.rom.read_chr(bank_address + address as usize)
    }

    fn read_prg(&self, address: u16) -> u8 {
        if address < 0x8000 {
            0xff
        } else {
            let prg_address = address as usize - 0x8000;
            let prg_banks = self.rom.prg_banks();

            self.rom.read_prg(prg_address % (ROM_PRG_BANK_SIZE * prg_banks))
        }
    }

    fn write_chr(&mut self, address: u16, _: u8) {
        println!("unsupported write to CHR 0x{:04x}", address)
    }

    fn write_prg(&mut self, address: u16, value: u8) {
        if address < 0x8000 {
            println!("unsupported write to PRG 0x{:04x}", address)
        } else {
            self.chr_bank = value & 0x03;
        }
    }
}