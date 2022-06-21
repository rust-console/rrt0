#![cfg_attr(target_vendor = "nintendo64", feature(asm_experimental_arch))]
#![no_std]

mod io;
mod math;
mod platforms;
pub mod prelude;

pub use crate::platforms::*;

/// This will be called by entrypoint.s if the main function returns.
#[no_mangle]
fn panic_main() -> ! {
    panic!("Main cannot return");
}
