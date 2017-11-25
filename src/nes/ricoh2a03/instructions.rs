use nes::ricoh2a03::Ricoh2A03;

impl Ricoh2A03 {
	pub fn adc(&mut self, value: u8) {
		let carry = self.p.carry as u16;
		let result = (self.a as u16) + (value as u16) + carry;

		self.set_nz(result as u8);
		self.p.carry = result > 0xff;
		self.p.overflow = ((self.a ^ result as u8) & !(self.a ^ value) & 0x80) != 0;

		self.a = result as u8;
	}

    pub fn and(&mut self, value: u8) {
		let result = self.a & value;
		self.a = result;
		self.set_nz(result);
	}

	pub fn asl(&mut self, value: u8) -> u8 {
		let result = value << 1;
		self.set_nz(result);
		self.p.carry = (value & 0x80) != 0;
		result
	}

	pub fn bit(&mut self, value: u8) {
		self.p.negative = (value & 0x80) != 0;
		self.p.overflow = (value & 0x40) != 0;
		self.p.zero = (self.a & value) == 0;
	}

	pub fn branch(&mut self, value: u8) {
		self.pc = self.pc.wrapping_add((value as i8) as u16);
	}

	pub fn cmp(&mut self, value: u8) {
		let result = self.a.wrapping_sub(value);
		self.set_nz(result);
		self.p.carry = self.a >= value;
	}

    pub fn cpx(&mut self, value: u8) {
		let result = self.x.wrapping_sub(value);
		self.set_nz(result);
		self.p.carry = self.x >= value;
	}

	pub fn cpy(&mut self, value: u8) {
		let result = self.y.wrapping_sub(value);
		self.set_nz(result);
		self.p.carry = self.y >= value;
	}

	pub fn dec(&mut self, value: u8) -> u8 {
		let value = value.wrapping_sub(1);
		self.set_nz(value);
		value
	}

	pub fn eor(&mut self, value: u8) {
		let result = self.a ^ value;
		self.a = result;
		self.set_nz(result);
	}

	pub fn inc(&mut self, value: u8) -> u8 {
		let value = value.wrapping_add(1);
		self.set_nz(value);
		value
	}

	pub fn jmp_ind(&mut self) {
		let pc = self.pc;
		let page = self.read16(pc) & 0xff00;
		let address = self.read16(pc);
		let addr_lo = self.read8(address) as u16;
		let addr_hi = self.read8((address.wrapping_add(1) & 0x00ff) | page) as u16;
		self.pc = addr_lo | (addr_hi << 8);
	}

    pub fn lda(&mut self, value: u8) {
		self.a = value;
		self.set_nz(value);
	}

    pub fn ldx(&mut self, value: u8) {
		self.x = value;
		self.set_nz(value);
	}

    pub fn ldy(&mut self, value: u8) {
		self.y = value;
		self.set_nz(value);
	}

	pub fn lsr(&mut self, value: u8) -> u8 {
		let result = value >> 1;
		self.set_nz(result);
		self.p.carry = (value & 1) != 0;
		result
	}

    pub fn ora(&mut self, value: u8) {
		let result = self.a | value;
		self.a = result;
		self.set_nz(result);
	}

    pub fn rol(&mut self, value: u8) -> u8 {
		let carry = self.p.carry as u8;
		let result = (value << 1) | carry;
		self.set_nz(result);
		self.p.carry = (value & 0x80) != 0;
		result
	}

    pub fn ror(&mut self, value: u8) -> u8 {
		let carry = self.p.carry as u8;
		let result = (value >> 1) | (carry << 7);
		self.set_nz(result);
		self.p.carry = (value & 1) != 0;
		result
	}
}