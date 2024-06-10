use std::io::{self, stdout, Write};

use gba_emu::arm_cpu::instruction::Decoded;

fn main() -> io::Result<()> {
    const BIOS_SIZE: usize = 16 * 1024;
    let bios: &[u8; BIOS_SIZE] = include_bytes!("./gba_bios.bin");
    let mut out = stdout();
    for i in (0..BIOS_SIZE).step_by(4).take(100) {
        let bytes = [bios[i], bios[i + 1], bios[i + 2], bios[i + 3]];
        let word = u32::from_le_bytes(bytes);
        let arm = Decoded::decode_arm(word);

        writeln!(out, "{i:0>8x}: {arm:?}")?;
    }

    Ok(())
}
