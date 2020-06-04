use super::*;

impl CPU {
    /// Executes the given Z80 instruction, updating registers and memory as appropriate
    ///
    /// This function returns an integer with the number of clock cycles that this operation would take
    /// on a genuine gameboy.
    pub fn execute(&mut self, ins: Instruction) -> u8 {
        match ins {
            Instruction::Ld8Reg(r1, r2) => match (r1, r2) {
                (RegisterName::HLRef, reg) => {
                    let hl = self.reg.read16(RegisterName::HL).unwrap();
                    self.mem.set_addr(hl, self.reg.read8(reg).unwrap());
                    8
                }
                (reg, RegisterName::HLRef) => {
                    let hl = self.reg.read16(RegisterName::HL).unwrap();
                    self.reg.write8(reg, self.mem.get_addr(hl));
                    8
                }
                (reg, RegisterName::BCRef) => {
                    let bc = self.reg.read16(RegisterName::BC).unwrap();
                    self.reg.write8(reg, self.mem.get_addr(bc));
                    8
                }
                (reg, RegisterName::DERef) => {
                    let de = self.reg.read16(RegisterName::DE).unwrap();
                    self.reg.write8(reg, self.mem.get_addr(de));
                    8
                }
                _ => {
                    self.reg.write8(r1, self.reg.read8(r2).unwrap());
                    4
                }
            },
            Instruction::IllInsn => 1,
            Instruction::Ld8Imm(r1, value) => {
                self.reg.write8(r1, value);
                8
            }
            Instruction::Halt => {
                self.state.halted = true;
                4
            }
            Instruction::Nop => {
                // println!("TILEMAP");
                // for addr in 0x8000..0x97FF {
                //     println!("{:#x}: {:#x}", addr, self.mem.get_addr(addr));
                // }
                // panic!();
                4
            }
            Instruction::StoSP(addr) => {
                let sp = self.reg.read16(RegisterName::SP).unwrap();
                let bytes = sp.to_le_bytes();
                self.mem.set_addr(addr, bytes[0]);
                self.mem.set_addr(addr + 1, bytes[1]);
                20
            }
            Instruction::Stop => {
                self.state.stopped = true;
                4
            }
            Instruction::LddHLA => {
                let addr = self.reg.read16(RegisterName::HL).unwrap();
                self.mem
                    .set_addr(addr, self.reg.read8(RegisterName::A).unwrap());
                self.reg.write16(RegisterName::HL, addr - 1);
                8
            }
            Instruction::LddAHL => {
                let addr = self.reg.read16(RegisterName::HL).unwrap();
                self.reg.write8(RegisterName::A, self.mem.get_addr(addr));
                self.reg.write16(RegisterName::HL, addr - 1);
                8
            }
            Instruction::LdiAHL => {
                let addr = self.reg.read16(RegisterName::HL).unwrap();
                self.reg.write8(RegisterName::A, self.mem.get_addr(addr));
                self.reg.write16(RegisterName::HL, addr + 1);
                8
            }
            Instruction::LdiHLA => {
                let addr = self.reg.read16(RegisterName::HL).unwrap();
                self.mem
                    .set_addr(addr, self.reg.read8(RegisterName::A).unwrap());
                self.reg.write16(RegisterName::HL, addr + 1);
                8
            }
            Instruction::LdhN(n) => {
                self.mem.set_addr(
                    0xFF00 + (n as u16),
                    self.reg.read8(RegisterName::A).unwrap(),
                );
                12
            }
            Instruction::LdhA(n) => {
                self.reg
                    .write8(RegisterName::A, self.mem.get_addr(0xFF00 + (n as u16)));
                12
            }
            Instruction::Lda(addr) => {
                self.reg.write8(RegisterName::A, self.mem.get_addr(addr));
                16
            }
            Instruction::LdaNN(addr) => {
                self.mem
                    .set_addr(addr, self.reg.read8(RegisterName::A).unwrap());
                16
            }
            Instruction::Ldac => {
                let addr = 0xFF00 + self.reg.read8(RegisterName::C).unwrap() as u16;
                self.reg.write8(RegisterName::A, self.mem.get_addr(addr));
                8
            }
            Instruction::Ldca => {
                let addr = 0xFF00 + self.reg.read8(RegisterName::C).unwrap() as u16;
                self.mem
                    .set_addr(addr, self.reg.read8(RegisterName::A).unwrap());
                8
            }
            Instruction::Ld16Imm(reg, val) => {
                self.reg.write16(reg, val);
                12
            }
            Instruction::LdSPHL => {
                self.reg
                    .write16(RegisterName::SP, self.reg.read16(RegisterName::HL).unwrap());
                8
            }
            Instruction::LdHLSPn(n) => {
                let sp = self.reg.read16(RegisterName::SP).unwrap();
                let (offset, overflow) = (sp as i16).overflowing_add(n as i16);

                let sp_lower = sp & 0xf;
                let n_lower = n & 0xf;

                let hc = (sp_lower as i8) + n_lower;

                self.reg.write16(RegisterName::HL, offset as u16);
                self.reg
                    .set_flags(false, false, hc & (1 << 4) != 0, overflow);
                12
            }
            Instruction::LdnnSP(nn) => {
                let bytes = self.reg.read16(RegisterName::SP).unwrap().to_le_bytes();
                self.mem.set_addr(nn, bytes[0]);
                self.mem.set_addr(nn + 1, bytes[1]);
                20
            }
            Instruction::Push(reg) => {
                let bytes = self.reg.read16(reg).unwrap().to_le_bytes();
                let sp = self.reg.read16(RegisterName::SP).unwrap() - 2;
                self.mem.set_addr(sp, bytes[0]);
                self.mem.set_addr(sp + 1, bytes[1]);
                self.reg.write16(RegisterName::SP, sp);
                16
            }
            Instruction::Pop(reg) => {
                let sp = self.reg.read16(RegisterName::SP).unwrap();

                let bytes = [self.mem.get_addr(sp), self.mem.get_addr(sp + 1)];
                self.reg.write16(reg, u16::from_le_bytes(bytes));
                self.reg.write16(RegisterName::SP, sp + 2);
                12
            }
            Instruction::AddImm(n) => {
                let a = self.reg.read8(RegisterName::A).unwrap();
                let (result, overflow) = a.overflowing_add(n);

                let half_add = a & 0xf + n & 0xf;

                self.reg.write8(RegisterName::A, result);
                self.reg
                    .set_flags(result == 0, false, half_add & (1 << 4) != 0, overflow);
                8
            }
            Instruction::AddReg(reg) => {
                let a = self.reg.read8(RegisterName::A).unwrap();
                let b = match reg {
                    RegisterName::HLRef => self
                        .mem
                        .get_addr(self.reg.read16(RegisterName::HL).unwrap()),
                    _ => self.reg.read8(reg).unwrap(),
                };

                let (result, overflow) = a.overflowing_add(b);

                let half_add = a & 0xf + b & 0xf;

                self.reg.write8(RegisterName::A, result);
                self.reg
                    .set_flags(result == 0, false, half_add & (1 << 4) != 0, overflow);
                if reg == RegisterName::HLRef {
                    8
                } else {
                    4
                }
            }
            Instruction::AdcImm(n) => {
                let carry = if self.reg.flag(Flag::Carry) { 1 } else { 0 };
                let a = self.reg.read8(RegisterName::A).unwrap();

                let (result, overflow) = a.overflowing_add(n + carry);

                let half_add = a & 0xf + (n + carry) & 0xf;

                self.reg
                    .set_flags(result == 0, false, half_add & (1 << 4) != 0, overflow);
                self.reg.write8(RegisterName::A, result);
                8
            }
            Instruction::AdcReg(reg) => {
                let carry = if self.reg.flag(Flag::Carry) { 1 } else { 0 };
                let a = self.reg.read8(RegisterName::A).unwrap();

                let b = match reg {
                    RegisterName::HLRef => self
                        .mem
                        .get_addr(self.reg.read16(RegisterName::HL).unwrap()),
                    _ => self.reg.read8(reg).unwrap(),
                };

                let (result, overflow) = a.overflowing_add(b + carry);

                let half_add = a & 0xf + (b + carry) & 0xf;

                self.reg
                    .set_flags(result == 0, false, half_add & (1 << 4) != 0, overflow);
                self.reg.write8(RegisterName::A, result);
                if reg == RegisterName::HLRef {
                    8
                } else {
                    4
                }
            }
            Instruction::SubImm(n) => {
                let a = self.reg.read8(RegisterName::A).unwrap();
                let result = a.wrapping_sub(n);
                let hc = result & (1 << 4) != 0;
                self.reg.set_flags(result == 0, true, hc, a < n);
                self.reg.write8(RegisterName::A, result);
                8
            }
            Instruction::SubReg(reg) => {
                let a = self.reg.read8(RegisterName::A).unwrap();
                let n = match reg {
                    RegisterName::HLRef => self
                        .mem
                        .get_addr(self.reg.read16(RegisterName::HL).unwrap()),
                    _ => self.reg.read8(reg).unwrap(),
                };
                let result = a.wrapping_sub(n);
                let hc = result & (1 << 4) != 0;
                self.reg.set_flags(result == 0, true, hc, a < n);
                self.reg.write8(RegisterName::A, result);
                if reg == RegisterName::HLRef {
                    8
                } else {
                    4
                }
            }
            Instruction::SbcImm(n) => {
                let c = if self.reg.flag(Flag::Carry) { 1 } else { 0 };
                let a = self.reg.read8(RegisterName::A).unwrap();

                let result = a.wrapping_sub(n + c);
                let hc = result & (1 << 4) != 0;
                self.reg.set_flags(result == 0, true, hc, a < (n + c));
                8
            }
            Instruction::SbcReg(reg) => {
                let c = if self.reg.flag(Flag::Carry) { 1 } else { 0 };
                let a = self.reg.read8(RegisterName::A).unwrap();
                match reg {
                    RegisterName::HLRef => {
                        let hl = self.reg.read16(RegisterName::HL).unwrap();
                        let n = self.mem.get_addr(hl);
                        let result = a.wrapping_sub(n + c);
                        let hc = result & (1 << 4) != 0;
                        self.reg.set_flags(result == 0, true, hc, a < n);
                        self.reg.write8(RegisterName::A, result);
                        8
                    }
                    _ => {
                        let n = self.reg.read8(reg).unwrap();
                        let result = a.wrapping_sub(n + c);
                        let hc = result & (1 << 4) != 0;
                        self.reg.set_flags(result == 0, true, hc, a < n);
                        self.reg.write8(RegisterName::A, result);
                        4
                    }
                }
            }
            Instruction::AndImm(n) => {
                let a = self.reg.read8(RegisterName::A).unwrap();

                let result = a & n;

                self.reg.set_flags(result == 0, false, true, false);
                self.reg.write8(RegisterName::A, result);
                8
            }
            Instruction::AndReg(reg) => {
                let n = match reg {
                    RegisterName::HLRef => self
                        .mem
                        .get_addr(self.reg.read16(RegisterName::HL).unwrap()),
                    _ => self.reg.read8(reg).unwrap(),
                };
                let a = self.reg.read8(RegisterName::A).unwrap();

                let result = a & n;
                self.reg.set_flags(result == 0, false, true, false);
                self.reg.write8(RegisterName::A, result);
                if reg == RegisterName::HLRef {
                    8
                } else {
                    4
                }
            }
            Instruction::OrImm(n) => {
                let a = self.reg.read8(RegisterName::A).unwrap();

                let result = a | n;
                self.reg.set_flags(result == 0, false, false, false);
                self.reg.write8(RegisterName::A, result);
                8
            }
            Instruction::OrReg(reg) => {
                let a = self.reg.read8(RegisterName::A).unwrap();
                let n = match reg {
                    RegisterName::HLRef => self
                        .mem
                        .get_addr(self.reg.read16(RegisterName::HL).unwrap()),
                    _ => self.reg.read8(reg).unwrap(),
                };

                let result = a | n;
                self.reg.set_flags(result == 0, false, false, false);
                self.reg.write8(RegisterName::A, result);
                if reg == RegisterName::HLRef {
                    8
                } else {
                    4
                }
            }
            Instruction::XorImm(n) => {
                let a = self.reg.read8(RegisterName::A).unwrap();

                let result = a ^ n;
                self.reg.set_flags(result == 0, false, false, false);
                self.reg.write8(RegisterName::A, result);
                8
            }
            Instruction::XorReg(reg) => {
                let a = self.reg.read8(RegisterName::A).unwrap();
                let n = match reg {
                    RegisterName::HLRef => self
                        .mem
                        .get_addr(self.reg.read16(RegisterName::HL).unwrap()),
                    _ => self.reg.read8(reg).unwrap(),
                };

                let result = a ^ n;
                self.reg.set_flags(result == 0, false, false, false);
                self.reg.write8(RegisterName::A, result);
                if reg == RegisterName::HLRef {
                    8
                } else {
                    4
                }
            }
            Instruction::CpImm(n) => {
                let a = self.reg.read8(RegisterName::A).unwrap();
                let result = a.wrapping_sub(n);
                let hc = result & (1 << 4) != 0;
                self.reg.set_flags(result == 0, true, hc, a < n);
                8
            }
            Instruction::CpReg(reg) => {
                let a = self.reg.read8(RegisterName::A).unwrap();
                let n = match reg {
                    RegisterName::HLRef => self
                        .mem
                        .get_addr(self.reg.read16(RegisterName::HL).unwrap()),
                    _ => self.reg.read8(reg).unwrap(),
                };

                let result = a - n;
                let hc = result & (1 << 4) != 0;

                self.reg.set_flags(result == 0, true, hc, a < n);
                if reg == RegisterName::HLRef {
                    8
                } else {
                    4
                }
            }
            Instruction::Inc(reg) => match reg {
                RegisterName::HLRef => {
                    let hl = self.reg.read16(RegisterName::HL).unwrap();
                    let x = self.mem.get_addr(hl);
                    let (result, _) = x.overflowing_add(1);
                    let hc = x & 0xf == 0xf;
                    self.reg.set_flag(Flag::Zero, result == 0);
                    self.reg.set_flag(Flag::AddSub, false);
                    self.reg.set_flag(Flag::HalfCarry, hc);
                    self.mem.set_addr(hl, result);
                    12
                }
                _ => {
                    let x = self.reg.read8(reg).unwrap();

                    let (result, _) = x.overflowing_add(1);
                    let hc = x & 0xf == 0xf;
                    self.reg.set_flag(Flag::Zero, result == 0);
                    self.reg.set_flag(Flag::AddSub, false);
                    self.reg.set_flag(Flag::HalfCarry, hc);
                    self.reg.write8(reg, result);
                    4
                }
            },
            Instruction::Dec(reg) => match reg {
                RegisterName::HLRef => {
                    let hl = self.reg.read16(RegisterName::HL).unwrap();
                    let a = self.mem.get_addr(hl);
                    let result = a.wrapping_sub(1);
                    let hc = result & (1 << 4) != 0;

                    self.reg.set_flag(Flag::Zero, result == 0);
                    self.reg.set_flag(Flag::AddSub, true);
                    self.reg.set_flag(Flag::HalfCarry, hc);
                    self.mem.set_addr(hl, result);

                    12
                }
                _ => {
                    let a = self.reg.read8(reg).unwrap();
                    let result = a.wrapping_sub(1);
                    let hc = result & (1 << 4) != 0;

                    self.reg.set_flag(Flag::Zero, result == 0);
                    self.reg.set_flag(Flag::AddSub, true);
                    self.reg.set_flag(Flag::HalfCarry, hc);
                    self.reg.write8(reg, result);

                    4
                }
            },
            Instruction::Add16(reg) => {
                let reg = self.reg.read16(reg).unwrap();
                let hl = self.reg.read16(RegisterName::HL).unwrap();

                let result = hl.wrapping_add(reg);

                self.reg.set_flag(Flag::AddSub, false);
                self.reg.set_flag(Flag::HalfCarry, result & (1 << 11) != 0);
                self.reg.set_flag(Flag::Carry, result & (1 << 15) != 0);
                self.reg.write16(RegisterName::HL, reg + hl);

                8
            }
            Instruction::AddSP(n) => {
                let sp = self.reg.read16(RegisterName::SP).unwrap();
                let result = (sp as i16 + n as i16) as u16;
                self.reg.set_flag(Flag::HalfCarry, result & (1 << 11) != 0);
                self.reg.set_flag(Flag::Carry, result & (1 << 15) != 0);
                self.reg.write16(RegisterName::SP, result);
                16
            }
            Instruction::Inc16(reg) => {
                let value = self.reg.read16(reg).unwrap();
                self.reg.write16(reg, value.wrapping_add(1));
                8
            }
            Instruction::Dec16(reg) => {
                let value = self.reg.read16(reg).unwrap();
                self.reg.write16(reg, value.wrapping_sub(1));
                8
            }
            Instruction::Swap(reg) => match reg {
                RegisterName::HLRef => {
                    let hl = self.reg.read16(RegisterName::HL).unwrap();
                    let x = self.mem.get_addr(hl);
                    let low = (x & 0xf0) >> 4;
                    let high = x & 0xf << 4;
                    self.mem.set_addr(hl, high | low);
                    16
                }
                _ => {
                    let x = self.reg.read8(reg).unwrap();
                    let low = (x & 0xf0) >> 4;
                    let high = x & 0xf << 4;
                    self.reg.write8(reg, high | low);
                    8
                }
            },
            Instruction::Daa => {
                let value = self.reg.read8(RegisterName::A).unwrap();

                let mut carry = false;
                let result = if !self.reg.flag(Flag::AddSub) {
                    let mut result = value;
                    if self.reg.flag(Flag::Carry) || value > 0x99 {
                        carry = true;
                        result = result.wrapping_add(0x60);
                    }
                    if self.reg.flag(Flag::HalfCarry) || value & 0x0F > 0x09 {
                        result = result.wrapping_add(0x06);
                    }
                    result
                } else if self.reg.flag(Flag::Carry) {
                    carry = true;
                    let add = if self.reg.flag(Flag::HalfCarry) {
                        0x9A
                    } else {
                        0xA0
                    };
                    value.wrapping_add(add)
                } else if self.reg.flag(Flag::HalfCarry) {
                    value.wrapping_add(0xFA)
                } else {
                    value
                };

                self.reg.set_flag(Flag::Zero, result == 0);
                self.reg.set_flag(Flag::HalfCarry, false);
                self.reg.set_flag(Flag::Carry, carry);
                self.reg.write8(RegisterName::A, result);
                4
            }
            Instruction::Cpl => {
                let a = self.reg.read8(RegisterName::A).unwrap();
                self.reg.set_flag(Flag::AddSub, true);
                self.reg.set_flag(Flag::HalfCarry, true);
                self.reg.write8(RegisterName::A, !a);
                4
            }
            Instruction::Ccf => {
                self.reg.set_flag(Flag::AddSub, false);
                self.reg.set_flag(Flag::HalfCarry, false);
                self.reg.set_flag(Flag::Carry, !self.reg.flag(Flag::Carry));
                4
            }
            Instruction::Scf => {
                self.reg.set_flag(Flag::AddSub, false);
                self.reg.set_flag(Flag::HalfCarry, false);
                self.reg.set_flag(Flag::Carry, true);
                4
            }
            Instruction::Di => {
                self.state.di_pending = true;
                4
            }
            Instruction::Ei => {
                self.state.ei_pending = true;
                4
            }
            Instruction::Rlca => {
                let a = self.reg.read8(RegisterName::A).unwrap();
                let top = a & 0x80;
                let result = a << 1 | (top >> 7);
                self.reg.write8(RegisterName::A, result);
                self.reg.set_flags(result == 0, false, false, top != 0);
                4
            }
            Instruction::Rla => {
                let a = self.reg.read8(RegisterName::A).unwrap();
                let c = if self.reg.flag(Flag::Carry) { 1 } else { 0 };
                let top = a & 0x80;
                let result = (a << 1) | c;
                self.reg.write8(RegisterName::A, result);
                self.reg.set_flags(result == 0, false, false, top != 0);
                4
            }
            Instruction::Rrca => {
                let a = self.reg.read8(RegisterName::A).unwrap();
                let bottom = a & 1;
                let result = (bottom << 7) | a >> 1;
                self.reg.write8(RegisterName::A, result);
                self.reg.set_flags(result == 0, false, false, bottom != 0);
                4
            }
            Instruction::Rra => {
                let a = self.reg.read8(RegisterName::A).unwrap();
                let bottom = a & 1;
                let c = if self.reg.flag(Flag::Carry) { 1 } else { 0 };
                let result = (a >> 1) | (c << 7);
                self.reg.write8(RegisterName::A, result);
                self.reg.set_flags(result == 0, false, false, bottom != 0);
                4
            }
            Instruction::Rlc(reg) => match reg {
                RegisterName::HLRef => {
                    let hl = self.reg.read16(RegisterName::HL).unwrap();
                    let r = self.mem.get_addr(hl);
                    let top = r & 0x80;
                    let result = r << 1 | (top >> 7);
                    self.mem.set_addr(hl, result);
                    self.reg.set_flags(result == 0, false, false, top != 0);
                    16
                }
                _ => {
                    let r = self.reg.read8(reg).unwrap();
                    let top = r & 0x80;
                    let result = r << 1 | (top >> 7);
                    self.reg.write8(reg, result);
                    self.reg.set_flags(result == 0, false, false, top != 0);
                    8
                }
            },
            Instruction::Rl(reg) => match reg {
                RegisterName::HLRef => {
                    let hl = self.reg.read16(RegisterName::HL).unwrap();
                    let r = self.mem.get_addr(hl);
                    let c = if self.reg.flag(Flag::Carry) { 1 } else { 0 };
                    let top = r & 0x80;
                    let result = (r << 1) | c;
                    self.mem.set_addr(hl, result);
                    self.reg.set_flags(result == 0, false, false, top != 0);
                    16
                }
                _ => {
                    let r = self.reg.read8(reg).unwrap();
                    let c = if self.reg.flag(Flag::Carry) { 1 } else { 0 };
                    let top = r & 0x80;
                    let result = (r << 1) | c;
                    self.reg.write8(reg, result);
                    self.reg.set_flags(result == 0, false, false, top != 0);
                    8
                }
            },
            Instruction::Rrc(reg) => match reg {
                RegisterName::HLRef => {
                    let hl = self.reg.read16(RegisterName::HL).unwrap();
                    let r = self.mem.get_addr(hl);
                    let bottom = r & 1;
                    let result = (bottom << 7) | r >> 1;
                    self.mem.set_addr(hl, result);
                    self.reg.set_flags(result == 0, false, false, bottom != 0);
                    16
                }
                _ => {
                    let r = self.reg.read8(reg).unwrap();
                    let bottom = r & 1;
                    let result = (bottom << 7) | r >> 1;
                    self.reg.write8(reg, result);
                    self.reg.set_flags(result == 0, false, false, bottom != 0);
                    8
                }
            },
            Instruction::Rr(reg) => match reg {
                RegisterName::HLRef => {
                    let hl = self.reg.read16(RegisterName::HL).unwrap();
                    let r = self.mem.get_addr(hl);
                    let c = if self.reg.flag(Flag::Carry) { 1 } else { 0 };
                    let bottom = r & 1;
                    let result = (c << 7) | (r >> 1);

                    self.mem.set_addr(hl, result);
                    self.reg.set_flags(result == 0, false, false, bottom != 0);
                    16
                }
                _ => {
                    let r = self.reg.read8(reg).unwrap();
                    let c = if self.reg.flag(Flag::Carry) { 1 } else { 0 };
                    let bottom = r & 1;
                    let result = (c << 7) | (r >> 1);

                    self.reg.write8(reg, result);
                    self.reg.set_flags(result == 0, false, false, bottom != 0);
                    8
                }
            },
            Instruction::Sla(reg) => match reg {
                RegisterName::HLRef => {
                    let hl = self.reg.read16(RegisterName::HL).unwrap();
                    let r = self.mem.get_addr(hl);
                    let top = r & 0x80;
                    let result = r << 1;
                    self.mem.set_addr(hl, result);
                    self.reg.set_flags(result == 0, false, false, top != 0);
                    16
                }
                _ => {
                    let r = self.reg.read8(reg).unwrap();
                    let top = r & 0x80;
                    let result = r << 1;
                    self.reg.write8(reg, result);
                    self.reg.set_flags(result == 0, false, false, top != 0);
                    8
                }
            },
            Instruction::Sra(reg) => match reg {
                RegisterName::HLRef => {
                    let hl = self.reg.read16(RegisterName::HL).unwrap();
                    let r = self.mem.get_addr(hl);
                    let top = r & 0x80;
                    let bottom = r & 1;
                    let result = top | r >> 1;
                    self.mem.set_addr(hl, result);
                    self.reg.set_flags(result == 0, false, false, bottom != 0);
                    16
                }
                _ => {
                    let r = self.reg.read8(reg).unwrap();
                    let top = r & 0x80;
                    let bottom = r & 1;
                    let result = top | r >> 1;
                    self.reg.write8(reg, result);
                    self.reg.set_flags(result == 0, false, false, bottom != 0);
                    8
                }
            },
            Instruction::Srl(reg) => match reg {
                RegisterName::HLRef => {
                    let hl = self.reg.read16(RegisterName::HL).unwrap();
                    let r = self.mem.get_addr(hl);
                    let bottom = r & 1;
                    let result = r >> 1;
                    self.mem.set_addr(hl, result);
                    self.reg.set_flags(result == 0, false, false, bottom != 0);
                    16
                }
                _ => {
                    let r = self.reg.read8(reg).unwrap();
                    let bottom = r & 1;
                    let result = r >> 1;
                    self.reg.write8(reg, result);
                    self.reg.set_flags(result == 0, false, false, bottom != 0);
                    8
                }
            },
            Instruction::Bit(b, reg) => {
                let r = match reg {
                    RegisterName::HLRef => self
                        .mem
                        .get_addr(self.reg.read16(RegisterName::HL).unwrap()),
                    _ => self.reg.read8(reg).unwrap(),
                };

                self.reg.set_flag(Flag::Zero, r & (1 << b) == 0);
                self.reg.set_flag(Flag::AddSub, false);
                self.reg.set_flag(Flag::HalfCarry, true);
                if reg == RegisterName::HL {
                    16
                } else {
                    8
                }
            }
            Instruction::Set(b, reg) => match reg {
                RegisterName::HLRef => {
                    let hl = self.reg.read16(RegisterName::HL).unwrap();
                    let r = self.mem.get_addr(hl);
                    let result = r | (1 << b);
                    self.mem.set_addr(hl, result);
                    16
                }
                _ => {
                    let r = self.reg.read8(reg).unwrap();
                    let result = r | (1 << b);
                    self.reg.write8(reg, result);
                    8
                }
            },
            Instruction::Res(b, reg) => match reg {
                RegisterName::HLRef => {
                    let hl = self.reg.read16(RegisterName::HL).unwrap();
                    let r = self.mem.get_addr(hl);
                    let result = r & !(1 << b);
                    self.mem.set_addr(hl, result);
                    16
                }
                _ => {
                    let r = self.reg.read8(reg).unwrap();
                    let result = r & !(1 << b);
                    self.reg.write8(reg, result);
                    8
                }
            },
            Instruction::Jp(addr) => {
                self.reg.pc = addr;
                12
            }
            Instruction::JpCond(cond, addr) => {
                match cond {
                    JpCond::Carry => {
                        if self.reg.flag(Flag::Carry) {
                            self.reg.pc = addr;
                        }
                    }
                    JpCond::NotCarry => {
                        if !self.reg.flag(Flag::Carry) {
                            self.reg.pc = addr;
                        }
                    }
                    JpCond::Zero => {
                        if self.reg.flag(Flag::Zero) {
                            self.reg.pc = addr;
                        }
                    }
                    JpCond::NotZero => {
                        if !self.reg.flag(Flag::Zero) {
                            self.reg.pc = addr;
                        }
                    }
                }
                12
            }
            Instruction::Jr(rel) => {
                let pc = self.reg.pc;
                self.reg.pc = (pc as i32 + rel as i32) as u16;
                println!("JR PC OLD :{:#x}, NEW {:#x}", pc, self.reg.pc);

                8
            }
            Instruction::JrCond(cond, rel) => {
                let pc = self.reg.pc;
                let new_pc = (pc as i32 + rel as i32) as u16;

                match cond {
                    JpCond::Carry => {
                        if self.reg.flag(Flag::Carry) {
                            self.reg.pc = new_pc;
                        }
                    }
                    JpCond::NotCarry => {
                        if !self.reg.flag(Flag::Carry) {
                            self.reg.pc = new_pc;
                        }
                    }
                    JpCond::Zero => {
                        if self.reg.flag(Flag::Zero) {
                            self.reg.pc = new_pc;
                        }
                    }
                    JpCond::NotZero => {
                        if !self.reg.flag(Flag::Zero) {
                            self.reg.pc = new_pc;
                        }
                    }
                }
                8
            }
            Instruction::JpHL => {
                self.reg.pc = self.reg.read16(RegisterName::HL).unwrap();
                4
            }
            Instruction::Call(addr) => {
                let pc = self.reg.pc.to_le_bytes();
                let sp = self.reg.read16(RegisterName::SP).unwrap() - 2;
                self.mem.set_addr(sp, pc[0]);
                self.mem.set_addr(sp + 1, pc[1]);
                self.reg.pc = addr;
                self.reg.write16(RegisterName::SP, sp);
                12
            }
            Instruction::CallCond(cond, addr) => {
                let pc = self.reg.pc.to_le_bytes();
                let sp = self.reg.read16(RegisterName::SP).unwrap() - 2;
                match cond {
                    JpCond::Carry => {
                        if self.reg.flag(Flag::Carry) {
                            self.mem.set_addr(sp, pc[0]);
                            self.mem.set_addr(sp + 1, pc[1]);
                            self.reg.pc = addr;
                            self.reg.write16(RegisterName::SP, sp);
                        }
                    }
                    JpCond::NotCarry => {
                        if !self.reg.flag(Flag::Carry) {
                            self.mem.set_addr(sp, pc[0]);
                            self.mem.set_addr(sp + 1, pc[1]);
                            self.reg.pc = addr;
                            self.reg.write16(RegisterName::SP, sp);
                        }
                    }
                    JpCond::Zero => {
                        if self.reg.flag(Flag::Zero) {
                            self.mem.set_addr(sp, pc[0]);
                            self.mem.set_addr(sp + 1, pc[1]);
                            self.reg.pc = addr;
                            self.reg.write16(RegisterName::SP, sp);
                        }
                    }
                    JpCond::NotZero => {
                        if !self.reg.flag(Flag::Zero) {
                            self.mem.set_addr(sp, pc[0]);
                            self.mem.set_addr(sp + 1, pc[1]);
                            self.reg.pc = addr;
                            self.reg.write16(RegisterName::SP, sp);
                        }
                    }
                }
                12
            }
            Instruction::Rst(addr) => {
                let pc = self.reg.pc.to_le_bytes();
                let sp = self.reg.read16(RegisterName::SP).unwrap() - 2;
                self.mem.set_addr(sp, pc[0]);
                self.mem.set_addr(sp + 1, pc[1]);
                self.reg.pc = addr as u16;
                self.reg.write16(RegisterName::SP, sp);
                32
            }
            Instruction::Ret => {
                let sp = self.reg.read16(RegisterName::SP).unwrap();
                let pc = [self.mem.get_addr(sp), self.mem.get_addr(sp + 1)];
                self.reg.pc = u16::from_le_bytes(pc);
                self.reg.write16(RegisterName::SP, sp + 2);
                8
            }
            Instruction::RetCond(cond) => {
                let sp = self.reg.read16(RegisterName::SP).unwrap();
                let pc = [self.mem.get_addr(sp), self.mem.get_addr(sp + 1)];
                match cond {
                    JpCond::Carry => {
                        if self.reg.flag(Flag::Carry) {
                            self.reg.pc = u16::from_le_bytes(pc);
                            self.reg.write16(RegisterName::SP, sp + 2);
                        }
                    }
                    JpCond::NotCarry => {
                        if !self.reg.flag(Flag::Carry) {
                            self.reg.pc = u16::from_le_bytes(pc);
                            self.reg.write16(RegisterName::SP, sp + 2);
                        }
                    }
                    JpCond::Zero => {
                        if self.reg.flag(Flag::Zero) {
                            self.reg.pc = u16::from_le_bytes(pc);
                            self.reg.write16(RegisterName::SP, sp + 2);
                        }
                    }
                    JpCond::NotZero => {
                        if !self.reg.flag(Flag::Zero) {
                            self.reg.pc = u16::from_le_bytes(pc);
                            self.reg.write16(RegisterName::SP, sp + 2);
                        }
                    }
                }
                8
            }
            Instruction::Reti => {
                let sp = self.reg.read16(RegisterName::SP).unwrap();
                let pc = [self.mem.get_addr(sp), self.mem.get_addr(sp + 1)];
                self.reg.pc = u16::from_le_bytes(pc);
                self.reg.write16(RegisterName::SP, sp + 2);
                self.state.ei_pending = true;
                8
            }
        }
    }
}
