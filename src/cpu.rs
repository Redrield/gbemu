mod decode;
mod exec;
pub mod int;
pub mod isa;
pub mod mem;
use int::*;
use isa::*;
use mem::*;
use crate::peripherals::audio::AudioDrv;
use sdl2::Sdl;
use crate::peripherals::video::VideoDrv;

/// The number of CPU cycles that can be performed between screen refreshes
const CYCLES_PER_FRAME: u32 = 280_896;
pub const CLOCK_SPEED: u64 = 4_190_000;
pub static BOOTROM: &'static [u8; 256] = include_bytes!("bootrom.bin");

#[derive(Default, Debug)]
pub struct Registers {
    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
    pub sp: u16,
    pub pc: u16,
    pub ie: bool,
}

pub enum Flag {
    Zero,
    AddSub,
    HalfCarry,
    Carry,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            sp: 0xFFFE,
            ..Default::default()
        }
    }

    fn read16(&self, name: RegisterName) -> Option<u16> {
        match name {
            RegisterName::AF => Some(self.af),
            RegisterName::BC => Some(self.bc),
            RegisterName::DE => Some(self.de),
            RegisterName::HL => Some(self.hl),
            RegisterName::SP => Some(self.sp),
            _ => None,
        }
    }

    fn read8(&self, name: RegisterName) -> Option<u8> {
        match name {
            RegisterName::A => Some((self.af >> 8) as u8),
            RegisterName::F => Some((self.af & 0xff) as u8),
            RegisterName::B => Some((self.bc >> 8) as u8),
            RegisterName::C => Some((self.bc & 0xff) as u8),
            RegisterName::D => Some((self.de >> 8) as u8),
            RegisterName::E => Some((self.de & 0xff) as u8),
            RegisterName::H => Some((self.hl >> 8) as u8),
            RegisterName::L => Some((self.hl & 0xff) as u8),
            _ => None,
        }
    }

    pub fn write16(&mut self, name: RegisterName, value: u16) {
        match name {
            RegisterName::AF => self.af = value,
            RegisterName::BC => self.bc = value,
            RegisterName::DE => self.de = value,
            RegisterName::HL => self.hl = value,
            RegisterName::SP => self.sp = value,
            _ => {}
        }
    }

    pub fn write8(&mut self, name: RegisterName, value: u8) {
        match name {
            RegisterName::A => self.af = ((value as u16) << 8) | (self.af & 0xff),
            RegisterName::F => self.af = (self.af & 0xff00) | value as u16,
            RegisterName::B => self.bc = ((value as u16) << 8) | (self.af & 0xff),
            RegisterName::C => self.bc = (self.bc & 0xff00) | value as u16,
            RegisterName::D => self.de = ((value as u16) << 8) | (self.de & 0xff),
            RegisterName::E => self.de = (self.de & 0xff00) | value as u16,
            RegisterName::H => self.hl = ((value as u16) << 8) | (self.hl & 0xff),
            RegisterName::L => self.hl = (self.hl & 0xff00) | value as u16,
            _ => {}
        }
    }

    fn set_flags(&mut self, z: bool, n: bool, hc: bool, c: bool) {
        let mut flags = 0x00;

        if z {
            flags |= 1 << 7;
        }
        if n {
            flags |= 1 << 6;
        }
        if hc {
            flags |= 1 << 5;
        }
        if c {
            flags |= 1 << 4;
        }

        self.write8(RegisterName::F, flags);
    }

    fn set_flag(&mut self, flag: Flag, set: bool) {
        let mut flags = self.read8(RegisterName::F).unwrap();

        match flag {
            Flag::Zero => {
                if set {
                    flags |= 1 << 7;
                } else {
                    flags &= !(1 << 7);
                }
            }
            Flag::AddSub => {
                if set {
                    flags |= 1 << 6;
                } else {
                    flags &= !(1 << 6);
                }
            }
            Flag::HalfCarry => {
                if set {
                    flags |= 1 << 5;
                } else {
                    flags &= !(1 << 5);
                }
            }
            Flag::Carry => {
                if set {
                    flags |= 1 << 4;
                } else {
                    flags &= !(1 << 4);
                }
            }
        }

        self.write8(RegisterName::F, flags);
    }

    fn flag(&self, flag: Flag) -> bool {
        let flags = self.read8(RegisterName::F).unwrap();
        match flag {
            Flag::Zero => flags & (1 << 7) != 0,
            Flag::AddSub => flags & (1 << 6) != 0,
            Flag::HalfCarry => flags & (1 << 5) != 0,
            Flag::Carry => flags & (1 << 4) != 0,
        }
    }
}

#[derive(Default)]
pub struct CPUState {
    pub ei_pending: bool,
    pub di_pending: bool,
    pub halted: bool,
    pub stopped: bool,
    pending_cycles: u8,
    pub bootrom_paged: bool,
}

pub struct CPU {
    pub sdl: Sdl,
    pub reg: Registers,
    pub mem: Memory,
    pub state: CPUState,
    pub audio: AudioDrv,
    pub video: VideoDrv,
    dbgwait: bool,
}

impl CPU {
    pub fn new() -> CPU {
        let sdl = sdl2::init().unwrap();
        let audio = AudioDrv::new(&sdl);
        let video = VideoDrv::new(&sdl);
        CPU {
            sdl,
            reg: Registers::new(),
            mem: Memory::new(),
            state: CPUState::default(),
            audio,
            video,
            dbgwait: false
        }
    }

    pub fn load_code(&mut self, code: Vec<u8>) {
        //TODO: Multiple ROM types
        for (i, mut b) in self.mem.buffer[0x0000..0x4000].iter_mut().enumerate() {
            if i >= code.len() {
                break;
            }
            *b = code[i];
        }
        self.reg.pc = 0x0;
        self.state.bootrom_paged = true;
    }

    pub fn read_u16(&self) -> u16 {
        let bytes = if self.state.bootrom_paged {
            [
                BOOTROM[self.reg.pc as usize],
                BOOTROM[self.reg.pc as usize + 1]
            ]
        } else {
            [
                self.mem.get_addr(self.reg.pc),
                self.mem.get_addr(self.reg.pc + 1),
            ]
        };
        u16::from_le_bytes(bytes)
    }

    pub fn tick(&mut self) -> u32 {
        if self.state.halted {
            if self.handle_interrupts() {
                self.state.halted = false;
                return 20;
            }
        }

        if self.mem.get_addr(0xFF50) == 1 {
            println!("Unpaging bootrom");
            self.state.bootrom_paged = false;
        }

        let ins = self.decode();
        let mut cycles = self.execute(ins) as u32;
        // let sc = self.mem.get_register(MemoryRegister::SC);
        // if sc == 0x81 && !self.dbgwait {
        //     print!("{}", char::from(self.mem.get_register(MemoryRegister::SB)));
        //     self.dbgwait = true;
        // } else if sc == 0x00 {
        //     self.dbgwait = false;
        // }

        if self.handle_interrupts() {
            cycles += 20;
        }

        // self.audio.tick(&self.mem);
        if self.video.tick(&mut self.mem) {
            self.dispatch_interrupt(Interrupt::Vblank);
        }

        if self.state.ei_pending && !self.reg.ie {
            self.reg.ie = true;
            self.state.ei_pending = false;
        }

        if self.state.di_pending && self.reg.ie {
            self.state.di_pending = false;
            self.reg.ie = false;
        }

        cycles
    }
}

#[cfg(test)]
mod tests {
    use super::isa::*;
    use super::*;

    #[test]
    fn test_ld() {
        let code = vec![0x06, 0x05];

        let mut cpu = CPU::new();
        cpu.load_code(code);
        let ins = cpu.decode();
        assert_eq!(ins, Instruction::Ld8Imm(RegisterName::B, 5));

        let code = vec![0x11, 0x01, 0x00];
        cpu.load_code(code);
        let ins = cpu.decode();
        assert_eq!(ins, Instruction::Ld16Imm(RegisterName::DE, 1));
    }
}
