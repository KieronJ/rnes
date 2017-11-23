mod instructions;
mod load;
mod status;
mod store;

use nes::bus::Bus;
use nes::ricoh2a03::status::Status;

pub struct Ricoh2A03 {
    pc: u16,
    a: u8,
    x: u8,
    y: u8,
    s: u8,
    p: Status,
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
            p: Status::new(),
            bus: bus,
        };

        cpu.reset();
        cpu
    }

    fn push8(&mut self, value: u8) {
		let s = self.s;
		self.write8(0x100 + (s as u16), value);
		self.s = s.wrapping_sub(1);
	}

	fn push16(&mut self, value: u16) {
		self.push8((value >> 8) as u8);
		self.push8(value as u8);
	}

	fn pop8(&mut self) -> u8 {
		self.s = self.s.wrapping_add(1);
        let s = self.s;
		self.read8(0x100 + (s as u16))
	}

	fn pop16(&mut self) -> u16 {
		let lo = self.pop8() as u16;
		let hi = self.pop8() as u16;
		lo | (hi << 8)
	}

    pub fn read8(&self, address: u16) -> u8 {
        self.bus.read(address)
    }

    pub fn read16(&self, address: u16) -> u16 {
        (self.read8(address) as u16) | ((self.read8(address + 1) as u16) << 8)
    }

    pub fn reset(&mut self) {
        self.pc = 0xc000;//self.read16(0xfffc);
    }

	fn set_nz(&mut self, value: u8) {
		self.p.negative = (value & 0x80) != 0;
		self.p.zero = value == 0;
	}

    pub fn step(&mut self) {
        let opcode = self.read8(self.pc);
        self.pc += 1;

        let a = self.a;
        let pc = self.pc;
        let s = self.s;
        let x = self.x;
        let y = self.y;

        print!("0x{:04x}: ", pc - 1);

        match opcode {
            0x01 => { println!("ORA ind,x"); let value = self.load_ind_x(); self.ora(value); },
			0x05 => { println!("ORA zp"); let value = self.load_zp(); self.ora(value); },
			//0x06 => { println!("ASL zp"); let value = self.asl_zp(); self.store_zp(value); },
			0x08 => { println!("PHP"); let value = self.p.read() | 0x30; self.push8(value); },
			0x09 => { println!("ORA #"); let value = self.load_imm(); self.ora(value); },
			0x0a => { println!("ASL A"); self.a = self.asl(a); },
			0x0d => { println!("ORA abs"); let value = self.load_abs(); self.ora(value); },
			//0x0e => { println!("ASL abs"); let value = self.asl_abs(); self.store_abs(value); },
            0x10 => { println!("BPL"); let value = self.load_imm(); if !self.p.negative { self.branch(value) }; },
            0x11 => { println!("ORA ind,y"); let value = self.load_ind_y(); self.ora(value); },
			0x15 => { println!("ORA zp,x"); let value = self.load_zp_x(); self.ora(value); },
			//0x16 => { println!("ASL zp,x"); let value = self.asl_zp_x(); self.store_zp_x(value); },
            0x18 => { println!("CLC"); self.p.carry = false; },
            0x19 => { println!("ORA abs,y"); let value = self.load_abs_y(); self.ora(value); },
			0x1d => { println!("ORA abs,x"); let value = self.load_abs_x(); self.ora(value); },
			//0x1e => { println!("ASL abs,x"); let value = self.asl_abs_x(); self.store_abs_x(value); },
            0x20 => { println!("JSR"); self.push16(pc.wrapping_add(1)); self.pc = self.read16(pc); },
            0x21 => { println!("AND ind,x"); let value = self.load_ind_x(); self.and(value); },
            0x24 => { println!("BIT zp"); let value = self.load_zp(); self.bit(value); },
            0x25 => { println!("AND zp"); let value = self.load_zp(); self.and(value); },
			//0x26 => { println!("ROL zp"); let value = self.rol_zp(); self.store_zp(value); },
			0x28 => { println!("PLP"); let value = self.pop8(); self.p.write(value); }
			0x29 => { println!("AND #"); let value = self.load_imm(); self.and(value); },
			0x2a => { println!("ROL A"); self.a = self.rol(a); },
			0x2c => { println!("BIT abs"); let value = self.load_abs(); self.bit(value); },
			0x2d => { println!("AND abs"); let value = self.load_abs(); self.and(value); },
			//0x2e => { println!("ROL abs"); let value = self.rol_abs(); self.store_abs(value); },
			0x30 => { println!("BMI"); let value = self.load_imm(); if self.p.negative { self.branch(value) }; },
			0x31 => { println!("AND ind,y"); let value = self.load_ind_y(); self.and(value); },
			0x35 => { println!("AND zp,x"); let value = self.load_zp_x(); self.and(value); },
			//0x36 => { println!("ROL zp, x"); let value = self.rol_zp_x(); self.store_zp_x(value); },
            0x38 => { println!("SEC"); self.p.carry = true; },
            0x39 => { println!("AND abs,y"); let value = self.load_abs_y(); self.and(value); },
			0x3d => { println!("AND abs,x"); let value = self.load_abs_x(); self.and(value); },
			//0x3e => { println!("ROL abs,x"); let value = self.rol_abs_x(); self.store_abs_x(value); },
			0x40 => { println!("RTI"); let p = self.pop8(); self.p.write(p); self.pc = self.pop16(); },
			0x41 => { println!("EOR ind,x"); let value = self.load_ind_x(); self.eor(value); },
			0x45 => { println!("EOR zp"); let value = self.load_zp(); self.eor(value); },
			//0x46 => { println!("LSR zp"); let value = self.lsr_zp(); self.store_zp(value); },
			0x48 => { println!("PHA"); self.push8(a); },
			0x49 => { println!("EOR #"); let value = self.load_imm(); self.eor(value); },
			0x4a => { println!("LSR A"); self.a = self.lsr(a); },
            0x4c => { println!("JMP abs"); self.pc = self.read16(pc); },
            0x4d => { println!("EOR abs"); let value = self.load_abs(); self.eor(value); },
			//0x4e => { println!("LSR abs"); let value = self.lsr_abs(); self.store_abs(value); },
            0x50 => { println!("BVC"); let value = self.load_imm(); if !self.p.overflow { self.branch(value) }; },
            0x51 => { println!("EOR ind,y"); let value = self.load_ind_y(); self.eor(value); },
			0x55 => { println!("EOR zp,x"); let value = self.load_zp_x(); self.eor(value); },
			//0x56 => { println!("LSR zp,x"); let value = self.lsr_zp_x(); self.store_zp_x(value); },
			0x59 => { println!("EOR abs,y"); let value = self.load_abs_y(); self.eor(value); },
			0x5d => { println!("EOR abs,x"); let value = self.load_abs_x(); self.eor(value); },
			//0x5e => { println!("LSR abs,x"); let value = self.lsr_abs_x(); self.store_abs_x(value); },
            0x60 => { println!("RTS"); self.pc = self.pop16().wrapping_add(1); },
			0x61 => { println!("ADC ind,x"); let value = self.load_ind_x(); self.adc(value); },
			0x65 => { println!("ADC zp"); let value = self.load_zp(); self.adc(value); },
			//0x66 => { println!("ROR zp"); let value = self.ror_zp(); self.store_zp(value); },
			0x68 => { println!("PLA"); let value = self.pop8(); self.a = value; self.set_nz(value); }
			0x69 => { println!("ADC #"); let value = self.load_imm(); self.adc(value); },
			0x6a => { println!("ROR A"); self.a = self.ror(a); },
			0x6d => { println!("ADC abs"); let value = self.load_abs(); self.adc(value); },
			0x6c => { println!("JMP ind"); self.jmp_ind(); },
			//0x6e => { println!("ROR abs"); let value = self.ror_abs(); self.store_abs(value); },
			0x70 => { println!("BVS"); let value = self.load_imm(); if self.p.overflow { self.branch(value) }; },
			0x71 => { println!("ADC ind,y"); let value = self.load_ind_y(); self.adc(value); },
			0x75 => { println!("ADC zp,x"); let value = self.load_zp_x(); self.adc(value); },
			//0x76 => { println!("ROR zp, x"); let value = self.ror_zp_x(); self.store_zp_x(value); },
			0x78 => { println!("SEI"); self.p.interrupt = true; },
			0x79 => { println!("ADC abs,y"); let value = self.load_abs_y(); self.adc(value); },
			0x7d => { println!("ADC abs,x"); let value = self.load_abs_x(); self.adc(value); },
			//0x7e => { println!("ROR abs,x"); let value = self.ror_abs_x(); self.store_abs_x(value); },
			0x81 => { println!("STA ind,x"); self.store_ind_x(a); },
			0x84 => { println!("STY zp"); self.store_zp(y); },
			0x85 => { println!("STA zp"); self.store_zp(a); },
			0x86 => { println!("STX zp"); self.store_zp(x); },
			0x88 => { println!("DEY"); let y = y.wrapping_sub(1); self.y = y; self.set_nz(y); },
			0x8a => { println!("TXA"); self.a = x; self.set_nz(x); },
			0x8c => { println!("STY abs"); self.store_abs(y); },
			0x8d => { println!("STA abs"); self.store_abs(a); },
			0x8e => { println!("STX abs"); self.store_abs(x); },
			0x90 => { println!("BCC"); let value = self.load_imm(); if !self.p.carry { self.branch(value) }; },
			0x91 => { println!("STA ind,y"); self.store_ind_y(a); },
			0x94 => { println!("STY zp,x"); self.store_zp_x(y); },
			0x95 => { println!("STA zp,x"); self.store_zp_x(a);  },
			0x96 => { println!("STX zp,y"); self.store_zp_y(x); },
			0x98 => { println!("TYA"); self.a = y; self.set_nz(y); },
			0x99 => { println!("STA abs,y"); self.store_abs_y(a); },
			0x9a => { println!("TXS"); self.s = x; },
			0x9d => { println!("STA abs,x"); self.store_abs_x(a); },
			0xa0 => { println!("LDY #"); let value = self.load_imm(); self.ldy(value); },
			0xa1 => { println!("LDA ind,x"); let value = self.load_ind_x(); self.lda(value); },
			0xa2 => { println!("LDX #"); let value = self.load_imm(); self.ldx(value); },
			0xa4 => { println!("LDY zp"); let value = self.load_zp(); self.ldy(value); },
			0xa5 => { println!("LDA zp"); let value = self.load_zp(); self.lda(value); },
			0xa6 => { println!("LDX zp"); let value = self.load_zp(); self.ldx(value); },
			0xa8 => { println!("TAY"); self.y = a; self.set_nz(a); },
			0xa9 => { println!("LDA #"); let value = self.load_imm(); self.lda(value); },
			0xaa => { println!("TAX"); self.x = a; self.set_nz(a); },
			0xac => { println!("LDY abs"); let value = self.load_abs(); self.ldy(value); },
			0xad => { println!("LDA abs"); let value = self.load_abs(); self.lda(value); },
			0xae => { println!("LDX abs"); let value = self.load_abs(); self.ldx(value); },
			0xb0 => { println!("BCS"); let value = self.load_imm(); if self.p.carry { self.branch(value) }; },
			0xb1 => { println!("LDA ind,y"); let value = self.load_ind_y(); self.lda(value); },
			0xb4 => { println!("LDY zp,x"); let value = self.load_zp_x(); self.ldy(value); },
			0xb5 => { println!("LDA zp,x"); let value = self.load_zp_x(); self.lda(value); },
			0xb6 => { println!("LDX zp,y"); let value = self.load_zp_y(); self.ldx(value); },
			0xb8 => { println!("CLV"); self.p.overflow = false; },
			0xb9 => { println!("LDA abs,y"); let value = self.load_abs_y(); self.lda(value); },
			0xba => { println!("TSX"); self.x = s; self.set_nz(s); },
			0xbc => { println!("LDY abs,x"); let value = self.load_abs_x(); self.ldy(value); },
			0xbd => { println!("LDA abs,x"); let value = self.load_abs_x(); self.lda(value); },
			0xbe => { println!("LDX abs,y"); let value = self.load_abs_y(); self.ldx(value); },
			0xc0 => { println!("CPY #"); let value = self.load_imm(); self.cpy(value); },
			0xc1 => { println!("CMP ind,x"); let value = self.load_ind_x(); self.cmp(value); },
			0xc4 => { println!("CPY zp"); let value = self.load_zp(); self.cpy(value); },
			0xc5 => { println!("CMP zp"); let value = self.load_zp(); self.cmp(value); },
			//0xc6 => { println!("DEC zp"); self.dec_zp(); },
			0xc8 => { println!("INY"); let y = y.wrapping_add(1); self.y = y; self.set_nz(y); },
			0xc9 => { println!("CMP imm"); let value = self.load_imm(); self.cmp(value); },
			0xca => { println!("DEX"); let x = x.wrapping_sub(1); self.x = x; self.set_nz(x); },
			0xcc => { println!("CPY abs"); let value = self.load_abs(); self.cpy(value); },
			0xcd => { println!("CMP abs"); let value = self.load_abs(); self.cmp(value); },
			//0xce => { println!("DEC abs"); self.dec_abs(); },
			0xd0 => { println!("BNE"); let value = self.load_imm(); if !self.p.zero { self.branch(value) }; },
			0xd1 => { println!("CMP ind,y"); let value = self.load_ind_y(); self.cmp(value); },
			0xd5 => { println!("CMP zp,x"); let value = self.load_zp_x(); self.cmp(value); },
			//0xd6 => { println!("DEC zp,x"); self.dec_zp_x(); },
			0xd8 => { println!("CLD"); self.p.decimal = false; },
			0xd9 => { println!("CMP abs,y"); let value = self.load_abs_y(); self.cmp(value); },
			0xdd => { println!("CMP abs,x"); let value = self.load_abs_x(); self.cmp(value); },
			//0xde => { println!("DEC abs,x"); self.dec_abs_x(); },
			0xe0 => { println!("CPX #"); let value = self.load_imm(); self.cpx(value); },
			0xe1 => { println!("SBC ind,x"); let value = self.load_ind_x(); self.adc(!value); },
			0xe4 => { println!("CPX zp"); let value = self.load_zp(); self.cpx(value); },
			0xe5 => { println!("SBC zp"); let value = self.load_zp(); self.adc(!value); },
			//0xe6 => { println!("INC zp"); self.inc_zp(); },
			0xe8 => { println!("INX"); let x = x.wrapping_add(1); self.x = x; self.set_nz(x); },
			0xe9 => { println!("SBC #"); let value = self.load_imm(); self.adc(!value); },
			0xea => { println!("NOP"); },
			0xec => { println!("CPX abs"); let value = self.load_abs(); self.cpx(value); },
			0xed => { println!("SBC abs"); let value = self.load_abs(); self.adc(!value); },
			//0xee => { println!("INC abs"); self.inc_abs(); },
			0xf0 => { println!("BEQ"); let value = self.load_imm(); if self.p.zero { self.branch(value) }; },
			0xf1 => { println!("SBC ind,y"); let value = self.load_ind_y(); self.adc(!value); },
			0xf5 => { println!("SBC zp,x"); let value = self.load_zp_x(); self.adc(!value); },
			//0xf6 => { println!("INC zp,x"); self.inc_zp_x(); },
			0xf8 => { println!("SED"); self.p.decimal = true; },
			0xf9 => { println!("SBC abs,y"); let value = self.load_abs_y(); self.adc(!value); },
			0xfd => { println!("SBC abs,x"); let value = self.load_abs_x(); self.adc(!value); },
			//0xfe => { println!("INC abs,x"); self.inc_abs_x(); },
            _ => panic!("unimplemented opcode 0x{:02x}", opcode)
        }
    }

    pub fn write8(&mut self, address: u16, value: u8) {
        self.bus.write(address, value)
    }
}