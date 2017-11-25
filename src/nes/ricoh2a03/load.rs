use nes::ricoh2a03::Ricoh2A03;

impl Ricoh2A03 {
    pub fn load_imm(&mut self) -> u8 {
		let pc = self.pc;
        self.pc += 1;
		self.read8(pc)
	}

	pub fn load_zp(&mut self) -> u8 {
		let pc = self.pc;
        self.pc += 1;
		let address = self.read8(pc) as u16;
		self.read8(address)
	}

	pub fn load_zp_x(&mut self) -> u8 {
		let pc = self.pc;
        self.pc += 1;
		let address = self.read8(pc).wrapping_add(self.x) as u16;
		self.read8(address)
	}

	pub fn load_zp_y(&mut self) -> u8 {
		let pc = self.pc;
        self.pc += 1;
		let address = self.read8(pc).wrapping_add(self.y) as u16;
		self.read8(address)
	}

	pub fn load_abs(&mut self) -> u8 {
		let pc = self.pc;
        self.pc += 2;
		let address = self.read16(pc);
		self.read8(address)
	}	

	pub fn load_abs_x(&mut self) -> u8 {
		let pc = self.pc;
        self.pc += 2;
		let address = self.read16(pc).wrapping_add(self.x as u16);
		self.read8(address)
	}	

	pub fn load_abs_y(&mut self) -> u8 {
		let pc = self.pc;
        self.pc += 2;
		let address = self.read16(pc).wrapping_add(self.y as u16);
		self.read8(address)
	}

	pub fn load_ind_x(&mut self) -> u8 {
		let pc = self.pc;
        self.pc += 1;
		let address = self.read8(pc).wrapping_add(self.x);
		let address2 = self.read8(address as u16) as u16 | (self.read8(address.wrapping_add(1) as u16) as u16) << 8;
	    self.read8(address2)
	}

	pub fn load_ind_y(&mut self) -> u8 {
		let pc = self.pc;
        self.pc += 1;
		let address = self.read8(pc) as u16;
		let address2 = self.read16(address).wrapping_add(self.y as u16);
	    self.read8(address2)
	}
}