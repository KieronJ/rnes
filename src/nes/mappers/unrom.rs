use nes::mapper::Mapper;
use nes::rom::MirrorMode;
use nes::rom::Rom;
use nes::rom::ROM_PRG_BANK_SIZE;

pub struct Unrom {
    rom: Rom,
    chr_ram: Box<[u8]>,
    prg_bank: u8
}

impl Unrom {
    pub fn new(rom: Rom) -> Unrom {
        Unrom {
            rom: rom,
            chr_ram: vec![0; 0x2000].into_boxed_slice(),
            prg_bank: 0
        }
    }
}

impl Mapper for Unrom {
    fn mirroring(&self) -> MirrorMode {
        self.rom.mirroring()
    }

    fn in_range(&self, address: u16) -> bool {
        return address >= 0x4020;
    }

    fn read_chr(&self, address: u16) -> u8 {
        self.chr_ram[address as usize]
    }

    fn read_prg(&self, address: u16) -> u8 {
        if address < 0x8000 {
            return 0xff;
        }

        if address < 0xc000 {
            let prg_address = (address - 0x8000) as usize;
            let bank_offset = self.prg_bank as usize * ROM_PRG_BANK_SIZE;
            return self.rom.read_prg(bank_offset + prg_address);
        } else {
            let prg_address = (address - 0xc000) as usize;
            let bank_offset = (self.rom.prg_banks() - 1) * ROM_PRG_BANK_SIZE;
            return self.rom.read_prg(bank_offset + prg_address);
        }
    }

    fn write_chr(&mut self, address: u16, value: u8) {
        self.chr_ram[address as usize] = value;
    }

    fn write_prg(&mut self, address: u16, value: u8) {
        if address < 0x8000 {
            println!("unsupported write to PRG 0x{:04x}", address)
        }

        self.prg_bank = value & 0x0f;
    }
}