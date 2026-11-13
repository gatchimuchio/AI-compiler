#![no_std]

#[unsafe(no_mangle)]
pub extern "C" fn aic_main(x: u64) -> u64 {
    x.wrapping_sub(1)
}
