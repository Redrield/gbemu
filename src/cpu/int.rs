use super::*;

const VBLANK: u16 = 0x0040;
const LCDSTAT: u16 = 0x0048;
const TIMER: u16 = 0x0050;
const SERIAL: u16 = 0x0058;
const JOYPAD: u16 = 0x0060;

#[repr(u8)]
pub enum Interrupt {
    Vblank,
    LcdStat,
    Timer,
    Serial,
    Joypad,
}

impl CPU {
    /// Updates the desired bit in IF to signal a pending interrupt.
    /// The running code will jump to the corresponding vector on the next call to handle_interrupts
    /// assuming that interrupts are enabled and the interrupt is not masked out.
    pub fn dispatch_interrupt(&mut self, int: Interrupt) {
        let mut iflags = self.mem.get_register(MemoryRegister::IF);
        iflags |= (1 << int as u8);
        self.mem.set_register(MemoryRegister::IF, iflags);
    }

    /// Attempts to service any pending interrupts
    /// Returns true if an interrupt was serviced. The CPU should consume 20 clock cycles to match the ISR of the Z80 if this returns true
    pub fn handle_interrupts(&mut self) -> bool {
        let iflags = self.mem.get_register(MemoryRegister::IF);
        let ieflags = self.mem.get_register(MemoryRegister::IE);
        let ie = self.reg.ie;

        for i in 0..5 {
            // Is the interrupt requested in IF
            let ireq = iflags & (1 << i) != 0;
            // Is the interrupt enabled (Flag toggled in IE register, interrupts are enabled globally)
            let int_enabled = ie && (ieflags & (1 << i) != 0);

            if ireq && int_enabled {
                self.reg.ie = false; // CLI
                self.mem
                    .set_register(MemoryRegister::IF, iflags & !(1 << i));
                let pc = self.reg.pc.to_le_bytes();
                let mut sp = self.reg.sp;
                sp -= 2;
                // PUSH PC
                self.mem.set_addr(sp, pc[0]);
                self.mem.set_addr(sp, pc[1]);
                self.reg.sp = sp;

                // JMP to vector
                match i {
                    0 => {
                        self.reg.pc = VBLANK;
                    }
                    1 => {
                        self.reg.pc = LCDSTAT;
                    }
                    2 => {
                        self.reg.pc = TIMER;
                    }
                    3 => {
                        self.reg.pc = SERIAL;
                    }
                    4 => {
                        self.reg.pc = JOYPAD;
                    }
                    _ => unreachable!(),
                }
                // Correctly handle interrupt priority, when one interrupt has matched it should execute regardless of any other set bits
                return true;
            }
        }
        false
    }
}
