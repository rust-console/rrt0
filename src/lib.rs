#![feature(core_intrinsics)]
#![feature(global_asm)]
#![feature(lang_items)]
#![feature(llvm_asm)]
#![warn(rust_2018_idioms)]
#![allow(unused_attributes)]
#![no_std]

mod math;
mod platforms;

pub use crate::platforms::*;
use core::panic::PanicInfo;

/// This is the executable start function, which directly follows the entry point.
#[cfg_attr(not(test), lang = "start")]
#[cfg(not(test))]
extern "C" fn start<T>(user_main: *const (), _argc: isize, _argv: *const *const u8) -> !
where
    T: Termination,
{
    let user_main: fn() -> T = unsafe { core::mem::transmute(user_main) };
    user_main();

    panic!("main() cannot return");
}

/// Termination trait required for the start function.
#[cfg_attr(not(test), lang = "termination")]
trait Termination {}

/// This implementation does the bare minimum to satisfy the executable start function.
impl Termination for () {}

/// This function is called on panic.
#[cfg_attr(not(test), panic_handler)]
#[no_mangle]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        // A loop without side effects may be optimized away by LLVM. This issue can be avoided with
        // a volatile no-op. See: https://github.com/rust-lang/rust/issues/28728
        unsafe { llvm_asm!("" :::: "volatile") };
    }
}

/// Error handler personality language item (current no-op, to satisfy clippy).
#[cfg_attr(not(test), lang = "eh_personality")]
#[no_mangle]
extern "C" fn rust_eh_personality() {}
