//! A `no_std`-compatible I/O implementation.
//!
//! This module exports the [`STDOUT`] and [`STDERR`] statics which provide methods for
//! initializing I/O. Once initialized, the [`print!`] and [`dbg!`] macros will write to the custom
//! [`core::fmt::Write`] trait object (aka `Stream`) configured.
//!
//! Example:
//!
//! ```
//! use rrt0::{println, io::STDOUT};
//!
//! static mut STREAM: String = String::new();
//!
//! fn main() {
//!     STDOUT.set_once(|| unsafe { &mut STREAM }).unwrap();
//!
//!     println!("Hello, world!");
//! }
//! ```
//!
//! # Safety
//!
//! Because the [`STDOUT`] and [`STDERR`] statics mutate statically allocated trait objects, it is
//! unsound to interact with the `Stream` without carefully synchronizing access.
//!
//! The example above is sound because the [`println!`] family of macros can only reach the `Stream`
//! through an internal `Mutex` and no attempt is made my the user code to access the contents of
//! the `String`.
//!
//! Safely accessing your own `Stream` types requires synchronization around _all_ uses of
//! [`println!`] and friends, in addition to the `unsafe` access of the `Stream` itself. This should
//! not be an issue in the general case where a `Stream` is only responsible for writing to an
//! external resource (e.g. USB FIFO, file system, socket). It is unsound to access internal
//! resources (like `String`) when there are any macro invocations that are not synchronized.
//!
//! The following example is sound, because all uses of [`println!`] are synchronized with the same
//! `Mutex` guarding access to the static `String`.
//!
//! ```
//! use rrt0::{println, io::STDOUT};
//!
//! use spin::{Lazy, Mutex};
//! use std::thread;
//!
//! static mut STREAM: String = String::new();
//! static MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
//!
//! fn main() {
//!     STDOUT.set_once(|| unsafe { &mut STREAM }).unwrap();
//!
//!     let mut threads = vec![];
//!     for _ in 0..5 {
//!         threads.push(thread::spawn(handler));
//!     }
//!     for thread in threads {
//!         thread.join().unwrap();
//!     }
//! }
//!
//! fn handler() {
//!     let guard = MUTEX.lock();
//!     {
//!         // Safe because all access is synchronized by a Mutex.
//!         println!("Hello, world!");
//!         assert!(unsafe { &STREAM }.contains("Hello, world!\n"));
//!     }
//!     drop(guard);
//! }
//! ```
//!
//! This next example is also sound, because the `Stream` is never accessed. In this case, an extra
//! sync primitive is unnecessary. It uses the same `main` function as shown above.
//!
//! ```
//! # use rrt0::println;
//! fn handler() {
//!     // Safe because the static mut is never accessed
//!     println!("Hello, world!");
//! }
//! ```
//!
//! Finally we offer a counterexample that is unsound.
//!
//! ```ignore (Intentionally unsound and missing boilerplate)
//! fn handler() {
//!     // ERROR: Unsound because the `Stream` is accessed without synchronization.
//!     println!("Hello, world!");
//!
//!     let guard = MUTEX.lock();
//!     {
//!         // This access has undefined behavior with other threads accessing the `Stream`
//!         // outside of the lock.
//!         assert!(unsafe { &STREAM }.contains("Hello, world!\n"));
//!     }
//!     drop(guard);
//! }
//! ```
//!
//! Developers are encouraged to run their tests with tools for detecting data races, e.g. `TSan`
//! and `miri`. Both tools are included with Rust nightly, but require some additional dependencies.
//!
//! ```bash
//! $ rustup run nightly rustup component add miri rust-src
//!
//! $ cargo +nightly miri test --all --all-features
//! $ RUSTFLAGS='-Z sanitizer=thread' RUSTDOCFLAGS='-Z sanitizer=thread' \
//!     cargo +nightly test --all-features -Z build-std \
//!     --target $(basename $(dirname $(rustc --print target-libdir)))
//! ```
//!
//! [`dbg!`]: crate::dbg
//! [`print!`]: crate::print
//! [`println!`]: crate::println

use core::fmt::Write;
use spin::{Lazy, Mutex};

/// Standard output.
///
/// Used by [`print!`] and [`println!`] macros. See [`StdIo`] for documentation.
///
/// [`print!`]: crate::print
/// [`println!`]: crate::println
pub static STDOUT: Lazy<StdIo> = Lazy::new(StdIo::new);

/// Standard error.
///
/// Used by [`eprint!`], [`eprintln!`], and [`dbg!`] macros. See [`StdIo`] for documentation.
///
/// [`dbg!`]: crate::dbg
/// [`eprint!`]: crate::eprint
/// [`eprintln!`]: crate::eprintln
pub static STDERR: Lazy<StdIo> = Lazy::new(StdIo::new);

type Stream = fn() -> &'static mut dyn Write;

/// Stream owner for I/O.
///
/// The [`STDOUT`] and [`STDERR`] statics are available instances of this struct. This type cannot
/// be constructed by user code.
///
/// # Safety
///
/// See the [module-level documentation] for important information regarding safety when using this
/// type.
///
/// [module-level documentation]: crate::io
pub struct StdIo {
    cell: Mutex<Option<Stream>>,
}

impl StdIo {
    fn new() -> Self {
        Self {
            cell: Mutex::new(None),
        }
    }

    /// Locks the internal `Mutex` and calls the provided closure, passing a mutable reference to
    /// the `Stream`.
    #[doc(hidden)]
    pub fn with_lock(&self, f: impl Fn(Stream)) {
        let guard = self.cell.lock();

        if let Some(stream) = *guard {
            f(stream);
        }

        drop(guard)
    }

    /// Set the `Stream` if one has not yet been configured, leaving any existing stream in place.
    ///
    /// # Errors
    ///
    /// Fails when a `Stream` was previously configured. Will return the existing Stream reference
    /// in the `Err` variant.
    pub fn set_once(&self, stream: Stream) -> Result<(), Stream> {
        let mut guard = self.cell.lock();

        if let Some(existing) = *guard {
            Err(existing)
        } else {
            *guard = Some(stream);

            Ok(())
        }
    }

    /// Set the `Stream`, replacing any existing value.
    ///
    /// This should be used with caution since it mutates global state. Using this method is sound
    /// but it may lead to unexpected results, particularly when concurrency is involved.
    ///
    /// For a more robust method of configuring the `Stream`, see [`StdIo::set_once`].
    pub fn set(&self, stream: Stream) {
        let mut guard = self.cell.lock();

        *guard = Some(stream);
    }
}
