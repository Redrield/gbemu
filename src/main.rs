mod cpu;
mod peripherals;
mod util;

use crate::cpu::{CPU, BOOTROM};
use std::time::Instant;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub struct CpuDrv {
    cpu: cpu::CPU,
    last_time: Instant,
}

impl CpuDrv {
    pub fn new(rom: Vec<u8>) -> CpuDrv {
        let mut cpu = CPU::new();
        cpu.load_code(rom);
        cpu.state.bootrom_paged = false;
        cpu.reg.pc = 0x100;

        CpuDrv {
            cpu,
            last_time: Instant::now(),
        }
    }

    pub fn drive(mut self) {
        let mut event = self.cpu.sdl.event_pump().unwrap();

        'main: loop {
            self.tick();
            for ev in event.poll_iter() {
                match ev {
                    Event::Quit { .. } |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'main,
                    _ => {}
                }
            }
        }
    }

    pub fn tick(&mut self) {
        let mut cycles =
            (self.last_time.elapsed().as_secs_f64() * cpu::CLOCK_SPEED as f64).floor() as i32;

        while cycles > 0 {
            cycles -= self.cpu.tick() as i32;
        }

        self.last_time = Instant::now();
    }
}

fn main() {
    use std::fs;
    let rom = fs::read("reee.o").unwrap();
    let mut drv = CpuDrv::new(rom);

    drv.drive();
}
