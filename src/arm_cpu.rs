use bitfield_struct::bitfield;

pub mod instruction;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Arm7TDMI {
    registers: Registers,
}
impl Arm7TDMI {
    pub fn init() -> Self {
        Self {
            registers: Registers::init(),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
struct Registers {
    lo: [i32; 8],
    hi_usr: [i32; 8],
    hi_fiq: [i32; 7],
    hi_svc: [i32; 2],
    hi_abt: [i32; 2],
    hi_irq: [i32; 2],
    hi_und: [i32; 2],

    cpsr: Status,
    spsr_fiq: Status,
    spsr_svc: Status,
    spsr_abt: Status,
    spsr_irq: Status,
    spsr_und: Status,
}
impl Registers {
    fn init() -> Self {
        Self::default()
    }
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
struct Status {
    #[bits(5, default = 0b10011)]
    mode: u8,
    #[bits(default = false)]
    t: bool,
    #[bits(default = true)]
    f: bool,
    #[bits(default = true)]
    i: bool,
    #[bits(20)]
    _rfu: usize,
    v: bool,
    c: bool,
    z: bool,
    n: bool,
}
