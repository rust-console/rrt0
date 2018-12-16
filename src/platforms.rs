#[cfg(any(unix, windows, target_vendor = "nintendo64"))]
pub use crate::math::*;

#[cfg(any(unix, windows, target_vendor = "nintendo64"))]
pub use libm::{F32Ext, F64Ext};

#[cfg(target_vendor = "nintendo64")]
global_asm!(include_str!("platforms/n64/entrypoint.s"));
