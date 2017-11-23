use std::io::Read;

pub const ROM_CHR_BANK_SIZE:    usize = 8192;
pub const ROM_HEADER_SIZE:      usize = 16;
pub const ROM_PRG_BANK_SIZE:    usize = 16384;

#[derive(Clone)]
pub struct INesHeader {
    magic: [u8; 4],
    prg: u8,
    chr: u8,
    flags6: u8,
    flags7: u8,
    ram: u8,
    flags9: u8,
}

#[derive(Clone)]
pub struct Rom {
    header: INesHeader,
    prg: Box<[u8]>,
    chr: Box<[u8]>,
}

impl Rom {
    pub fn new(file: &mut Read) -> Rom {
        let mut header = [0; ROM_HEADER_SIZE];
        let mut bytes_read = file.read(&mut header[0..]).unwrap();

        let ines_header = INesHeader {
            magic: [header[0], header[1], header[2], header[3]],
            prg: header[4],
            chr: header[5],
            flags6: header[6],
            flags7: header[7],
            ram: header[8],
            flags9: header[9],
        };

        if ines_header.magic != *b"NES\x1a" {
            panic!("invalid iNES header");
        }

        let prg_size = ines_header.prg as usize * ROM_PRG_BANK_SIZE;
        let mut prg = vec![0u8; prg_size].into_boxed_slice();
        bytes_read += file.read(&mut prg[0..]).unwrap();

        let chr_size = ines_header.chr as usize * ROM_CHR_BANK_SIZE;
        let mut chr = vec![0u8; chr_size].into_boxed_slice();
        bytes_read += file.read(&mut chr[0..]).unwrap();

        if bytes_read != (ROM_HEADER_SIZE + prg_size + chr_size) {
            panic!("unexpected EOF");
        }

        Rom {
            header: ines_header,
            prg: prg,
            chr: chr,
        }
    }

    pub fn mapper(&self) -> u8 {
        (self.header.flags6 >> 4) | (self.header.flags7 & 0xf0)
    }

    pub fn read_chr(&self, address: u16) -> u8 {
        self.chr[address as usize]
    }

    pub fn read_prg(&self, address: u16) -> u8 {
        self.prg[address as usize]
    }

    pub fn prg_banks(&self) -> usize {
        self.prg.len() / ROM_PRG_BANK_SIZE
    }
}