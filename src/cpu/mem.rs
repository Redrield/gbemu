use crate::cpu::BOOTROM;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum MemoryRegister {
    P1,
    SB,
    SC,
    DIV,
    TIMA,
    TMA,
    TAC,
    IF,
    NR10,
    NR11,
    NR12,
    NR13,
    NR14,
    NR21,
    NR22,
    NR23,
    NR24,
    NR30,
    NR31,
    NR32,
    NR33,
    NR34,
    NR41,
    NR42,
    NR43,
    NR44,
    NR50,
    NR51,
    NR52,
    WavePatternRAM(u8),
    LCDC,
    STAT,
    SCY,
    SCX,
    LY,
    LYC,
    DMA,
    BGP,
    OBP0,
    OBP1,
    WY,
    WX,
    IE,
}

impl MemoryRegister {
    pub fn to_addr(self) -> u16 {
        match self {
            MemoryRegister::P1 => 0xFF00,
            MemoryRegister::SB => 0xFF01,
            MemoryRegister::SC => 0xFF02,
            MemoryRegister::DIV => 0xFF04,
            MemoryRegister::TIMA => 0xFF05,
            MemoryRegister::TMA => 0xFF06,
            MemoryRegister::TAC => 0xFF07,
            MemoryRegister::IF => 0xFF0F,
            MemoryRegister::NR10 => 0xFF10,
            MemoryRegister::NR11 => 0xFF11,
            MemoryRegister::NR12 => 0xFF12,
            MemoryRegister::NR13 => 0xFF13,
            MemoryRegister::NR14 => 0xFF14,
            MemoryRegister::NR21 => 0xFF16,
            MemoryRegister::NR22 => 0xFF17,
            MemoryRegister::NR23 => 0xFF18,
            MemoryRegister::NR24 => 0xFF19,
            MemoryRegister::NR30 => 0xFF1A,
            MemoryRegister::NR31 => 0xFF1B,
            MemoryRegister::NR32 => 0xFF1C,
            MemoryRegister::NR33 => 0xFF1D,
            MemoryRegister::NR34 => 0xFF1E,
            MemoryRegister::NR41 => 0xFF20,
            MemoryRegister::NR42 => 0xFF21,
            MemoryRegister::NR43 => 0xFF22,
            MemoryRegister::NR44 => 0xFF23,
            MemoryRegister::NR50 => 0xFF24,
            MemoryRegister::NR51 => 0xFF25,
            MemoryRegister::NR52 => 0xFF26,
            MemoryRegister::WavePatternRAM(offset) => 0xFF30 + (offset as u16),
            MemoryRegister::LCDC => 0xFF40,
            MemoryRegister::STAT => 0xFF41,
            MemoryRegister::SCY => 0xFF42,
            MemoryRegister::SCX => 0xFF43,
            MemoryRegister::LY => 0xFF44,
            MemoryRegister::LYC => 0xFF45,
            MemoryRegister::DMA => 0xFF46,
            MemoryRegister::BGP => 0xFF47,
            MemoryRegister::OBP0 => 0xFF48,
            MemoryRegister::OBP1 => 0xFF49,
            MemoryRegister::WY => 0xFF4A,
            MemoryRegister::WX => 0xFF4B,
            MemoryRegister::IE => 0xFFFF,
        }
    }
}

pub struct Memory {
    pub buffer: [u8; 0xFFFF + 1],
    pub bootrom_paged: bool,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            buffer: [0; 0xFFFF + 1],
            bootrom_paged: true,
        }
    }

    pub fn get_register(&self, reg: MemoryRegister) -> u8 {
        self.buffer[reg.to_addr() as usize]
    }

    pub fn set_register(&mut self, reg: MemoryRegister, value: u8) {
        self.buffer[reg.to_addr() as usize] = value
    }

    pub fn get_addr(&self, addr: u16) -> u8 {
        if self.bootrom_paged && addr < 0xff {
            BOOTROM[addr as usize]
        } else {
            self.buffer[addr as usize]
        }
    }

    pub fn get_u16_at(&self, idx: u16) -> u16 {
        let bytes = [self.get_addr(idx), self.get_addr(idx + 1)];
        u16::from_le_bytes(bytes)
    }

    pub fn set_addr(&mut self, addr: u16, value: u8) {
        self.buffer[addr as usize] = value;
    }
}
