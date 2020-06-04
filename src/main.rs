mod cpu;
mod peripherals;
mod util;

use crate::cpu::{BOOTROM, CPU, CYCLES_PER_FRAME, CLOCK_SPEED};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::{Instant, Duration};
use std::sync::mpsc;
use std::thread;

pub struct CpuDrv {
    cpu: cpu::CPU,
    last_time: Instant,
    step_mode: bool,
    breakpoints: Vec<u16>,
    bp_channel: mpsc::Receiver<BreakpointMessage>,
}

pub enum BreakpointMessage {
    Add(u16),
    Remove(u16),
}

impl CpuDrv {
    pub fn new(rom: Vec<u8>, bp_channel: mpsc::Receiver<BreakpointMessage>) -> CpuDrv {
        let mut cpu = CPU::new();
        cpu.load_code(rom);

        CpuDrv {
            cpu,
            last_time: Instant::now(),
            step_mode: true,
            breakpoints: Vec::new(),
            bp_channel
        }
    }

    pub fn drive(mut self) {
        let mut event = self.cpu.sdl.event_pump().unwrap();

        'main: loop {
            if !self.step_mode {
                self.run();
            }

            if let Ok(msg) = self.bp_channel.try_recv() {
                match msg {
                    BreakpointMessage::Add(pc) => if !self.breakpoints.contains(&pc) {
                        println!("Added breakpoint at PC={:#x}", pc);
                        self.breakpoints.push(pc)
                    },
                    BreakpointMessage::Remove(pc) => if let Some((i, _)) = self.breakpoints.iter().enumerate().find(|(_, bp)| pc == **bp) {
                        println!("Removed breakpoint at PC={:#x}", pc);
                        self.breakpoints.remove(i);
                    }
                }
            }


            for ev in event.poll_iter() {
                match ev {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'main,
                    Event::KeyDown {
                        keycode: Some(Keycode::E),
                        ..
                    } => {
                        self.step_mode = !self.step_mode;
                        println!("Single step mode: {}", self.step_mode)
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::S),
                        ..
                    } => {
                        if self.step_mode {
                            self.cpu.tick();
                        }
                    }
                    _ => {}
                }
            }
            self.last_time = Instant::now();
        }
    }

    /// Runs one full frame, plus additional cycles at the start of the period to clear the blanking interval
    pub fn run(&mut self) {
        // Tick through the blanking interval if necessary
        while self.cpu.video.vblank_acc != 0 || self.cpu.video.hblank_acc != 0 {
            self.cpu.tick();
        }

        let future = Instant::now() + Duration::from_secs(CYCLES_PER_FRAME / CLOCK_SPEED);

        // Tick through the frame
        while self.cpu.video.vblank_acc == 0 {
            self.cpu.tick();
        }

        // If need be delay to match the expected frequency
        let now = Instant::now();
        if now < future {
            thread::sleep(future - now);
        }
        // let mut cycles =
        //     (self.last_time.elapsed().as_secs_f64() * cpu::CLOCK_SPEED as f64).floor() as i32;
        //
        // while cycles > 0 {
        //     if self.breakpoints.contains(&self.cpu.reg.pc) && !self.step_mode {
        //         self.step_mode = true;
        //         println!("BREAK PC={:#x}", self.cpu.reg.pc);
        //         return;
        //     }
        //     cycles -= self.cpu.tick() as i32;
        // }
    }
}

fn main() {
    use std::fs;
    let rom = fs::read("cpu_instrs.gb").unwrap();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let stdin = std::io::stdin();
        loop {
            let mut line = String::new();
            stdin.read_line(&mut line).unwrap();

            if line.starts_with("break") {
                let parts = line.split(" ").collect::<Vec<&str>>();

                if let Ok(pc) = u16::from_str_radix(parts[2].trim(), 16) {
                    match parts[1].to_lowercase().as_str() {
                        "add" => {
                            tx.send(BreakpointMessage::Add(pc)).unwrap();
                        }
                        "rm" => {
                            tx.send(BreakpointMessage::Remove(pc)).unwrap();
                        }
                        _ => {}
                    }
                }
            }
        }
    });
    let mut drv = CpuDrv::new(rom, rx);

    println!("Reminder that CPU is started in single step mode.");
    drv.drive();
}
