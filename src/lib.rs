#![feature(global_asm)]
#![feature(lang_items)]
#![warn(rust_2018_idioms)]
#![no_std]

mod math;
mod platforms;

use core::panic::PanicInfo;
pub use crate::platforms::*;

/// This is the executable start function, which directly follows the entry point.
#[lang = "start"]
extern "C" fn start<T>(user_main: fn() -> T, _argc: isize, _argv: *const *const u8) -> isize
where
    T: Termination,
{
    user_main().report() as isize
}

/// Termination trait required for the start function.
#[lang = "termination"]
trait Termination {
    fn report(self) -> i32;
}

/// This implementation does the bare minimum to satisfy the executable start function.
impl Termination for () {
    fn report(self) -> i32 {
        0
    }
}

/// This function is called on panic.
#[panic_handler]
#[no_mangle]
pub fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {}
}
