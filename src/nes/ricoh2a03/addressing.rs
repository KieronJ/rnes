use nes::ricoh2a03::Ricoh2A03;

impl Ricoh2A03 {
    pub fn imm(&mut self) -> u16 {
		self.pc = self.pc.wrapping_add(1);
		self.pc.wrapping_sub(1)
	}

	pub fn imm16(&mut self) -> u16 {
		self.pc = self.pc.wrapping_add(2);
		self.pc.wrapping_sub(2)
	}

	pub fn zp(&mut self) -> u16 {
		let imm = self.imm();
		self.read8(imm) as u16
	}

	pub fn zpx(&mut self) -> u16 {
		self.tick();
		(self.zp().wrapping_add(self.x as u16)) % 0x100
	}

	pub fn zpy(&mut self) -> u16 {
		self.tick();
		(self.zp().wrapping_add(self.y as u16)) % 0x100
	}

	pub fn abs(&mut self) -> u16 {
		let imm = self.imm16();
		self.read16(imm)
	}	

	pub fn _absx(&mut self) -> u16 {
		self.tick();
		let a = self.abs();
		let x = self.x as u16;
		a.wrapping_add(x)
	}	

	pub fn absx(&mut self) -> u16 {
		let a = self.abs();
		let x = self.x as u16;
		if self.cross(a, x) { self.tick(); }
		a.wrapping_add(x)
	}	

	pub fn absy(&mut self) -> u16 {
		let a = self.abs();
		let y = self.y as u16;
		if self.cross(a, y) { self.tick(); }
		a.wrapping_add(y)
	}	

	pub fn indx(&mut self) -> u16 {
		let i = self.zpx();
		self.read16_d(i, i.wrapping_add(1) % 0x100)
	}

	pub fn _indy(&mut self) -> u16 {
		let i = self.zp();
		let y = self.y as u16;
		self.read16_d(i, i.wrapping_add(1) % 0x100).wrapping_add(y)
	}

	pub fn indy(&mut self) -> u16 {
		let i = self.zp();
		let y = self.y as u16;
		let a = self.read16_d(i, i.wrapping_add(1) % 0x100).wrapping_add(y);
		if self.cross(a.wrapping_sub(y), y) { self.tick(); }
		a
	}
}