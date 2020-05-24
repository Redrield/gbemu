pub static TABLE_R: [RegisterName; 8] = [
    RegisterName::B,
    RegisterName::C,
    RegisterName::D,
    RegisterName::E,
    RegisterName::H,
    RegisterName::L,
    RegisterName::HLRef,
    RegisterName::A,
];
pub static TABLE_RP: [RegisterName; 4] = [
    RegisterName::BC,
    RegisterName::DE,
    RegisterName::HL,
    RegisterName::SP,
];
pub static TABLE_RP2: [RegisterName; 4] = [
    RegisterName::BC,
    RegisterName::DE,
    RegisterName::HL,
    RegisterName::AF,
];
pub static TABLE_CC: [JpCond; 4] = [
    JpCond::NotZero,
    JpCond::Zero,
    JpCond::NotCarry,
    JpCond::Carry,
];

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RegisterName {
    A,
    F,
    AF,
    B,
    C,
    BC,
    D,
    E,
    DE,
    H,
    L,
    HL,
    BCRef,
    DERef,
    HLRef, // LUL
    SP,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum JpCond {
    NotZero,
    Zero,
    NotCarry,
    Carry,
}

pub fn rot(idx: usize, reg: RegisterName) -> Instruction {
    match idx {
        0 => Instruction::Rlc(reg),
        1 => Instruction::Rrc(reg),
        2 => Instruction::Rl(reg),
        3 => Instruction::Rr(reg),
        4 => Instruction::Sla(reg),
        5 => Instruction::Sra(reg),
        7 => Instruction::Srl(reg),
        _ => Instruction::IllInsn,
    }
}

pub fn alu_reg(idx: usize, reg: RegisterName) -> Instruction {
    match idx {
        0 => Instruction::AddReg(reg),
        1 => Instruction::AdcReg(reg),
        2 => Instruction::SubReg(reg),
        3 => Instruction::SbcReg(reg),
        4 => Instruction::AndReg(reg),
        5 => Instruction::XorReg(reg),
        6 => Instruction::OrReg(reg),
        7 => Instruction::CpReg(reg),
        _ => Instruction::IllInsn,
    }
}

pub fn alu_imm(idx: usize, oper: u8) -> Instruction {
    match idx {
        0 => Instruction::AddImm(oper),
        1 => Instruction::AdcImm(oper),
        2 => Instruction::SubImm(oper),
        3 => Instruction::SbcImm(oper),
        4 => Instruction::AndImm(oper),
        5 => Instruction::XorImm(oper),
        6 => Instruction::OrImm(oper),
        7 => Instruction::CpImm(oper),
        _ => Instruction::IllInsn,
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum Instruction {
    /// Any illegal opcode in decoding
    IllInsn,
    /// LD <reg>,n
    Ld8Imm(RegisterName, u8),
    /// LD <reg>,<reg>
    Ld8Reg(RegisterName, RegisterName),
    /// HALT
    Halt,
    /// NOP
    Nop,
    /// LD (<addr>), SP
    StoSP(u16),
    /// STOP
    Stop,
    /// LDD (HL), A
    LddHLA,
    /// LDD A, (HL)
    LddAHL,
    /// LDI A, (HL)
    LdiAHL,
    /// LDI (HL), A
    LdiHLA,
    /// LDH (n), A
    /// LD ($FF00 + n),A
    LdhN(u8),
    /// LDH A, (n)
    LdhA(u8),
    /// LD A, (nn)
    Lda(u16),
    /// LD (nn),A
    LdaNN(u16),
    /// LD A,(0xFF00+C)
    Ldac,
    /// LD (0xFF00+C),A
    Ldca,
    /// LD <reg>,<word>
    Ld16Imm(RegisterName, u16),
    /// LD SP,HL
    LdSPHL,
    /// LDHL SP,n
    LdHLSPn(i8),
    /// LD (nn),SP
    LdnnSP(u16),
    /// PUSH <reg>
    Push(RegisterName),
    /// POP <reg>
    Pop(RegisterName),
    /// ADD <u8>
    AddImm(u8),
    /// ADD <reg>
    AddReg(RegisterName),
    /// ADC <u8>
    AdcImm(u8),
    /// ADC <reg>
    AdcReg(RegisterName),
    /// SUB <u8>
    SubImm(u8),
    /// SUB <reg>
    SubReg(RegisterName),
    /// SBC <u8>
    SbcImm(u8),
    /// SBC <reg>
    SbcReg(RegisterName),
    /// AND <u8>
    AndImm(u8),
    /// AND <reg>
    AndReg(RegisterName),
    /// OR <u8>
    OrImm(u8),
    /// OR <reg>
    OrReg(RegisterName),
    /// XOR <u8>
    XorImm(u8),
    /// XOR <reg>
    XorReg(RegisterName),
    /// CP <u8>
    CpImm(u8),
    /// CP <reg>
    CpReg(RegisterName),
    /// INC <reg>
    Inc(RegisterName),
    /// DEC <reg>
    Dec(RegisterName),
    /// ADD HL,<reg16>
    Add16(RegisterName),
    /// ADD SP,n
    AddSP(i8),
    /// INC <reg16>
    Inc16(RegisterName),
    /// DEC <reg16>
    Dec16(RegisterName),
    /// SWAP <reg>
    Swap(RegisterName),
    /// DAA
    Daa,
    /// CPL
    Cpl,
    /// CCF
    Ccf,
    /// SCF
    Scf,
    /// DI
    Di,
    /// EI
    Ei,
    /// RLCA
    Rlca,
    /// RLA
    Rla,
    /// RRCA
    Rrca,
    /// RRA
    Rra,
    /// RLC <reg>
    Rlc(RegisterName),
    /// RL <reg>
    Rl(RegisterName),
    /// RRC <reg>
    Rrc(RegisterName),
    /// RR <reg>
    Rr(RegisterName),
    /// SLA <reg>
    Sla(RegisterName),
    /// SRA <reg>
    Sra(RegisterName),
    /// SRL <reg>
    Srl(RegisterName),
    /// BIT <u8>,<reg>
    Bit(u8, RegisterName),
    /// SET <u8>,<reg>
    Set(u8, RegisterName),
    /// RES <u8>,<reg>
    Res(u8, RegisterName),
    /// JP <addr>
    Jp(u16),
    /// JP <cond>,<addr>
    JpCond(JpCond, u16),
    /// JR <rel>
    Jr(i8),
    /// JR <cond>,<rel>
    JrCond(JpCond, i8),
    /// JP (HL)
    JpHL,
    /// CALL <addr>
    Call(u16),
    /// CALL <cond>,<addr>
    CallCond(JpCond, u16),
    /// RST <u8>
    Rst(u8),
    /// RET
    Ret,
    /// RET <cond>
    RetCond(JpCond),
    /// RETI
    Reti,
}
