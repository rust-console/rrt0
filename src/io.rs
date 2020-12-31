//! A `no_std`-compatible I/O implementation.
//!
//! This module exports the [`STDOUT`] and [`STDERR`] statics which provide methods for
//! initializing I/O. Once initialized, the [`println!`] and [`dbg!`] macros will write to the
//! custom [`core::fmt::Write`] trait object (aka `Stream`) configured.
//!
//! Example:
//!
//! ```
//! use rrt0::{println, io::STDOUT};
//!
//! static mut STREAM: String = String::new();
//!
//! fn main() {
//!     STDOUT.set(unsafe { &mut STREAM });
//!
//!     println!("Hello, world!");
//! }
//! ```
//!
//! # Safety
//!
//! Because the [`STDOUT`] and [`STDERR`] statics mutate statically allocated trait objects, it is
//! unsound to interact with the `Stream` without carefully synchronizing access. Gaining exclusive
//! (mutable) access to the `Stream` outside of the macros is always undefined behavior.
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
//!     STDOUT.set(unsafe { &mut STREAM });
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
use spin::Mutex;

/// This is the output stream type.
///
/// Note that this is a trait object meaning it uses dynamic dispatch. This type was chosen because
/// it is a fat pointer (can be statically allocated).
type Stream = &'static mut dyn Write;

/// Storage for `Stream` pointers.
///
/// This isolates the static mutable output Stream from the thread-safe `StdIo` wrapper.
static mut STORAGE: [Option<Stream>; 2] = [None, None];

/// Standard output.
///
/// Used by [`print!`] and [`println!`] macros. See [`StdIo`] for documentation.
///
/// [`print!`]: crate::print
/// [`println!`]: crate::println
pub static STDOUT: StdIo = StdIo::new(0);

/// Standard error.
///
/// Used by [`eprint!`], [`eprintln!`], and [`dbg!`] macros. See [`StdIo`] for documentation.
///
/// [`dbg!`]: crate::dbg
/// [`eprint!`]: crate::eprint
/// [`eprintln!`]: crate::eprintln
pub static STDERR: StdIo = StdIo::new(1);

/// The Error type for I/O.
///
/// This type does not capture any context. The only method that can fail is setting the Stream
/// handler.
pub struct Error;

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
    index: Mutex<usize>,
}

impl StdIo {
    /// Privately construct a new `StdIo`.
    const fn new(index: usize) -> Self {
        Self {
            index: Mutex::new(index),
        }
    }

    /// Locks the internal `Mutex` and calls the provided closure, passing a mutable reference to
    /// the `Stream`.
    #[doc(hidden)]
    pub fn with_lock(&self, f: impl Fn(Stream)) {
        let guard = self.index.lock();

        if let Some(stream) = Self::get_stream(*guard) {
            f(stream);
        }
    }

    /// Set the `Stream`, replacing any existing value.
    ///
    /// This should be used with caution since it mutates global state. Using this method is sound
    /// but it may lead to unexpected results, particularly when concurrency is involved.
    pub fn set(&self, stream: Stream) {
        let guard = self.index.lock();

        Self::set_stream(*guard, stream);
    }

    /// Private static method that isolates unsafe `STORAGE` reads to a single location.
    fn get_stream(index: usize) -> Option<&'static mut Stream> {
        unsafe { STORAGE[index].as_mut() }
    }

    /// Private method that isolates unsafe `STORAGE` writes to a single location.
    fn set_stream(index: usize, stream: Stream) {
        // SAFETY: The `Stream` is guaranteed to be a valid pointer and a lock is held to
        // prevent data races with `STORAGE`.
        unsafe {
            STORAGE[index] = Some(stream);
        }
    }
}
