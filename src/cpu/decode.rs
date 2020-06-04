use super::isa::*;
use super::{isa, CPU};
use crate::cpu::BOOTROM;

impl CPU {
    pub fn decode(&mut self) -> Instruction {
        let ins0 = self.mem.get_addr(self.reg.pc);
        self.reg.pc += 1;
        let x = (ins0 & 0xc0) >> 6;
        let y = (ins0 & 0x38) >> 3;
        let z = ins0 & 0x7;

        let p = y >> 1;
        let q = y % 2;

        match x {
            0 => match z {
                0 => match y {
                    0 => Instruction::Nop,
                    1 => {
                        let word = self.mem.get_u16_at(self.reg.pc);
                        self.reg.pc += 2;
                        Instruction::StoSP(word)
                    }
                    2 => {
                        self.reg.pc += 1;
                        Instruction::Stop
                    }
                    3 => {
                        let offset = self.mem.get_addr(self.reg.pc) as i8;
                        self.reg.pc += 1;

                        Instruction::Jr(offset)
                    }
                    _ => {
                        let rel = self.mem.get_addr(self.reg.pc) as i8;
                        self.reg.pc += 1;
                        Instruction::JrCond(isa::TABLE_CC[y as usize - 4], rel)
                    }
                },
                1 => match q {
                    0 => {
                        let word = self.mem.get_u16_at(self.reg.pc);
                        self.reg.pc += 2;
                        Instruction::Ld16Imm(isa::TABLE_RP[p as usize], word)
                    }
                    1 => Instruction::Add16(isa::TABLE_RP[p as usize]),
                    _ => Instruction::IllInsn,
                },
                2 => match q {
                    0 => match p {
                        0 => Instruction::Ld8Reg(RegisterName::BCRef, RegisterName::A),
                        1 => Instruction::Ld8Reg(RegisterName::DERef, RegisterName::A),
                        2 => Instruction::LdiHLA,
                        3 => Instruction::LddHLA,
                        _ => Instruction::IllInsn,
                    },
                    1 => match p {
                        0 => Instruction::Ld8Reg(RegisterName::A, RegisterName::BCRef),
                        1 => Instruction::Ld8Reg(RegisterName::A, RegisterName::DERef),
                        2 => Instruction::LdiAHL,
                        3 => Instruction::LddAHL,
                        _ => Instruction::IllInsn,
                    },
                    _ => Instruction::IllInsn,
                },
                3 => {
                    if q == 0 {
                        Instruction::Inc16(isa::TABLE_RP[p as usize])
                    } else {
                        Instruction::Dec16(isa::TABLE_RP[p as usize])
                    }
                }
                4 => Instruction::Inc(isa::TABLE_R[y as usize]),
                5 => Instruction::Dec(isa::TABLE_R[y as usize]),
                6 => {
                    let oper = self.mem.get_addr(self.reg.pc);
                    self.reg.pc += 1;
                    Instruction::Ld8Imm(isa::TABLE_R[y as usize], oper)
                }
                7 => match y {
                    0 => Instruction::Rlca,
                    1 => Instruction::Rrca,
                    2 => Instruction::Rla,
                    3 => Instruction::Rra,
                    4 => Instruction::Daa,
                    5 => Instruction::Cpl,
                    6 => Instruction::Scf,
                    7 => Instruction::Ccf,
                    _ => Instruction::IllInsn,
                },
                _ => Instruction::IllInsn,
            },
            1 => {
                if z == 6 && y == 6 {
                    Instruction::Halt
                } else {
                    Instruction::Ld8Reg(isa::TABLE_R[y as usize], isa::TABLE_R[z as usize])
                }
            }
            2 => isa::alu_reg(y as usize, isa::TABLE_R[z as usize]),
            3 => match z {
                0 => match y {
                    4 => {
                        let arg = self.mem.get_addr(self.reg.pc);
                        self.reg.pc += 1;
                        Instruction::LdhN(arg)
                    }
                    5 => {
                        let arg = self.mem.get_addr(self.reg.pc) as i8;
                        self.reg.pc += 1;
                        Instruction::AddSP(arg)
                    }
                    6 => {
                        let arg = self.mem.get_addr(self.reg.pc);
                        self.reg.pc += 1;
                        Instruction::LdhA(arg)
                    }
                    7 => {
                        let arg = self.mem.get_addr(self.reg.pc) as i8;
                        self.reg.pc += 1;
                        Instruction::LdHLSPn(arg)
                    }
                    _ => Instruction::RetCond(isa::TABLE_CC[y as usize]),
                },
                1 => match q {
                    0 => Instruction::Pop(isa::TABLE_RP2[p as usize]),
                    1 => match p {
                        0 => Instruction::Ret,
                        1 => Instruction::Reti,
                        2 => Instruction::JpHL,
                        3 => Instruction::LdSPHL,
                        _ => Instruction::IllInsn,
                    },
                    _ => Instruction::IllInsn,
                },
                2 => match y {
                    4 => Instruction::Ldca,
                    5 => {
                        let word = self.mem.get_u16_at(self.reg.pc);
                        self.reg.pc += 2;
                        Instruction::LdaNN(word)
                    }
                    6 => Instruction::Ldac,
                    7 => {
                        let word = self.mem.get_u16_at(self.reg.pc);
                        self.reg.pc += 2;
                        Instruction::Lda(word)
                    }
                    _ => {
                        let addr = self.mem.get_u16_at(self.reg.pc);
                        self.reg.pc += 2;
                        Instruction::JpCond(isa::TABLE_CC[y as usize], addr)
                    }
                },
                3 => match y {
                    0 => {
                        let addr = self.mem.get_u16_at(self.reg.pc);
                        self.reg.pc += 2;
                        Instruction::Jp(addr)
                    }
                    1 => self.decode_cb(),
                    // Lots of Z80 instructions here that dont have GB equivalents
                    // Skip to y = <case that can exist>
                    6 => Instruction::Di,
                    7 => Instruction::Ei,
                    _ => Instruction::IllInsn,
                },
                4 => {
                    if y > 3 {
                        return Instruction::IllInsn;
                    }
                    let addr = self.mem.get_u16_at(self.reg.pc);
                    self.reg.pc += 2;
                    Instruction::CallCond(isa::TABLE_CC[y as usize], addr)
                }
                5 => match q {
                    0 => Instruction::Push(isa::TABLE_RP2[p as usize]),
                    1 => match p {
                        0 => {
                            let addr = self.mem.get_u16_at(self.reg.pc);
                            self.reg.pc += 2;
                            Instruction::Call(addr)
                        }
                        _ => Instruction::IllInsn, // Including FD prefix; doesnt exist on GB
                    },
                    _ => Instruction::IllInsn,
                },
                6 => {
                    let oper = self.mem.get_addr(self.reg.pc);
                    self.reg.pc += 1;
                    isa::alu_imm(y as usize, oper)
                }
                7 => Instruction::Rst(y * 8),
                _ => Instruction::IllInsn,
            },
            _ => Instruction::IllInsn,
        }
    }

    fn decode_cb(&mut self) -> Instruction {
        let ins0 = self.mem.get_addr(self.reg.pc);
        self.reg.pc += 1;
        let x = (ins0 & 0xc0) >> 6;
        let y = (ins0 & 0x38) >> 3;
        let z = ins0 & 0x7;

        match x {
            0 => isa::rot(y as usize, isa::TABLE_R[z as usize]),
            1 => Instruction::Bit(y, isa::TABLE_R[z as usize]),
            2 => Instruction::Res(y, isa::TABLE_R[z as usize]),
            3 => Instruction::Set(y, isa::TABLE_R[z as usize]),
            _ => Instruction::IllInsn,
        }
    }
}
