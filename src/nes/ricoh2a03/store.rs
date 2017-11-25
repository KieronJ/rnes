use nes::ricoh2a03::Ricoh2A03;

impl Ricoh2A03 {
    pub fn store_zp(&mut self, value: u8) {
		let pc = self.pc;
        self.pc += 1;
		let address = self.read8(pc) as u16;
		self.write8(address, value);
	}

	pub fn store_zp_x(&mut self, value: u8) {
		let pc = self.pc;
        self.pc += 1;
		let address = self.read8(pc).wrapping_add(self.x) as u16;
		self.write8(address, value);
	}

	pub fn store_zp_y(&mut self, value: u8) {
		let pc = self.pc;
		self.pc += 1;
		let address = self.read8(pc).wrapping_add(self.y) as u16;
		self.write8(address, value);
	}

	pub fn store_abs(&mut self, value: u8) {
		let pc = self.pc;
        self.pc += 2;
		let address = self.read16(pc);
		self.write8(address, value);
	}	

	pub fn store_abs_x(&mut self, value: u8) {
		let pc = self.pc;
		self.pc += 2;
		let address = self.read16(pc).wrapping_add(self.x as u16);
		self.write8(address, value);
	}	

	pub fn store_abs_y(&mut self, value: u8) {
		let pc = self.pc;
        self.pc += 2;
		let address = self.read16(pc).wrapping_add(self.y as u16);
		self.write8(address, value);
	}

	pub fn store_ind_x(&mut self, value: u8) {
		let pc = self.pc;
        self.pc += 1;
		let address = self.read8(pc).wrapping_add(self.x) as u16;
		let address2 = self.read16(address);
		self.write8(address2, value);
	}

	pub fn store_ind_y(&mut self, value: u8) {
		let pc = self.pc;
    	self.pc += 1;
		let address = self.read8(pc) as u16;
		let address2 = self.read16(address).wrapping_add(self.y as u16);
		self.write8(address2, value);
	}
}