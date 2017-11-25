pub struct Status {
    pub negative: bool,
	pub overflow: bool,
	pub decimal: bool,
	pub interrupt: bool,
	pub zero: bool,
	pub carry: bool,
}

impl Status {
    pub fn new() -> Status {
        Status {
            negative: false,
            overflow: false,
            decimal: false,
            interrupt: true,
            zero: false,
            carry: false,
        }
    }

    pub fn read(&self) -> u8 {
        (self.negative as u8)  << 7  |
		(self.overflow as u8)  << 6  |
		1					   << 5  |
		(self.decimal as u8)   << 3  |
		(self.interrupt as u8) << 2  |
		(self.zero as u8)      << 1  |
		(self.carry as u8)     << 0
    }

    pub fn write(&mut self, value: u8) {
		self.negative  = (value & 0x80) != 0;
		self.overflow  = (value & 0x40) != 0;
		self.decimal   = (value & 0x08) != 0;
		self.interrupt = (value & 0x04) != 0;
		self.zero      = (value & 0x02) != 0;
		self.carry     = (value & 0x01) != 0;
	}
}