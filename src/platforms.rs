#[cfg(any(unix, windows, target_vendor = "nintendo64"))]
pub use crate::math::*;

#[cfg(target_vendor = "nintendo64")]
use core::arch::global_asm;

#[cfg(target_vendor = "nintendo64")]
global_asm!(include_str!("platforms/n64/entrypoint.s"));
