mod addressing;
mod functions;
mod instructions;
mod status;

use nes::bus::Bus;
use nes::ricoh2a03::status::Status;

#[derive(PartialEq)]
pub enum InterruptType {
	NMI,
	RESET,
	IRQ,
	BRK,
}

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
        Ricoh2A03 {
            pc: 0xc000,
            a: 0,
            x: 0,
            y: 0,
            s: 0xfd,
            p: Status::new(),
            bus: bus,
        }
    }

    pub fn reset(&mut self) {
        self.interrupt(InterruptType::RESET);
    }
}