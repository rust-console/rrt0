//! `rrt0` is the Rust runtime zero for bare metal targets with special attention to game consoles.
//!
//! You can think of it as analogous to `crt0` in the C programming language. It performs some basic
//! bootstrap steps for running an application without an operating system. At a minimum, this means
//! setting up the stack, clearing the `.bss` section, providing a panic handler, and calling the
//! `main` function.
//!
//! There are some platform-specific steps as well:
//!
//! # Nintendo 64
//!
//! * Makes the FPU usable.
//! * Enables the PIF NMI.
//! * Writes the LBA for the embedded file system to a static location in low memory.
//!   * User applications can use the FS LBA to read resources from the cartridge domain.
//!
//! # I/O primitives
//!
//! Some I/O primitives are provided, which are similar in scope to the low level I/O functionality
//! in [`std`].
//!
//! * Standard output (`stderr`) and standard error (`stderr`) macros.
//!
//! # Math intrinsics
//!
//! Custom Rust targets will lack some compiler intrinsics like those needed for floating point math
//! operations. `rrt0` reexports some intrinsics provided by the [`libm`] crate.
//!
//! [`std`]: https://doc.rust-lang.org/std/index.html
//! [`libm`]: https://crates.io/crates/libm

#![feature(global_asm)]
#![cfg_attr(target_vendor = "nintendo64", feature(lang_items))]
#![feature(llvm_asm)]
#![cfg_attr(test, feature(pattern))]
#![warn(rust_2018_idioms)]
#![cfg_attr(target_vendor = "nintendo64", no_std)]

pub mod io;
mod macros;
pub mod math;
mod platforms;
pub mod prelude;

use core::panic::PanicInfo;

/// This is the executable start function, which directly follows the entry point.
#[cfg(target_vendor = "nintendo64")]
#[cfg_attr(target_vendor = "nintendo64", lang = "start")]
extern "C" fn start<T>(user_main: *const (), _argc: isize, _argv: *const *const u8) -> !
where
    T: Termination,
{
    let user_main: fn() -> T = unsafe { core::mem::transmute(user_main) };
    user_main();

    panic!("main() cannot return");
}

/// Termination trait required for the start function.
#[cfg_attr(target_vendor = "nintendo64", lang = "termination")]
trait Termination {}

/// This implementation does the bare minimum to satisfy the executable start function.
impl Termination for () {}

/// This function is called on panic.
#[no_mangle]
#[cfg_attr(target_vendor = "nintendo64", panic_handler)]
fn panic(panic_info: &PanicInfo<'_>) -> ! {
    eprintln!("Application {}", panic_info);

    loop {
        // A loop without side effects may be optimized away by LLVM. This issue can be avoided with
        // a volatile no-op. See: https://github.com/rust-lang/rust/issues/28728
        unsafe { llvm_asm!("" :::: "volatile") };
    }
}

/// Error handler personality language item (current no-op, to satisfy clippy).
#[no_mangle]
#[cfg(target_vendor = "nintendo64")]
#[cfg_attr(target_vendor = "nintendo64", lang = "eh_personality")]
extern "C" fn rust_eh_personality() {}
