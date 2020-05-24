use crate::cpu::mem::{MemoryRegister, Memory};
use crate::cpu::*;
use crate::peripherals::audio::SelectedSquareWaveCycle::DutyCycle12_5;
use sdl2::audio::{AudioQueue, AudioStatus, AudioSpecDesired};
use sdl2::Sdl;

/// The 12.5% duty cycle square wave. true is a signal, false is no signal
pub const DUTY_CYCLE_00: [bool; 8] = [false, false, false, false, false, false, false, true];
/// The 25% duty cycle square wave. true is a signal, false is no signal
pub const DUTY_CYCLE_01: [bool; 8] = [true, false, false, false, false, false, false, true];
/// The 50% duty cycle square wave. true is a signal, false is no signal
pub const DUTY_CYCLE_10: [bool; 8] = [true, true, false, false, false, false, true, true];
/// The 75% duty cycle square wave. true is a signal, false is no signal
pub const DUTY_CYCLE_11: [bool; 8] = [false, true, true, true, true, true, true, false];

pub enum SelectedSquareWaveCycle {
    DutyCycle12_5,
    DutyCycle25,
    DutyCycle50,
    DutyCycle75,
}

pub struct AudioDrv {
    ch2_seq_ptr: usize,
    ch2_sel_cycle: SelectedSquareWaveCycle,
    sdl_queue: AudioQueue<u8>,
    init_timer: u32,
    timer: u32,
}

impl AudioDrv {
    pub fn new(sdl: &Sdl) -> AudioDrv {
        let subsys = sdl.audio().unwrap();
        let desspec = AudioSpecDesired {
            freq: Some(44_100),
            channels: Some(2),
            samples: Some(4096)
        };

        let queue = subsys.open_queue(None, &desspec).unwrap();
        AudioDrv {
            ch2_seq_ptr: 0,
            ch2_sel_cycle: SelectedSquareWaveCycle::DutyCycle50,
            sdl_queue: queue,
            init_timer: 0,
            timer: 0,
        }
    }

    pub fn tick(&mut self, mem: &Memory) {
        let freq_low = mem.get_register(MemoryRegister::NR23);
        let nr21 = mem.get_register(MemoryRegister::NR21);
        let nr24 = mem.get_register(MemoryRegister::NR24);
        let nr22 = mem.get_register(MemoryRegister::NR22);
        let freq_hi = nr24 & 0x7;

        let x = ((freq_hi as u16) << 8) | freq_low as u16;
        let timer = 131072 / (2048 - x as u32);
        let volume = (nr22 & 0xf0) >> 4;


        if volume == 0 && self.sdl_queue.status() == AudioStatus::Playing {
            self.sdl_queue.pause();
            self.sdl_queue.clear();
        }

        if volume != 0 {
            if timer != self.init_timer {
                self.init_timer = timer;
                self.timer = timer;
                self.ch2_seq_ptr = 0;

                self.ch2_sel_cycle = match (nr21 & 0xc0) >> 6 {
                    0b00 => SelectedSquareWaveCycle::DutyCycle12_5,
                    0b01 => SelectedSquareWaveCycle::DutyCycle25,
                    0b10 => SelectedSquareWaveCycle::DutyCycle50,
                    0b11 => SelectedSquareWaveCycle::DutyCycle75,
                    _ => unreachable!()
                };
                self.sdl_queue.resume();
            }

            self.timer -= 1;

            let bit = match self.ch2_sel_cycle {
                SelectedSquareWaveCycle::DutyCycle12_5 => {
                    DUTY_CYCLE_00[self.ch2_seq_ptr]
                }
                SelectedSquareWaveCycle::DutyCycle25 => {
                    DUTY_CYCLE_01[self.ch2_seq_ptr]
                }
                SelectedSquareWaveCycle::DutyCycle50 => {
                    DUTY_CYCLE_10[self.ch2_seq_ptr]
                }
                SelectedSquareWaveCycle::DutyCycle75 => {
                    DUTY_CYCLE_11[self.ch2_seq_ptr]
                }
            };

            if bit {
                self.sdl_queue.queue(&[volume]);
            } else {
                self.sdl_queue.queue(&[0]);
            }
        }

    }
}
