use bitfield_struct::bitfield;

use crate::util::{get_field32, get_flag32};

fn major(word: u32) -> u8 {
    get_field32(word, 25, 3) as u8
}



#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(align(8))]
pub struct Decoded {
    condition: Condition,
    operation: Operation,
}
impl Decoded {
    pub fn decode_arm(word: u32) -> Self {
        let condition = Self::decode_condition(word);
        let operation = match major(word) {
            0 => Self::decode_major_0(word),
            1 => Self::decode_major_1(word),
            2 => Self::decode_major_2(word),
            3 => Self::decode_major_3(word),
            5 => Self::decode_major_5(word),
            8.. => unreachable!(),
            _ => Operation::Undefined,
        };

        Self {
            condition,
            operation,
        }
    }
    pub fn decode_thumb(half: u16) -> Self {
        todo!()
    }
    pub fn decode_thumb_lo(word: u32) -> Self {
        Self::decode_thumb(word as u16)
    }
    pub fn decode_thumb_hi(word: u32) -> Self {
        Self::decode_thumb((word >> 16) as u16)
    }

    fn decode_reg(word: u32, at: u32) -> Reg {
        Reg(get_field32(word, at, 4) as u8)
    }
    fn decode_condition(word: u32) -> Condition {
        let code = get_field32(word, 28, 4);
        match code {
            0x0 => Condition::Eq      ,
            0x1 => Condition::Ne      ,
            0x2 => Condition::Cs      ,
            0x3 => Condition::Cc      ,
            0x4 => Condition::Mi      ,
            0x5 => Condition::Pl      ,
            0x6 => Condition::Vs      ,
            0x7 => Condition::Vc      ,
            0x8 => Condition::Hi      ,
            0x9 => Condition::Ls      ,
            0xA => Condition::Ge      ,
            0xB => Condition::Lt      ,
            0xC => Condition::Gt      ,
            0xD => Condition::Le      ,
            0xE => Condition::Always  ,
            0xF => Condition::Reserved,
            16.. => unreachable!(),
        }
    }


    fn decode_major_0(word: u32) -> Operation {
        let op_hi = get_field32(word, 23, 2) == 2;
        let s = get_flag32(word, 20);
        let se = get_flag32(word, 7);
        let fo = get_flag32(word, 4);

        if fo && se {
            Self::decode_multiplies_and_extra_mem_instructions(word)
        }
        else if fo && !se && !s && op_hi {
            Self::decode_misc_instructions(word)
        }
        else if fo && !se {
            let op2 = Self::decode_op2(word, false, false);
            Self::decode_data_proc(word, s, op2)
        }
        else if !fo && !s && op_hi {
            Self::decode_misc_instructions(word)
        }
        else {
            let op2 = Self::decode_op2(word, false, true);
            Self::decode_data_proc(word, s, op2)
        }
    }
    fn decode_multiplies_and_extra_mem_instructions(word: u32) -> Operation {
        todo!()
    }
    fn decode_misc_instructions(word: u32) -> Operation {
        let lo = get_field32(word, 4, 4);
        let hi = get_flag32(word, 21);
        let r = get_flag32(word, 22);

        if !hi && lo == 0 {
            let rd = Self::decode_reg(word, 12);
            Operation::Mrs(rd, r)
        }
        else if hi && lo == 0 {
            let mask = Self::decode_mask(word);
            let rm = Self::decode_reg(word, 0);
            Operation::Msr(rm, r, mask)
        }
        else if hi && lo == 1 {
            let rm = Self::decode_reg(word, 0);
            Operation::Bx(rm)
        }
        else {
            Operation::Undefined
        }
    }

    fn decode_major_1(word: u32) -> Operation {
        let op_hi = get_field32(word, 23, 2) == 2;
        let op_lo = get_flag32(word, 21);
        let s = get_flag32(word, 20);

        if !s && op_lo && op_hi {
            let mask = Self::decode_mask(word);
            let imm = Self::decode_imm32(word);
            let r = get_flag32(word, 22);
            Operation::MsrI(r, imm, mask)
        }
        else if !s && op_lo {
            Operation::Undefined
        }
        else {
            let op2 = Self::decode_op2(word, true, false);
            Self::decode_data_proc(word, s, op2)
        }
    }

    fn decode_data_proc(word: u32, s: bool, op2: Op2) -> Operation {
        let rd = Self::decode_reg(word, 12);
        let rn = Self::decode_reg(word, 16);


        let opc = get_field32(word, 21, 4);
        match opc {
            0 =>  Operation::And(s, rd, rn, op2),
            1 =>  Operation::Eor(s, rd, rn, op2),
            2 =>  Operation::Sub(s, rd, rn, op2),
            3 =>  Operation::Rsb(s, rd, rn, op2),
            4 =>  Operation::Add(s, rd, rn, op2),
            5 =>  Operation::Adc(s, rd, rn, op2),
            6 =>  Operation::Sbc(s, rd, rn, op2),
            7 =>  Operation::Rsc(s, rd, rn, op2),
            8 =>  Operation::Tst(rn, op2),
            9 =>  Operation::Teq(rn, op2),
            10 => Operation::Cmp(rn, op2),
            11 => Operation::Cmn(rn, op2),
            12 => Operation::Orr(s, rd, rn, op2),
            13 => Operation::Mov(s, rd, op2),
            14 => Operation::Bic(s, rd, rn, op2),
            15 => Operation::Mvn(s, rd, op2),
            16.. => unreachable!(),
        }
    }
    fn decode_op2(word: u32, has_imm: bool, shift_is_constant: bool) -> Op2 {
        if has_imm {
            let immediate = Self::decode_imm32(word);
            Op2::Immediate(immediate)
        }
        else {
            let rm = Self::decode_reg(word, 0);
            let typ = get_field32(word, 5, 2);
            let imm = get_field32(word, 7, 5) as u8;
            let rs = Self::decode_reg(word, 8);

            match (typ, shift_is_constant) {
                (0, true) =>  Op2::ShiftLeftImm(rm, imm),
                (0, false) => Op2::ShiftLeftReg(rm, rs),
                (1, true) =>  Op2::ShiftRightLogicalImm(rm, imm),
                (1, false) => Op2::ShiftRightLogicalReg(rm, rs),
                (2, true) =>  Op2::ShiftRightArithmeticImm(rm, imm),
                (2, false) => Op2::ShiftRightArithmeticReg(rm, rs),
                (3, true) =>  Op2::RotateRightImm(rm, imm),
                (3, false) => Op2::RotateRightReg(rm, rs),
                (4.., _) => unreachable!(),
            }
        }
    }
    fn decode_imm32(word: u32) -> Immediate {
        let immediate = get_field32(word, 0, 8) as u8;
            let rotate_amount = get_field32(word, 8, 4) as u8;
            Immediate { immediate, rotate_amount }
    }
    fn decode_mask(word: u32) -> Mask {
        let c = get_flag32(word, 16);
        let x = get_flag32(word, 17);
        let s = get_flag32(word, 18);
        let f = get_flag32(word, 19);
        
        Mask::new()
            .with_control(c)
            .with_extension(x)
            .with_status(s)
            .with_flags(f)
    }

    fn decode_major_2(word: u32) -> Operation {
        Self::decode_load_store(word, true)
    }
    fn decode_major_3(word: u32) -> Operation {
        let fo = get_flag32(word, 4);
        if fo {
            Operation::Undefined
        }
        else {
            Self::decode_load_store(word, false)
        }
    }
    fn decode_load_store(word: u32, imm_offset: bool) -> Operation {
        let mode = Self::decode_mode2(word, imm_offset);
        let rd = Self::decode_reg(word, 12);
        let b = get_flag32(word, 22);
        let l = get_flag32(word, 20);
        let t = get_flag32(word, 21);
        let p = get_flag32(word, 24);
        let access_override = !p && t;

        if !access_override {
            match (b, l) {
                (false, false) => Operation::Str(rd, mode),
                (false, true) => Operation::Ldr(rd, mode),
                (true, false) => Operation::Strb(rd, mode),
                (true, true) => Operation::Ldrb(rd, mode),
            }
        }
        else {
            match (b, l) {
                (false, false) => Operation::Strt(rd, mode),
                (false, true) => Operation::Ldrt(rd, mode),
                (true, false) => Operation::Strbt(rd, mode),
                (true, true) => Operation::Ldrbt(rd, mode),
            }
        }
    }
    fn decode_mode2(word: u32, imm_offset: bool) -> Mode2 {
        let p = get_flag32(word, 24);
        let subtract = !get_flag32(word, 23);
        let w = get_flag32(word, 21);
        let rn = Self::decode_reg(word, 16);

        let offset = if imm_offset {
            let offset = ((get_field32(word, 0, 12)) as u16).to_le_bytes();
            if !p {
                Offset2::PostIndexedImm(offset)
            }
            else if w {
                Offset2::PreIndexeedImm(offset)
            }
            else {
                Offset2::OffsetImm(offset)
            }
        }
        else {
            let rm = Self::decode_reg(word, 0);
            let shift_amount = get_field32(word, 7, 5) as u8;
            let typ = get_field32(word, 5, 2);

            if !p {
                match typ {
                    0 => Offset2::PostIndexedLSL(rm, shift_amount),
                    1 => Offset2::PostIndexedLSR(rm, shift_amount),
                    2 => Offset2::PostIndexedASR(rm, shift_amount),
                    3 => Offset2::PostIndexedROR(rm, shift_amount),
                    4.. => unreachable!(),
                }
            }
            else if w {
                match typ {
                    0 => Offset2::PreIndexeedLSL(rm, shift_amount),
                    1 => Offset2::PreIndexeedLSR(rm, shift_amount),
                    2 => Offset2::PreIndexeedASR(rm, shift_amount),
                    3 => Offset2::PreIndexeedROR(rm, shift_amount),
                    4.. => unreachable!(),
                }
            }
            else {
                match typ {
                    0 => Offset2::OffsetLSL(rm, shift_amount),
                    1 => Offset2::OffsetLSR(rm, shift_amount),
                    2 => Offset2::OffsetASR(rm, shift_amount),
                    3 => Offset2::OffsetROR(rm, shift_amount),
                    4.. => unreachable!(),
                }
            }

        };


        Mode2 {
            rn,
            offset,
            subtract,
        }
    }

    fn decode_major_5(word: u32) -> Operation {
        let l = get_flag32(word, 24);
        let bytes = word.to_le_bytes();
        let offset = [bytes[0], bytes[1], bytes[2]];
        Operation::B(l, offset)
    }
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Condition {
    Eq,
    Ne,
    Cs,
    Cc,
    Mi,
    Pl,
    Vs,
    Vc,
    Hi,
    Ls,
    Ge,
    Lt,
    Gt,
    Le,
    Always,
    Reserved,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Operation {
    Undefined,

    Mov(bool, Reg, Op2),
    Mvn(bool, Reg, Op2),
    Mrs(Reg, bool),
    Msr(Reg, bool, Mask),
    MsrI(bool, Immediate, Mask),

    Add(bool, Reg, Reg, Op2),
    Adc(bool, Reg, Reg, Op2),
    Sub(bool, Reg, Reg, Op2),
    Sbc(bool, Reg, Reg, Op2),
    Rsb(bool, Reg, Reg, Op2),
    Rsc(bool, Reg, Reg, Op2),
    Mul(bool, Reg, Reg, Reg),
    Mla(bool, Reg, Reg, Reg, Reg),
    Umull(bool, Reg, Reg, Reg, Reg),
    Umlal(bool, Reg, Reg, Reg, Reg),
    Smull(bool, Reg, Reg, Reg, Reg),
    Smlal(bool, Reg, Reg, Reg, Reg),
    Cmp(Reg, Op2),
    Cmn(Reg, Op2),

    Tst(Reg, Op2),
    Teq(Reg, Op2),
    And(bool, Reg, Reg, Op2),
    Eor(bool, Reg, Reg, Op2),
    Orr(bool, Reg, Reg, Op2),
    Bic(bool, Reg, Reg, Op2),

    B(bool, [u8; 3]),
    Bx(Reg),

    Ldr  (Reg, Mode2),
    Ldrt (Reg, Mode2),
    Ldrb (Reg, Mode2),
    Ldrbt(Reg, Mode2),
    Ldrsb(Reg, Mode3),
    Ldrh (Reg, Mode3),
    Ldrsh(Reg, Mode3),
    Ldm {
        rd: Reg,
        regs: [u8; 2],
        offset: Offset4,
    },

    Str  (Reg, Mode2),
    Strt (Reg, Mode2),
    Strb (Reg, Mode2),
    Strbt(Reg, Mode2),
    Strh (Reg, Mode3, bool),
    Stm {
        rd: Reg,
        regs: [u8; 2],
        offset: Offset4,
    },

    Swp(Reg, Reg, Reg),
    Swpb(Reg, Reg, Reg),

    Cdp {
        pcnum: u8,
        op1: u8,
        crd: Reg,
        crn: Reg,
        crm: Reg,
        op2: u8,
    },
    Mrc {
        cpnum: u8,
        op1: u8,
        rd: Reg,
        crn: u8,
        crm: u8,
    },
    Mcr {
        cpnum: u8,
        op1: u8,
        rd: Reg,
        crn: u8,
        crm: u8,
    },
    Ldc {
        cpnum: u8,
        crd: Reg,
        mode: Mode5,
    },
    Stc {
        cpnum: u8,
        crd: Reg,
        mode: Mode5,
    },

    Swi([u8; 3]),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Reg(u8);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Op2 {
    Immediate(Immediate),
    ShiftLeftImm(Reg, u8),
    ShiftLeftReg(Reg, Reg),
    ShiftRightLogicalImm(Reg, u8),
    ShiftRightLogicalReg(Reg, Reg),
    ShiftRightArithmeticImm(Reg, u8),
    ShiftRightArithmeticReg(Reg, Reg),
    RotateRightReg(Reg, Reg),
    RotateRightImm(Reg, u8),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Immediate {
    pub immediate: u8,
    pub rotate_amount: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Mode2 {
    pub rn: Reg,
    pub offset: Offset2,
    pub subtract: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Offset2 {
    OffsetImm([u8; 2]),
    OffsetLSL(Reg, u8),
    OffsetLSR(Reg, u8),
    OffsetASR(Reg, u8),
    OffsetROR(Reg, u8),

    PreIndexeedImm([u8; 2]),
    PreIndexeedLSL(Reg, u8),
    PreIndexeedLSR(Reg, u8),
    PreIndexeedASR(Reg, u8),
    PreIndexeedROR(Reg, u8),
    PostIndexedImm([u8; 2]),
    PostIndexedLSL(Reg, u8),
    PostIndexedLSR(Reg, u8),
    PostIndexedASR(Reg, u8),
    PostIndexedROR(Reg, u8),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Mode3 {
    pub rn: Reg,
    pub offset: Offset3,
    pub subtract: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Offset3 {
    OffsetImm(u8),
    PreIndexedImm( u8),
    PostIndexedImm( u8),
    OffsetReg( Reg),
    PreIndexedReg(Reg),
    PostIndexedReg(Reg),
}

#[bitfield(u8)]
#[derive(PartialEq, Eq, Hash)]
pub struct Offset4 {
    pub decrement: bool,
    pub after: bool,
    pub write_back: bool,
    pub cpsr_or_force_user: bool,
    #[bits(4)]
    _rfu: usize,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Mode5 {
    pub rn: Reg,
    pub offset: Offset5,
    pub subtract: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Offset5 {
    ImmediateOffset( u8),
    PreIndexed( u8),
    PostIndexed(u8),
}

#[bitfield(u8)]
#[derive(PartialEq, Eq, Hash)]
pub struct Mask {
    control: bool,
    extension: bool,
    status: bool,
    flags: bool,

    #[bits(4)]
    _rfu: usize,
}
