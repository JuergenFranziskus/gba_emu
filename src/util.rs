pub fn sext24(val: u32) -> i32 {
    let sign = val & (1 << 23) != 0;
    if sign {
        (val | 0xFF000000) as i32
    } else {
        val as i32
    }
}


pub const fn bitmask_32(at: u32, width: u32) -> u32 {
    let mut mask = 0;
    let mut i = 0;

    while i < width {
        mask <<= 1;
        mask |= 1;
        i += 1;
    }


    mask << at
}

pub fn get_field32(word: u32, at: u32, width: u32) -> u32 {
    let mask = bitmask_32(0, width);
    (word >> at) & mask
}
pub fn get_flag32(word: u32, at: u32) -> bool {
    get_field32(word, at, 1) != 0
}
