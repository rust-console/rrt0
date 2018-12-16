# `rrt0`

Simple cross-platform runtime / startup (like crt0).

## Supported platforms

* [Nintendo 64](./src/platforms/n64/)

## Primary goals

* Create a sane, platform-specific ABI environment
  * Set the stack pointer
  * Clear the `.bss` and `.sbss` sections (uninitialized static data)
  * Minimal hardware initialization (e.g. configuring the FPU)
  * Re-exports `F32Ext` and `F64Ext` traits from [`libm`](https://crates.io/crates/libm) (for platforms with an FPU)

## Usage

The `panic` function must be imported with `pub use`, or you will get missing-symbol errors at link time. The floating point trait extensions can also be used for convenience.

```rust
pub use rrt0::panic;
use rrt0::{F32Ext, F64Ext};
```
