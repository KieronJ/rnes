use sdl2::keyboard::*;

pub const CONTROLLER_ADDRESS: u16 = 0x4016;

pub struct Controller {
    strobe: bool,
    state: u8,
    state_locked: u8,
}

impl Controller {
    pub fn new() -> Controller {
        Controller {
            strobe: false,
            state: 0,
            state_locked: 0,
        }
    }

    pub fn in_range(&self, address: u16) -> bool {
        return address == CONTROLLER_ADDRESS
    }

    pub fn io_read(&mut self) -> u8 {
        if self.strobe {
            return 0x40 | (self.state & 0x1);
        }

        let key = 0x40 | (self.state_locked & 0x1);
        self.state_locked = 0x80 | (self.state_locked >> 1);

        return key;
    }

    pub fn io_write(&mut self, value: u8) {
        if self.strobe && (value & 0x1) == 0 {
            self.state_locked = self.state;
        }

        self.strobe = (value & 0x1) != 0;
    }

    pub fn set_button(&mut self, keycode: Keycode, state: bool) {
        match keycode {
            Keycode::A      => { self.state &= !0x1; self.state |= (state as u8) << 0 },
            Keycode::S      => { self.state &= !0x2; self.state |= (state as u8) << 1 },
            Keycode::Z      => { self.state &= !0x4; self.state |= (state as u8) << 2 },
            Keycode::X      => { self.state &= !0x8; self.state |= (state as u8) << 3 },
            Keycode::Up     => { self.state &= !0x10; self.state |= (state as u8) << 4 },
            Keycode::Down   => { self.state &= !0x20; self.state |= (state as u8) << 5 },
            Keycode::Left   => { self.state &= !0x40; self.state |= (state as u8) << 6 },
            Keycode::Right  => { self.state &= !0x80; self.state |= (state as u8) << 7 },
            _ => (),
        }
    }
}