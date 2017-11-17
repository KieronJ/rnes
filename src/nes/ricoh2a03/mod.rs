use nes::bus::Bus;

struct StatusRegister {
    negative: bool,
	overflow: bool,
	decimal: bool,
	interrupt: bool,
	zero: bool,
	carry: bool,
}

impl StatusRegister {
    pub fn new() -> StatusRegister {
        StatusRegister {
            negative: false,
            overflow: false,
            decimal: false,
            interrupt: false,
            zero: false,
            carry: false,
        }
    }
}

pub struct Ricoh2A03 {
    pc: u16,
    a: u8,
    x: u8,
    y: u8,
    s: u8,
    p: StatusRegister,
    bus: Bus,
}

impl Ricoh2A03 {
    pub fn new(bus: Bus) -> Ricoh2A03 {
        let mut cpu = Ricoh2A03 {
            pc: 0,
            a: 0,
            x: 0,
            y: 0,
            s: 0,
            p: StatusRegister::new(),
            bus: bus,
        };

        cpu.reset();
        cpu
    }

    pub fn read8(&self, address: u16) -> u8 {
        self.bus.read(address)
    }

    pub fn read16(&self, address: u16) -> u16 {
        (self.read8(address) as u16) | ((self.read8(address + 1) as u16) << 8)
    }

    pub fn reset(&mut self) {
        self.pc = self.read16(0xfffc);
    }

    pub fn step(&mut self) {
        let opcode = self.read8(self.pc);

        match opcode {
            _ => panic!("unimplemented opcode 0x{:02x}", opcode)
        }
    }

    pub fn write8(&mut self, address: u16, value: u8) {
        self.bus.write(address, value)
    }

    pub fn write16(&mut self, address: u16, value: u16) {
        self.write8(address, value as u8);
        self.write8(address, (value >> 8) as u8);
    }
}