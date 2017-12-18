use nes::ricoh2a03::Ricoh2A03;
use nes::ricoh2a03::InterruptType;

pub const NMI_VECTOR: u16 = 0xfffa;
pub const RESET_VECTOR: u16 = 0xfffc;
pub const IRQ_VECTOR: u16 = 0xfffe;

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
		$cpu.tick();
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

macro_rules! dec {
	($cpu: expr, $address: expr) => {
		let address = $address;
		let value = $cpu.read8(address);
		$cpu.tick();
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
		$cpu.tick();
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
		$cpu.tick();
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

macro_rules! rol {
	($cpu: expr, $address: expr) => {
		let address = $address;
		let value = $cpu.read8(address);
		let carry = $cpu.p.carry as u8;
		$cpu.p.carry = (value & 0x80) != 0;
		let result = (value << 1) | carry;
		$cpu.tick();
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
		$cpu.tick();
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

macro_rules! rti {
	($cpu: expr) => {
		$cpu.tick();
		$cpu.tick();
		let status = $cpu.pop8();
		$cpu.p.write(status);
		$cpu.pc = $cpu.pop16();
	};
}

macro_rules! rts {
	($cpu: expr) => {
		$cpu.tick();
		$cpu.tick();
		$cpu.pc = $cpu.pop16().wrapping_add(1);
		$cpu.tick();
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

macro_rules! st {
	($cpu: expr, $address: expr, $reg: expr) => {
		let address = $address;
		let value = $reg;
		$cpu.write8(address, value);
	};
}

macro_rules! stabx {
	($cpu: expr, $reg: expr) => {
		$cpu.tick();
		let address = $cpu.absx().wrapping_add($cpu.x as u16);
		let value = $reg;
		$cpu.write8(address, value);
	};
}

macro_rules! staby {
	($cpu: expr, $reg: expr) => {
		$cpu.tick();
		let address = $cpu.absy().wrapping_add($cpu.y as u16);
		let value = $reg;
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
		self.tick();

		if t != InterruptType::BRK {
			self.tick();
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

        //print!("0x{:04x}: ", pc);

        match opcode {
			0x00 => { /* println!("BRK"); */ self.interrupt(InterruptType::BRK); }
			0x01 => { /* println!("ORA ind,x"); */ ora!(self, self.indx()); },
            0x05 => { /* println!("ORA zp"); */ ora!(self, self.zp()); },
			0x06 => { /* println!("ASL zp"); */ asl!(self, self.zp()); },
			0x08 => { /* println!("PHP"); */ php!(self); },
			0x09 => { /* println!("ORA #"); */ ora!(self, self.imm()); },
			0x0a => { /* println!("ASL A"); */ asl_a!(self); },
			0x0d => { /* println!("ORA abs"); */ ora!(self, self.abs()); },
			0x0e => { /* println!("ASL abs"); */ asl!(self, self.abs()); },
            0x10 => { /* println!("BPL"); */ branch!(self, self.p.negative, false); },
			0x11 => { /* println!("ORA ind,y"); */ ora!(self, self.indy()); },
			0x15 => { /* println!("ORA zp,x"); */ ora!(self, self.zpx()); },
			0x16 => { /* println!("ASL zp,x"); */ asl!(self, self.zpx()); },
            0x18 => { /* println!("CLC"); */ flag!(self, self.p.carry, false); },
            0x19 => { /* println!("ORA abs,y"); */ ora!(self, self.absy()); },
			0x1d => { /* println!("ORA abs,x"); */ ora!(self, self.absx()); },
			0x1e => { /* println!("ASL abs,x"); */ asl!(self, self.absx()); },
            0x20 => { /* println!("JSR"); */ jsr!(self); },
            0x21 => { /* println!("AND ind,x"); */ and!(self, self.indx()); },
            0x24 => { /* println!("BIT zp"); */ bit!(self, self.zp()); },
            0x25 => { /* println!("AND zp"); */ and!(self, self.zp()); },
			0x26 => { /* println!("ROL zp"); */ rol!(self, self.zp()); },
			0x28 => { /* println!("PLP"); */ plp!(self); },
			0x29 => { /* println!("AND #"); */ and!(self, self.imm()); },
			0x2a => { /* println!("ROL A"); */ rol_a!(self); },
			0x2c => { /* println!("BIT abs"); */ bit!(self, self.abs()); },
			0x2d => { /* println!("AND abs"); */ and!(self, self.abs()); },
			0x2e => { /* println!("ROL abs"); */ rol!(self, self.abs()); },
			0x30 => { /* println!("BMI"); */ branch!(self, self.p.negative, true); },
			0x31 => { /* println!("AND ind,y"); */ and!(self, self.indy()); },
			0x35 => { /* println!("AND zp,x"); */ and!(self, self.zpx()); },
			0x36 => { /* println!("ROL zp,x"); */ rol!(self, self.zpx()); },
            0x38 => { /* println!("SEC"); */ flag!(self, self.p.carry, true); },
            0x39 => { /* println!("AND abs,y"); */ and!(self, self.absy()); },
			0x3d => { /* println!("AND abs,x"); */ and!(self, self.absx()); },
			0x3e => { /* println!("ROL abs,x"); */ rol!(self, self.absx()); },
			0x40 => { /* println!("RTI"); */ rti!(self); },
			0x41 => { /* println!("EOR ind,x"); */ eor!(self, self.indx()); },
            0x45 => { /* println!("EOR zp"); */ eor!(self, self.zp()); },
			0x46 => { /* println!("LSR zp"); */ lsr!(self, self.zp()); },
			0x48 => { /* println!("PHA"); */ pha!(self); },
			0x49 => { /* println!("EOR #"); */ eor!(self, self.imm()); },
			0x4a => { /* println!("LSR A"); */ lsr_a!(self); },
            0x4c => { /* println!("JMP abs"); */ jmp!(self); },
			0x4d => { /* println!("EOR abs"); */ eor!(self, self.abs()); },
			0x4e => { /* println!("LSR abs"); */ lsr!(self, self.abs()); },
            0x50 => { /* println!("BVC"); */ branch!(self, self.p.overflow, false); },
			0x51 => { /* println!("EOR ind,y"); */ eor!(self, self.indy()); },
			0x55 => { /* println!("EOR zp,x"); */ eor!(self, self.zpx()); },
			0x56 => { /* println!("LSR zp,x"); */ lsr!(self, self.zpx()); },
			0x58 => { /* println!("CLI"); */ flag!(self, self.p.interrupt, false); },
            0x59 => { /* println!("EOR abs,y"); */ eor!(self, self.absy()); },
			0x5d => { /* println!("EOR abs,x"); */ eor!(self, self.absx()); },
			0x5e => { /* println!("LSR abs,x"); */ lsr!(self, self.absx()); },
            0x60 => { /* println!("RTS"); */ rts!(self); },
			0x61 => { /* println!("ADC ind,x"); */ adc!(self, self.indx()); },
			0x65 => { /* println!("ADC zp"); */ adc!(self, self.zp()); },
			0x66 => { /* println!("ROR zp"); */ ror!(self, self.zp()); },
			0x68 => { /* println!("PLA"); */ pla!(self); }
			0x69 => { /* println!("ADC #"); */ adc!(self, self.imm()); },
			0x6a => { /* println!("ROR A"); */ ror_a!(self); },
			0x6c => { /* println!("JMP ind"); */ jmp_ind!(self); },
			0x6d => { /* println!("ADC abs"); */ adc!(self, self.abs()); },
			0x6e => { /* println!("ROR abs"); */ rol!(self, self.abs()); },
			0x70 => { /* println!("BVS"); */ branch!(self, self.p.overflow, true); },
			0x71 => { /* println!("ADC ind,y"); */ adc!(self, self.indy()); },
			0x75 => { /* println!("ADC zp,x"); */ adc!(self, self.zpx()); },
			0x76 => { /* println!("ROR zp,x"); */ rol!(self, self.zpx()); },
			0x78 => { /* println!("SEI"); */ flag!(self, self.p.interrupt, true); },
			0x79 => { /* println!("ADC abs,y"); */ adc!(self, self.absy()); },
			0x7d => { /* println!("ADC abs,x"); */ adc!(self, self.absx()); },
			0x7e => { /* println!("ROR abs,x"); */ rol!(self, self.absx()); },
			0x81 => { /* println!("STA ind,x"); */ st!(self, self.indx(), self.a); },
			0x84 => { /* println!("STY zp"); */ st!(self, self.zp(), self.y); },
			0x85 => { /* println!("STA zp"); */ st!(self, self.zp(), self.a); },
			0x86 => { /* println!("STX zp"); */ st!(self, self.zp(), self.x); },
			0x88 => { /* println!("DEY"); */ decr!(self, self.y); },
			0x8a => { /* println!("TXA"); */ tr!(self, self.x, self.a); },
			0x8c => { /* println!("STY abs"); */ st!(self, self.abs(), self.y); },
			0x8d => { /* println!("STA abs"); */ st!(self, self.abs(), self.a); },
			0x8e => { /* println!("STX abs"); */ st!(self, self.abs(), self.x); },
			0x90 => { /* println!("BCC"); */ branch!(self, self.p.carry, false); },
			0x91 => { /* println!("STA ind,y"); */ stindy!(self, self.a); },
			0x94 => { /* println!("STY zp,x"); */ st!(self, self.zpx(), self.y); },
			0x95 => { /* println!("STA zp,x"); */ st!(self, self.zpx(), self.a);  },
			0x96 => { /* println!("STX zp,y"); */ st!(self, self.zpy(), self.x); },
			0x98 => { /* println!("TYA"); */ tr!(self, self.y, self.a); },
			0x99 => { /* println!("STA abs,y"); */ staby!(self, self.a); },
			0x9a => { /* println!("TXS"); */ trxs!(self); },
			0x9d => { /* println!("STA abs,x"); */ stabx!(self, self.a); },
			0xa0 => { /* println!("LDY #"); */ ld!(self, self.imm(), self.y); },
			0xa1 => { /* println!("LDA ind,x"); */ ld!(self, self.indx(), self.a); },
			0xa2 => { /* println!("LDX #"); */ ld!(self, self.imm(), self.x); },
			0xa4 => { /* println!("LDY zp"); */ ld!(self, self.zp(), self.y); },
			0xa5 => { /* println!("LDA zp"); */ ld!(self, self.zp(), self.a); },
			0xa6 => { /* println!("LDX zp"); */ ld!(self, self.zp(), self.x); },
			0xa8 => { /* println!("TAY"); */ tr!(self, self.a, self.y); },
			0xa9 => { /* println!("LDA #"); */ ld!(self, self.imm(), self.a); },
			0xaa => { /* println!("TAX"); */ tr!(self, self.a, self.x); },
			0xac => { /* println!("LDY abs"); */ ld!(self, self.abs(), self.y); },
			0xad => { /* println!("LDA abs"); */ ld!(self, self.abs(), self.a); },
			0xae => { /* println!("LDX abs"); */ ld!(self, self.abs(), self.x); },
			0xb0 => { /* println!("BCS"); */ branch!(self, self.p.carry, true); },
			0xb1 => { /* println!("LDA ind,y"); */ ld!(self, self.indy(), self.a); },
			0xb4 => { /* println!("LDY zp,x"); */ ld!(self, self.zpx(), self.y); },
			0xb5 => { /* println!("LDA zp,x"); */ ld!(self, self.zpx(), self.a); },
			0xb6 => { /* println!("LDX zp,y"); */ ld!(self, self.zpy(), self.x); },
			0xb8 => { /* println!("CLV"); */ flag!(self, self.p.overflow, false); },
			0xb9 => { /* println!("LDA abs,y"); */ ld!(self, self.absy(), self.a); },
			0xba => { /* println!("TSX"); */ tr!(self, self.s, self.x); },
			0xbc => { /* println!("LDY abs,x"); */ ld!(self, self.absx(), self.y); },
			0xbd => { /* println!("LDA abs,x"); */ ld!(self, self.absx(), self.a); },
			0xbe => { /* println!("LDX abs,y"); */ ld!(self, self.absy(), self.x); },
			0xc0 => { /* println!("CPY #"); */ cmp!(self, self.imm(), self.y); },
			0xc1 => { /* println!("CMP ind,x"); */ cmp!(self, self.indx(), self.a); },
			0xc4 => { /* println!("CPY zp"); */ cmp!(self, self.zp(), self.y); },
			0xc5 => { /* println!("CMP zp"); */ cmp!(self, self.zp(), self.a); },
			0xc6 => { /* println!("DEC zp"); */ dec!(self, self.zp()); },
			0xc8 => { /* println!("INY"); */ incr!(self, self.y); },
			0xc9 => { /* println!("CMP imm"); */ cmp!(self, self.imm(), self.a); },
			0xca => { /* println!("DEX"); */ decr!(self, self.x); },
			0xcc => { /* println!("CPY abs"); */ cmp!(self, self.abs(), self.y); },
			0xcd => { /* println!("CMP abs"); */ cmp!(self, self.abs(), self.a); },
			0xce => { /* println!("DEC abs"); */ dec!(self, self.abs()); },
			0xd0 => { /* println!("BNE"); */ branch!(self, self.p.zero, false); },
			0xd1 => { /* println!("CMP ind,y"); */ cmp!(self, self.indy(), self.a); },
			0xd5 => { /* println!("CMP zp,x"); */ cmp!(self, self.zpx(), self.a); },
			0xd6 => { /* println!("DEC zp,x"); */ dec!(self, self.zpx()); },
			0xd8 => { /* println!("CLD"); */ flag!(self, self.p.decimal, false); },
			0xd9 => { /* println!("CMP abs,y"); */ cmp!(self, self.absy(), self.a); },
			0xdd => { /* println!("CMP abs,x"); */ cmp!(self, self.absx(), self.a); },
			0xde => { /* println!("DEC abs,x"); */ dec!(self, self.absx()); },
			0xe0 => { /* println!("CPX #"); */ cmp!(self, self.imm(), self.x); },
			0xe1 => { /* println!("SBC ind,x"); */ sbc!(self, self.indx()); },
			0xe4 => { /* println!("CPX zp"); */ cmp!(self, self.zp(), self.x); },
			0xe5 => { /* println!("SBC zp"); */ sbc!(self, self.zp()); },
			0xe6 => { /* println!("INC zp"); */ inc!(self, self.zp()); },
			0xe8 => { /* println!("INX"); */ incr!(self, self.x); },
			0xe9 => { /* println!("SBC #"); */ sbc!(self, self.imm()); },
			0xea => { /* println!("NOP"); */ self.tick(); },
			0xec => { /* println!("CPX abs"); */ cmp!(self, self.abs(), self.x); },
			0xed => { /* println!("SBC abs"); */ sbc!(self, self.abs()); },
			0xee => { /* println!("INC abs"); */ inc!(self, self.abs()); },
			0xf0 => { /* println!("BEQ"); */ branch!(self, self.p.zero, true); },
			0xf1 => { /* println!("SBC ind,y"); */ sbc!(self, self.indy()); },
			0xf5 => { /* println!("SBC zp,x"); */ sbc!(self, self.zpx()); },
			0xf6 => { /* println!("INC zp,x"); */ inc!(self, self.zpx()); },
			0xf8 => { /* println!("SED"); */ flag!(self, self.p.decimal, true); },
			0xf9 => { /* println!("SBC abs,y"); */ sbc!(self, self.absy()); },
			0xfd => { /* println!("SBC abs,x"); */ sbc!(self, self.absx()); },
			0xfe => { /* println!("INC abs,x"); */ inc!(self, self.absx()); },
            _ => panic!("unimplemented opcode 0x{:02x}", opcode)
        }
    }
}