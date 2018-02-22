use nes::ricoh2a03::Ricoh2A03;
use nes::ricoh2a03::InterruptType;

pub const NMI_VECTOR: u16 = 0xfffa;
pub const RESET_VECTOR: u16 = 0xfffc;
pub const IRQ_VECTOR: u16 = 0xfffe;

pub const TRACE: bool = false;

macro_rules! mnemonic {
	($address: expr, $mnemonic: expr) => {
		if TRACE {
			println!("0x{:04x}: {}", $address, $mnemonic);
		}
	};
}

macro_rules! and {
	($cpu: expr, $address: expr) => {
		let address = $address;
		let value = $cpu.read8(address);
		let result = $cpu.a & value;
		$cpu.a = result;
		$cpu.set_nz(result);
	};
}

macro_rules! adc {
	($cpu: expr, $address: expr) => {
		let address = $address;
		let value = $cpu.read8(address);
		let carry = $cpu.p.carry as u16;
		let result = ($cpu.a as u16) + (value as u16) + carry;

		$cpu.p.carry = result > 0xff;
		$cpu.p.overflow = (($cpu.a ^ result as u8) & !($cpu.a ^ value) & 0x80) != 0;
		
		$cpu.a = result as u8;
		$cpu.set_nz(result as u8);
	};
}

macro_rules! asl {
	($cpu: expr, $address: expr) => {
		let address = $address;
		let value = $cpu.read8(address);
		$cpu.p.carry = (value & 0x80) != 0;
		$cpu.write8(address, value);
		$cpu.set_nz(value << 1);
		$cpu.write8(address, value << 1);
	};
}

macro_rules! asl_a {
	($cpu: expr) => {
		$cpu.p.carry = ($cpu.a & 0x80) != 0;
		let result = $cpu.a << 1;
		$cpu.a = result;
		$cpu.set_nz(result);
		$cpu.tick();
	};
}

macro_rules! bit {
	($cpu: expr, $address: expr) => {
		let address = $address;
		let value = $cpu.read8(address);
		$cpu.p.negative = (value & 0x80) != 0;
		$cpu.p.overflow = (value & 0x40) != 0;
		$cpu.p.zero = ($cpu.a & value) == 0;
	};
}

macro_rules! branch {
	($cpu: expr, $flag: expr, $condition: expr) => {
		let address = $cpu.imm();
		let jump = $cpu.read8(address) as i8 as u16;
		
		if $flag == $condition {
			$cpu.tick();
			let new_pc = $cpu.pc.wrapping_add(jump);

			if ($cpu.pc & 0xff00) != (new_pc & 0xff00) {
				$cpu.tick();
			}

			$cpu.pc = new_pc;
		}
	};
}

macro_rules! cmp {
	($cpu: expr, $address: expr, $reg: expr) => {
		let address = $address;
		let value = $cpu.read8(address);
		let result = $reg.wrapping_sub(value);
		$cpu.set_nz(result);
		$cpu.p.carry = $reg >= value;
	};
}

macro_rules! dcp {
	($cpu: expr, $address: expr) => {
		let address = $address;
		let value = $cpu.read8(address);

		$cpu.write8(address, value);

		let result = value.wrapping_sub(1);
		$cpu.write8(address, result);

		let result2 = $cpu.a.wrapping_sub(result);
		$cpu.set_nz(result2);
		$cpu.p.carry = $cpu.a >= result2;
	};
}

macro_rules! dec {
	($cpu: expr, $address: expr) => {
		let address = $address;
		let value = $cpu.read8(address);
		$cpu.write8(address, value);
		let result = value.wrapping_sub(1);
		$cpu.set_nz(result);
		$cpu.write8(address, result);
	};
}

macro_rules! decr {
	($cpu: expr, $reg: expr) => {
		let reg = $reg.wrapping_sub(1);
		$reg = reg;
		$cpu.set_nz(reg);
		$cpu.tick();
	};
}

macro_rules! eor {
	($cpu: expr, $address: expr) => {
		let address = $address;
		let value = $cpu.read8(address);
		let result = $cpu.a ^ value;
		$cpu.a = result;
		$cpu.set_nz(result);
	};
}

macro_rules! flag {
	($cpu: expr, $flag: expr, $value: expr) => {
		$flag = $value;
		$cpu.tick();
	};
}

macro_rules! inc {
	($cpu: expr, $address: expr) => {
		let address = $address;
		let value = $cpu.read8(address);
		$cpu.write8(address, value);
		let result = value.wrapping_add(1);
		$cpu.set_nz(result);
		$cpu.write8(address, result);
	};
}

macro_rules! incr {
	($cpu: expr, $reg: expr) => {
		let reg = $reg.wrapping_add(1);
		$reg = reg;
		$cpu.set_nz(reg);
		$cpu.tick();
	};
}

macro_rules! isc {
	($cpu: expr, $address: expr) => {
		let address = $address;
		let value = $cpu.read8(address);

		$cpu.write8(address, value);

		let result = value.wrapping_add(1);
		$cpu.write8(address, result);

		let value2 = result ^ 0xff;
		let carry = $cpu.p.carry as u16;
		let result2 = ($cpu.a as u16) + (value2 as u16) + carry;

		$cpu.p.carry = result2 > 0xff;
		$cpu.p.overflow = (($cpu.a ^ result2 as u8) & !($cpu.a ^ value2) & 0x80) != 0;
		
		$cpu.a = result2 as u8;
		$cpu.set_nz(result2 as u8);
	};
}

macro_rules! jmp {
	($cpu: expr) => {
		let address = $cpu.imm();
		$cpu.pc = $cpu.read16(address);
	};
}

macro_rules! jmp_ind {
	($cpu: expr) => {
		let address = $cpu.imm();
		let value = $cpu.read16(address);
		$cpu.pc = $cpu.read16_d(value, (value & 0xff00) | (value.wrapping_add(1) & 0xff));
	};
}

macro_rules! jsr {
	($cpu: expr) => {
		$cpu.tick();
		let pc = $cpu.pc;
		$cpu.push16(pc.wrapping_add(1));
		let address = $cpu.imm();
		$cpu.pc = $cpu.read16(address);
	};
}

macro_rules! lax {
	($cpu: expr, $address: expr) => {
		let address = $address;
		let value = $cpu.read8(address);
		$cpu.a = value;
		$cpu.x = value;
		$cpu.set_nz(value);	
	};
}

macro_rules! ld {
	($cpu: expr, $address: expr, $reg: expr) => {
		let address = $address;
		let value = $cpu.read8(address);
		$reg = value;
		$cpu.set_nz(value);
	};
}

macro_rules! lsr {
	($cpu: expr, $address: expr) => {
		let address = $address;
		let value = $cpu.read8(address);
		$cpu.p.carry = (value & 0x1) != 0;
		$cpu.write8(address, value);
		$cpu.set_nz(value >> 1);
		$cpu.write8(address, value >> 1);
	};
}

macro_rules! lsr_a {
	($cpu: expr) => {
		$cpu.p.carry = ($cpu.a & 0x1) != 0;
		let result = $cpu.a >> 1;
		$cpu.a = result;
		$cpu.set_nz(result);
		$cpu.tick();
	};
}

macro_rules! nop {
	($cpu: expr, $address: expr) => {
		let address = $address;
		$cpu.read8(address);
	};
}

macro_rules! ora {
	($cpu: expr, $address: expr) => {
		let address = $address;
		let value = $cpu.read8(address);
		let result = $cpu.a | value;
		$cpu.a = result;
		$cpu.set_nz(result);
	};
}

macro_rules! pha {
	($cpu: expr) => {
		$cpu.tick();
		let a = $cpu.a;
		$cpu.push8(a);
	};
}

macro_rules! php {
	($cpu: expr) => {
		$cpu.tick();
		let status = $cpu.p.read();
		let brk_flag = 1 << 4;
		$cpu.push8(status | brk_flag);
	};
}

macro_rules! pla {
	($cpu: expr) => {
		$cpu.tick();
		$cpu.tick();
		let a = $cpu.pop8();
		$cpu.a = a;
		$cpu.set_nz(a);
	};
}

macro_rules! plp {
	($cpu: expr) => {
		$cpu.tick();
		$cpu.tick();
		let status = $cpu.pop8();
		$cpu.p.write(status);
	};
}

macro_rules! rla {
	($cpu: expr, $address: expr) => {
		let address = $address;
		let value = $cpu.read8(address);

		let carry = $cpu.p.carry as u8;
		$cpu.p.carry = (value & 0x80) != 0;
		$cpu.write8(address, value);

		let result = (value << 1) | carry;
		$cpu.write8(address, result);

		let result2 = $cpu.a & result;
		$cpu.a = result2;
		$cpu.set_nz(result2);
	};
}

macro_rules! rol {
	($cpu: expr, $address: expr) => {
		let address = $address;
		let value = $cpu.read8(address);
		let carry = $cpu.p.carry as u8;
		$cpu.p.carry = (value & 0x80) != 0;
		let result = (value << 1) | carry;
		$cpu.write8(address, value);
		$cpu.set_nz(result);
		$cpu.write8(address, result);
	};
}

macro_rules! rol_a {
	($cpu: expr) => {
		let carry = $cpu.p.carry as u8;
		$cpu.p.carry = ($cpu.a & 0x80) != 0;
		let result = ($cpu.a << 1) | carry;
		$cpu.a = result;
		$cpu.set_nz(result);
		$cpu.tick();
	};
}

macro_rules! ror {
	($cpu: expr, $address: expr) => {
		let address = $address;
		let value = $cpu.read8(address);
		let carry = $cpu.p.carry as u8;
		$cpu.p.carry = (value & 0x1) != 0;
		let result = (value >> 1) | (carry << 7);
		$cpu.write8(address, value);
		$cpu.set_nz(result);
		$cpu.write8(address, result);
	};
}

macro_rules! ror_a {
	($cpu: expr) => {
		let carry = $cpu.p.carry as u8;
		$cpu.p.carry = ($cpu.a & 0x1) != 0;
		let result = ($cpu.a >> 1) | (carry << 7);
		$cpu.a = result;
		$cpu.set_nz(result);
		$cpu.tick();
	};
}

macro_rules! rra {
	($cpu: expr, $address: expr) => {
		let address = $address;
		let value = $cpu.read8(address);

		let carry = $cpu.p.carry as u8;
		$cpu.p.carry = (value & 0x1) != 0;

		let result = (value >> 1) | (carry << 7);
		$cpu.write8(address, value);
		$cpu.write8(address, result);

		let carry = $cpu.p.carry as u16;
		let result2 = ($cpu.a as u16) + (result as u16) + carry;

		$cpu.p.carry = result2 > 0xff;
		$cpu.p.overflow = (($cpu.a ^ result2 as u8) & !($cpu.a ^ result) & 0x80) != 0;
		
		$cpu.a = result2 as u8;
		$cpu.set_nz(result2 as u8);
	};
}

macro_rules! rti {
	($cpu: expr) => {
		let imm = $cpu.imm();
		$cpu.read8(imm);
		$cpu.tick();
		let status = $cpu.pop8();
		$cpu.p.write(status);
		$cpu.pc = $cpu.pop16();
	};
}

macro_rules! rts {
	($cpu: expr) => {
		let imm = $cpu.imm();
		$cpu.read8(imm);
		$cpu.tick();
		$cpu.pc = $cpu.pop16().wrapping_add(1);
		$cpu.tick();
	};
}

macro_rules! sax {
	($cpu: expr, $address: expr) => {
		let address = $address;
		let value = $cpu.a & $cpu.x;
		$cpu.write8(address, value);	
	};
}

macro_rules! sbc {
	($cpu: expr, $address: expr) => {
		let address = $address;
		let value = $cpu.read8(address) ^ 0xff;
		let carry = $cpu.p.carry as u16;
		let result = ($cpu.a as u16) + (value as u16) + carry;

		$cpu.p.carry = result > 0xff;
		$cpu.p.overflow = (($cpu.a ^ result as u8) & !($cpu.a ^ value) & 0x80) != 0;
		
		$cpu.a = result as u8;
		$cpu.set_nz(result as u8);
	};
}

macro_rules! slo {
	($cpu: expr, $address: expr) => {
		let address = $address;
		let value = $cpu.read8(address);

		$cpu.p.carry = (value & 0x80) != 0;
		$cpu.write8(address, value);

		let result = value << 1;
		$cpu.write8(address, result);

		let result2 = $cpu.a | result;
		$cpu.a = result2;
		$cpu.set_nz(result2);
	};
}

macro_rules! sre {
	($cpu: expr, $address: expr) => {
		let address = $address;
		let value = $cpu.read8(address);

		$cpu.p.carry = (value & 0x1) != 0;
		$cpu.write8(address, value);

		let result = value >> 1;
		$cpu.write8(address, result);

		let result2 = $cpu.a ^ result;
		$cpu.a = result2;
		$cpu.set_nz(result2);
	};
}

macro_rules! st {
	($cpu: expr, $address: expr, $reg: expr) => {
		let address = $address;
		let value = $reg;
		$cpu.write8(address, value);
	};
}

macro_rules! stabx {
	($cpu: expr, $reg: expr) => {
		let abs = $cpu.abs();
		let address = abs.wrapping_add($cpu.x as u16);
		let address2 = (abs & 0xff00) + $cpu.x as u16;
		let value = $reg;
		$cpu.read8(address2);
		$cpu.write8(address, value);
	};
}

macro_rules! staby {
	($cpu: expr, $reg: expr) => {
		let abs = $cpu.abs();
		let address = abs.wrapping_add($cpu.y as u16);
		let address2 = (abs & 0xff00) + $cpu.y as u16;
		let value = $reg;
		$cpu.read8(address2);
		$cpu.write8(address, value);
	};
}

macro_rules! stindy {
	($cpu: expr, $reg: expr) => {
		$cpu.tick();
		let address = $cpu._indy();
		let value = $reg;
		$cpu.write8(address, value);
	};
}

macro_rules! tr {
	($cpu: expr, $src: expr, $dest: expr) => {
		let src = $src;
		$dest = src;
		$cpu.set_nz(src);
		$cpu.tick();
	};
}

macro_rules! trxs {
	($cpu: expr) => {
		$cpu.s = $cpu.x;
		$cpu.tick();
	};
}

impl Ricoh2A03 {
	pub fn interrupt(&mut self, t: InterruptType) {
		if t != InterruptType::BRK {
			let addr = self.pc;
			self.read8(addr);
			self.read8(addr);
		} else {
			let addr = self.imm();
			self.read8(addr);
		}

		if t != InterruptType::RESET {
			let pc = self.pc;
			self.push16(pc);
			let brk_bit = ((t == InterruptType::BRK) as u8) << 4;
			let status = self.p.read();
			self.push8(status | brk_bit);
		} else {
			self.s = self.s.wrapping_sub(3);
			self.tick();
			self.tick();
			self.tick();
		}

		self.p.interrupt = true;
		
		let vector = match t {
			InterruptType::NMI => NMI_VECTOR,
			InterruptType::RESET => RESET_VECTOR,
			InterruptType::IRQ => IRQ_VECTOR,
			InterruptType::BRK => IRQ_VECTOR,
		};
		
		self.pc = self.read16(vector);
	}

	pub fn step(&mut self) {
		let pc = self.imm();
        let opcode = self.read8(pc);

        match opcode {
			0x00 => { mnemonic!(pc, "BRK"); self.interrupt(InterruptType::BRK); },
			0x01 => { mnemonic!(pc, "ORA ind,x"); ora!(self, self.indx()); },
			0x03 => { mnemonic!(pc, "SLO ind,x"); slo!(self, self.indx()); },
			0x04 => { mnemonic!(pc, "NOP"); nop!(self, self.zp()); },
            0x05 => { mnemonic!(pc, "ORA zp"); ora!(self, self.zp()); },
			0x06 => { mnemonic!(pc, "ASL zp"); asl!(self, self.zp()); },
			0x07 => { mnemonic!(pc, "SLO zp"); slo!(self, self.zp()); },
			0x08 => { mnemonic!(pc, "PHP"); php!(self); },
			0x09 => { mnemonic!(pc, "ORA #"); ora!(self, self.imm()); },
			0x0a => { mnemonic!(pc, "ASL A"); asl_a!(self); },
			0x0c => { mnemonic!(pc, "NOP"); nop!(self, self.abs()); },
			0x0d => { mnemonic!(pc, "ORA abs"); ora!(self, self.abs()); },
			0x0e => { mnemonic!(pc, "ASL abs"); asl!(self, self.abs()); },
			0x0f => { mnemonic!(pc, "SLO abs"); slo!(self, self.abs()); },
            0x10 => { mnemonic!(pc, "BPL"); branch!(self, self.p.negative, false); },
			0x11 => { mnemonic!(pc, "ORA ind,y"); ora!(self, self.indy()); },
			0x13 => { mnemonic!(pc, "SLO ind,y"); slo!(self, self.indy()); },
			0x14 => { mnemonic!(pc, "NOP"); nop!(self, self.zpx()); },
			0x15 => { mnemonic!(pc, "ORA zp,x"); ora!(self, self.zpx()); },
			0x16 => { mnemonic!(pc, "ASL zp,x"); asl!(self, self.zpx()); },
			0x17 => { mnemonic!(pc, "SLO zp,x"); slo!(self, self.zpx()); },
            0x18 => { mnemonic!(pc, "CLC"); flag!(self, self.p.carry, false); },
            0x19 => { mnemonic!(pc, "ORA abs,y"); ora!(self, self.absy()); },
			0x1a => { mnemonic!(pc, "NOP"); nop!(self, pc); },
			0x1b => { mnemonic!(pc, "SLO abs,x"); slo!(self, self.absy()); },
			0x1c => { mnemonic!(pc, "NOP"); nop!(self, self.absx()); },
			0x1d => { mnemonic!(pc, "ORA abs,x"); ora!(self, self.absx()); },
			0x1e => { mnemonic!(pc, "ASL abs,x"); asl!(self, self._absx()); },
			0x1f => { mnemonic!(pc, "SLO abs,x"); slo!(self, self.absx()); },
            0x20 => { mnemonic!(pc, "JSR"); jsr!(self); },
            0x21 => { mnemonic!(pc, "AND ind,x"); and!(self, self.indx()); },
			0x23 => { mnemonic!(pc, "RLA ind,x"); rla!(self, self.indx()); },
            0x24 => { mnemonic!(pc, "BIT zp"); bit!(self, self.zp()); },
            0x25 => { mnemonic!(pc, "AND zp"); and!(self, self.zp()); },
			0x26 => { mnemonic!(pc, "ROL zp"); rol!(self, self.zp()); },
			0x27 => { mnemonic!(pc, "RLA zp"); rla!(self, self.zp()); },
			0x28 => { mnemonic!(pc, "PLP"); plp!(self); },
			0x29 => { mnemonic!(pc, "AND #"); and!(self, self.imm()); },
			0x2a => { mnemonic!(pc, "ROL A"); rol_a!(self); },
			0x2c => { mnemonic!(pc, "BIT abs"); bit!(self, self.abs()); },
			0x2d => { mnemonic!(pc, "AND abs"); and!(self, self.abs()); },
			0x2e => { mnemonic!(pc, "ROL abs"); rol!(self, self.abs()); },
			0x2f => { mnemonic!(pc, "RLA abs"); rla!(self, self.abs()); },
			0x30 => { mnemonic!(pc, "BMI"); branch!(self, self.p.negative, true); },
			0x31 => { mnemonic!(pc, "AND ind,y"); and!(self, self.indy()); },
			0x33 => { mnemonic!(pc, "RLA ind,y"); rla!(self, self.indy()); },
			0x34 => { mnemonic!(pc, "NOP"); nop!(self, self.zpx()); },
			0x35 => { mnemonic!(pc, "AND zp,x"); and!(self, self.zpx()); },
			0x36 => { mnemonic!(pc, "ROL zp,x"); rol!(self, self.zpx()); },
			0x37 => { mnemonic!(pc, "RLA zp,x"); rla!(self, self.zpx()); },
            0x38 => { mnemonic!(pc, "SEC"); flag!(self, self.p.carry, true); },
            0x39 => { mnemonic!(pc, "AND abs,y"); and!(self, self.absy()); },
			0x3a => { mnemonic!(pc, "NOP"); nop!(self, pc); },
			0x3b => { mnemonic!(pc, "RLA abs,x"); rla!(self, self.absy()); },
			0x3c => { mnemonic!(pc, "NOP"); nop!(self, self.absx()); },
			0x3d => { mnemonic!(pc, "AND abs,x"); and!(self, self.absx()); },
			0x3e => { mnemonic!(pc, "ROL abs,x"); rol!(self, self._absx()); },
			0x3f => { mnemonic!(pc, "RLA abs,x"); rla!(self, self.absx()); },
			0x40 => { mnemonic!(pc, "RTI"); rti!(self); },
			0x41 => { mnemonic!(pc, "EOR ind,x"); eor!(self, self.indx()); },
			0x43 => { mnemonic!(pc, "SRE ind,x"); sre!(self, self.indx()); },
			0x44 => { mnemonic!(pc, "NOP"); nop!(self, self.zp()); },
            0x45 => { mnemonic!(pc, "EOR zp"); eor!(self, self.zp()); },
			0x46 => { mnemonic!(pc, "LSR zp"); lsr!(self, self.zp()); },
			0x47 => { mnemonic!(pc, "SRE zp"); sre!(self, self.zp()); },
			0x48 => { mnemonic!(pc, "PHA"); pha!(self); },
			0x49 => { mnemonic!(pc, "EOR #"); eor!(self, self.imm()); },
			0x4a => { mnemonic!(pc, "LSR A"); lsr_a!(self); },
            0x4c => { mnemonic!(pc, "JMP abs"); jmp!(self); },
			0x4d => { mnemonic!(pc, "EOR abs"); eor!(self, self.abs()); },
			0x4e => { mnemonic!(pc, "LSR abs"); lsr!(self, self.abs()); },
			0x4f => { mnemonic!(pc, "SRE abs"); sre!(self, self.abs()); },
            0x50 => { mnemonic!(pc, "BVC"); branch!(self, self.p.overflow, false); },
			0x51 => { mnemonic!(pc, "EOR ind,y"); eor!(self, self.indy()); },
			0x53 => { mnemonic!(pc, "SRE ind,y"); sre!(self, self.indy()); },
			0x54 => { mnemonic!(pc, "NOP"); nop!(self, self.zpx()); },
			0x55 => { mnemonic!(pc, "EOR zp,x"); eor!(self, self.zpx()); },
			0x56 => { mnemonic!(pc, "LSR zp,x"); lsr!(self, self.zpx()); },
			0x57 => { mnemonic!(pc, "SRE zp,x"); sre!(self, self.zpx()); },
			0x58 => { mnemonic!(pc, "CLI"); flag!(self, self.p.interrupt, false); },
            0x59 => { mnemonic!(pc, "EOR abs,y"); eor!(self, self.absy()); },
			0x5a => { mnemonic!(pc, "NOP"); nop!(self, pc); },
			0x5b => { mnemonic!(pc, "SRE abs,x"); sre!(self, self.absy()); },
			0x5c => { mnemonic!(pc, "NOP"); nop!(self, self.absx()); },
			0x5d => { mnemonic!(pc, "EOR abs,x"); eor!(self, self.absx()); },
			0x5e => { mnemonic!(pc, "LSR abs,x"); lsr!(self, self._absx()); },
			0x5f => { mnemonic!(pc, "SRE abs,x"); sre!(self, self.absx()); },
            0x60 => { mnemonic!(pc, "RTS"); rts!(self); },
			0x61 => { mnemonic!(pc, "ADC ind,x"); adc!(self, self.indx()); },
			0x63 => { mnemonic!(pc, "RRA ind,x"); rra!(self, self.indx()); },
			0x64 => { mnemonic!(pc, "NOP"); nop!(self, self.zp()); },
			0x65 => { mnemonic!(pc, "ADC zp"); adc!(self, self.zp()); },
			0x66 => { mnemonic!(pc, "ROR zp"); ror!(self, self.zp()); },
			0x67 => { mnemonic!(pc, "RRA zp"); rra!(self, self.zp()); },
			0x68 => { mnemonic!(pc, "PLA"); pla!(self); }
			0x69 => { mnemonic!(pc, "ADC #"); adc!(self, self.imm()); },
			0x6a => { mnemonic!(pc, "ROR A"); ror_a!(self); },
			0x6c => { mnemonic!(pc, "JMP ind"); jmp_ind!(self); },
			0x6d => { mnemonic!(pc, "ADC abs"); adc!(self, self.abs()); },
			0x6e => { mnemonic!(pc, "ROR abs"); ror!(self, self.abs()); },
			0x6f => { mnemonic!(pc, "RRA abs"); rra!(self, self.abs()); },
			0x70 => { mnemonic!(pc, "BVS"); branch!(self, self.p.overflow, true); },
			0x71 => { mnemonic!(pc, "ADC ind,y"); adc!(self, self.indy()); },
			0x73 => { mnemonic!(pc, "RRA ind,y"); rra!(self, self.indy()); },
			0x74 => { mnemonic!(pc, "NOP"); nop!(self, self.zpx()); },
			0x75 => { mnemonic!(pc, "ADC zp,x"); adc!(self, self.zpx()); },
			0x76 => { mnemonic!(pc, "ROR zp,x"); ror!(self, self.zpx()); },
			0x77 => { mnemonic!(pc, "RRA zp,x"); rra!(self, self.zpx()); },
			0x78 => { mnemonic!(pc, "SEI"); flag!(self, self.p.interrupt, true); },
			0x79 => { mnemonic!(pc, "ADC abs,y"); adc!(self, self.absy()); },
			0x7a => { mnemonic!(pc, "NOP"); nop!(self, pc); },
			0x7b => { mnemonic!(pc, "RRA abs,x"); rra!(self, self.absy()); },
			0x7c => { mnemonic!(pc, "NOP"); nop!(self, self.absx()); },
			0x7d => { mnemonic!(pc, "ADC abs,x"); adc!(self, self.absx()); },
			0x7e => { mnemonic!(pc, "ROR abs,x"); ror!(self, self._absx()); },
			0x7f => { mnemonic!(pc, "RRA abs,x"); rra!(self, self.absx()); },
			0x80 => { mnemonic!(pc, "NOP"); nop!(self, self.imm()); },
			0x81 => { mnemonic!(pc, "STA ind,x"); st!(self, self.indx(), self.a); },
			0x82 => { mnemonic!(pc, "NOP"); nop!(self, self.imm()); },
			0x83 => { mnemonic!(pc, "SAX ind,x"); sax!(self, self.indx()); },
			0x84 => { mnemonic!(pc, "STY zp"); st!(self, self.zp(), self.y); },
			0x85 => { mnemonic!(pc, "STA zp"); st!(self, self.zp(), self.a); },
			0x86 => { mnemonic!(pc, "STX zp"); st!(self, self.zp(), self.x); },
			0x87 => { mnemonic!(pc, "SAX zp"); sax!(self, self.zp()); },
			0x88 => { mnemonic!(pc, "DEY"); decr!(self, self.y); },
			0x89 => { mnemonic!(pc, "NOP"); nop!(self, self.imm()); },
			0x8a => { mnemonic!(pc, "TXA"); tr!(self, self.x, self.a); },
			0x8c => { mnemonic!(pc, "STY abs"); st!(self, self.abs(), self.y); },
			0x8d => { mnemonic!(pc, "STA abs"); st!(self, self.abs(), self.a); },
			0x8e => { mnemonic!(pc, "STX abs"); st!(self, self.abs(), self.x); },
			0x8f => { mnemonic!(pc, "SAX abs"); sax!(self, self.abs()); },
			0x90 => { mnemonic!(pc, "BCC"); branch!(self, self.p.carry, false); },
			0x91 => { mnemonic!(pc, "STA ind,y"); stindy!(self, self.a); },
			0x94 => { mnemonic!(pc, "STY zp,x"); st!(self, self.zpx(), self.y); },
			0x95 => { mnemonic!(pc, "STA zp,x"); st!(self, self.zpx(), self.a);  },
			0x96 => { mnemonic!(pc, "STX zp,y"); st!(self, self.zpy(), self.x); },
			0x97 => { mnemonic!(pc, "SAX zp,y"); sax!(self, self.zpy()); },
			0x98 => { mnemonic!(pc, "TYA"); tr!(self, self.y, self.a); },
			0x99 => { mnemonic!(pc, "STA abs,y"); staby!(self, self.a); },
			0x9a => { mnemonic!(pc, "TXS"); trxs!(self); },
			0x9d => { mnemonic!(pc, "STA abs,x"); stabx!(self, self.a); },
			0xa0 => { mnemonic!(pc, "LDY #"); ld!(self, self.imm(), self.y); },
			0xa1 => { mnemonic!(pc, "LDA ind,x"); ld!(self, self.indx(), self.a); },
			0xa2 => { mnemonic!(pc, "LDX #"); ld!(self, self.imm(), self.x); },
			0xa3 => { mnemonic!(pc, "LAX ind,x"); lax!(self, self.indx()); },
			0xa4 => { mnemonic!(pc, "LDY zp"); ld!(self, self.zp(), self.y); },
			0xa5 => { mnemonic!(pc, "LDA zp"); ld!(self, self.zp(), self.a); },
			0xa6 => { mnemonic!(pc, "LDX zp"); ld!(self, self.zp(), self.x); },
			0xa7 => { mnemonic!(pc, "LAX zp"); lax!(self, self.zp()); },
			0xa8 => { mnemonic!(pc, "TAY"); tr!(self, self.a, self.y); },
			0xa9 => { mnemonic!(pc, "LDA #"); ld!(self, self.imm(), self.a); },
			0xaa => { mnemonic!(pc, "TAX"); tr!(self, self.a, self.x); },
			0xac => { mnemonic!(pc, "LDY abs"); ld!(self, self.abs(), self.y); },
			0xad => { mnemonic!(pc, "LDA abs"); ld!(self, self.abs(), self.a); },
			0xae => { mnemonic!(pc, "LDX abs"); ld!(self, self.abs(), self.x); },
			0xaf => { mnemonic!(pc, "LAX abs"); lax!(self, self.abs()); },
			0xb0 => { mnemonic!(pc, "BCS"); branch!(self, self.p.carry, true); },
			0xb1 => { mnemonic!(pc, "LDA ind,y"); ld!(self, self.indy(), self.a); },
			0xb3 => { mnemonic!(pc, "LAX ind,y"); lax!(self, self.indy()); },
			0xb4 => { mnemonic!(pc, "LDY zp,x"); ld!(self, self.zpx(), self.y); },
			0xb5 => { mnemonic!(pc, "LDA zp,x"); ld!(self, self.zpx(), self.a); },
			0xb6 => { mnemonic!(pc, "LDX zp,y"); ld!(self, self.zpy(), self.x); },
			0xb7 => { mnemonic!(pc, "LAX zp,y"); lax!(self, self.zpy()); },
			0xb8 => { mnemonic!(pc, "CLV"); flag!(self, self.p.overflow, false); },
			0xb9 => { mnemonic!(pc, "LDA abs,y"); ld!(self, self.absy(), self.a); },
			0xba => { mnemonic!(pc, "TSX"); tr!(self, self.s, self.x); },
			0xbc => { mnemonic!(pc, "LDY abs,x"); ld!(self, self.absx(), self.y); },
			0xbd => { mnemonic!(pc, "LDA abs,x"); ld!(self, self.absx(), self.a); },
			0xbe => { mnemonic!(pc, "LDX abs,y"); ld!(self, self.absy(), self.x); },
			0xbf => { mnemonic!(pc, "LAX abs,y"); lax!(self, self.absy()); },
			0xc0 => { mnemonic!(pc, "CPY #"); cmp!(self, self.imm(), self.y); },
			0xc1 => { mnemonic!(pc, "CMP ind,x"); cmp!(self, self.indx(), self.a); },
			0xc2 => { mnemonic!(pc, "NOP"); nop!(self, self.imm()); },
			0xc3 => { mnemonic!(pc, "DCP ind,x"); dcp!(self, self.indx()); }
			0xc4 => { mnemonic!(pc, "CPY zp"); cmp!(self, self.zp(), self.y); },
			0xc5 => { mnemonic!(pc, "CMP zp"); cmp!(self, self.zp(), self.a); },
			0xc6 => { mnemonic!(pc, "DEC zp"); dec!(self, self.zp()); },
			0xc7 => { mnemonic!(pc, "DCP zp"); dcp!(self, self.zp()); },
			0xc8 => { mnemonic!(pc, "INY"); incr!(self, self.y); },
			0xc9 => { mnemonic!(pc, "CMP #"); cmp!(self, self.imm(), self.a); },
			0xca => { mnemonic!(pc, "DEX"); decr!(self, self.x); },
			0xcc => { mnemonic!(pc, "CPY abs"); cmp!(self, self.abs(), self.y); },
			0xcd => { mnemonic!(pc, "CMP abs"); cmp!(self, self.abs(), self.a); },
			0xce => { mnemonic!(pc, "DEC abs"); dec!(self, self.abs()); },
			0xcf => { mnemonic!(pc, "DCP abs"); dcp!(self, self.abs()); },
			0xd0 => { mnemonic!(pc, "BNE"); branch!(self, self.p.zero, false); },
			0xd1 => { mnemonic!(pc, "CMP ind,y"); cmp!(self, self.indy(), self.a); },
			0xd3 => { mnemonic!(pc, "DCP ind,y"); dcp!(self, self.indy()); },
			0xd4 => { mnemonic!(pc, "NOP"); nop!(self, self.zpx()); },
			0xd5 => { mnemonic!(pc, "CMP zp,x"); cmp!(self, self.zpx(), self.a); },
			0xd6 => { mnemonic!(pc, "DEC zp,x"); dec!(self, self.zpx()); },
			0xd7 => { mnemonic!(pc, "DCP zp,x"); dcp!(self, self.zpx()); },
			0xd8 => { mnemonic!(pc, "CLD"); flag!(self, self.p.decimal, false); },
			0xd9 => { mnemonic!(pc, "CMP abs,y"); cmp!(self, self.absy(), self.a); },
			0xda => { mnemonic!(pc, "NOP"); nop!(self, pc); },
			0xdb => { mnemonic!(pc, "DCP abs,y"); dcp!(self, self.absy()); },
			0xdc => { mnemonic!(pc, "NOP"); nop!(self, self.absx()); },
			0xdd => { mnemonic!(pc, "CMP abs,x"); cmp!(self, self.absx(), self.a); },
			0xde => { mnemonic!(pc, "DEC abs,x"); dec!(self, self._absx()); },
			0xdf => { mnemonic!(pc, "DCP abs,x"); dcp!(self, self.absx()); },
			0xe0 => { mnemonic!(pc, "CPX #"); cmp!(self, self.imm(), self.x); },
			0xe1 => { mnemonic!(pc, "SBC ind,x"); sbc!(self, self.indx()); },
			0xe2 => { mnemonic!(pc, "NOP"); nop!(self, self.imm()); },
			0xe3 => { mnemonic!(pc, "ISC ind,x"); isc!(self, self.indx()); },
			0xe4 => { mnemonic!(pc, "CPX zp"); cmp!(self, self.zp(), self.x); },
			0xe5 => { mnemonic!(pc, "SBC zp"); sbc!(self, self.zp()); },
			0xe6 => { mnemonic!(pc, "INC zp"); inc!(self, self.zp()); },
			0xe7 => { mnemonic!(pc, "ISC zp"); isc!(self, self.zp()); },
			0xe8 => { mnemonic!(pc, "INX"); incr!(self, self.x); },
			0xe9 => { mnemonic!(pc, "SBC #"); sbc!(self, self.imm()); },
			0xea => { mnemonic!(pc, "NOP"); nop!(self, pc); },
			0xeb => { mnemonic!(pc, "SBC #"); sbc!(self, self.imm()); },
			0xec => { mnemonic!(pc, "CPX abs"); cmp!(self, self.abs(), self.x); },
			0xed => { mnemonic!(pc, "SBC abs"); sbc!(self, self.abs()); },
			0xee => { mnemonic!(pc, "INC abs"); inc!(self, self.abs()); },
			0xef => { mnemonic!(pc, "ISC abs"); isc!(self, self.abs()); },
			0xf0 => { mnemonic!(pc, "BEQ"); branch!(self, self.p.zero, true); },
			0xf1 => { mnemonic!(pc, "SBC ind,y"); sbc!(self, self.indy()); },
			0xf3 => { mnemonic!(pc, "ISC ind,y"); isc!(self, self.indy()); },
			0xf4 => { mnemonic!(pc, "NOP"); nop!(self, self.zpx()); },
			0xf5 => { mnemonic!(pc, "SBC zp,x"); sbc!(self, self.zpx()); },
			0xf6 => { mnemonic!(pc, "INC zp,x"); inc!(self, self.zpx()); },
			0xf7 => { mnemonic!(pc, "ISC zp,x"); isc!(self, self.zpx()); },
			0xf8 => { mnemonic!(pc, "SED"); flag!(self, self.p.decimal, true); },
			0xf9 => { mnemonic!(pc, "SBC abs,y"); sbc!(self, self.absy()); },
			0xfa => { mnemonic!(pc, "NOP"); nop!(self, pc); },
			0xfb => { mnemonic!(pc, "ISC abs,y"); isc!(self, self.absy()); },
			0xfc => { mnemonic!(pc, "NOP"); nop!(self, self.absx()); },
			0xfd => { mnemonic!(pc, "SBC abs,x"); sbc!(self, self.absx()); },
			0xfe => { mnemonic!(pc, "INC abs,x"); inc!(self, self._absx()); },
			0xff => { mnemonic!(pc, "ISC abs,x"); isc!(self, self.absx()); },
            _ => println!("unimplemented opcode 0x{:02x}", opcode)
        }
    }
}