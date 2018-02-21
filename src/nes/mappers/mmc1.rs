use nes::mapper::Mapper;
use nes::rom::MirrorMode;
use nes::rom::Rom;
use nes::rom::ROM_PRG_BANK_SIZE;

pub struct Mmc1 {
    rom: Rom,
    prg_ram: Box<[u8]>,
    chr_ram: Box<[u8]>,

    shift: u8,
    control: u8,
    prg_bank: u8
}

impl Mmc1 {
    pub fn new(rom: Rom) -> Mmc1 {
        Mmc1 {
            rom: rom,
            prg_ram: vec![0; 0x2000].into_boxed_slice(),
            chr_ram: vec![0; 0x2000].into_boxed_slice(),

            shift: 0b10000,
            control: 0xc | 0x3,
            prg_bank: 0
        }
    }
}

impl Mapper for Mmc1 {
    fn mirroring(&self) -> MirrorMode {
        match self.control & 0x03 {
            0 => MirrorMode::OneScreen,
            1 => MirrorMode::OneScreen,
            2 => MirrorMode::Vertical,
            3 => MirrorMode::Horizontal,
            _ => unreachable!()
        }
    }

    fn in_range(&self, address: u16) -> bool {
        return address >= 0x4020;
    }

    fn read_chr(&self, address: u16) -> u8 {
        self.chr_ram[address as usize]
    }

    fn read_prg(&self, address: u16) -> u8 {
        if address < 0x6000 {
            return 0xff;
        }

        if address < 0x8000 {
            return self.prg_ram[address as usize - 0x6000]
        }

        let prg_address = address - 0x8000;

        if self.control & 0xc == 0x0 || self.control & 0xc == 0x4 {
            let bank = self.prg_bank & 0x0e;

            return self.rom.read_prg(prg_address as usize + (bank as usize * ROM_PRG_BANK_SIZE));
        } 

        if self.control & 0xc == 0x8 {
            if address < 0xc000 {
                return self.rom.read_prg(prg_address as usize);
            } else {
                return self.rom.read_prg((prg_address as usize - ROM_PRG_BANK_SIZE) +
                self.prg_bank as usize * ROM_PRG_BANK_SIZE);
            }
        } 

        if self.control & 0xc == 0xc {
            if address < 0xc000 {
                return self.rom.read_prg(prg_address as usize +
                self.prg_bank as usize * ROM_PRG_BANK_SIZE);
            } else {
                return self.rom.read_prg((prg_address as usize - ROM_PRG_BANK_SIZE) +
                (self.rom.prg_banks() - 1) * ROM_PRG_BANK_SIZE);
            }
        } 

        panic!("unavailable mmc1 configuration! {}", (self.control & 0xc) >> 2)
    }

    fn write_chr(&mut self, address: u16, value: u8) {
        self.chr_ram[address as usize] = value;
    }

    fn write_prg(&mut self, address: u16, value: u8) {
        if address < 0x6000 {
            println!("unsupported write to PRG 0x{:04x}", address)
        }

        if address < 0x8000 {
            self.prg_ram[address as usize - 0x6000] = value
        }

        if value & 0x80 != 0 {
            self.shift = 0b10000;
            self.control |= 0x0c;
        }

        let data = ((value & 0x1) << 4) | (self.shift >> 1);

        if self.shift & 0x1 == 0x1 {
            if address < 0xa000 {
                self.control = data; 
            } else if address >= 0xe000 {
                self.prg_bank = data;
            }

            self.shift = 0b10000;
        } else {
            self.shift = data;
        }
    }
}