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
//! fn main() -> Result<(), rrt0::io::Error> {
//!     STDOUT.set_once(String::new())?;
//!
//!     println!("Hello, world!");
//!
//!     Ok(())
//! }
//! ```
//!
//! [`dbg!`]: crate::dbg
//! [`println!`]: crate::println

use core::cell::RefCell;
use core::fmt::Write;
use core::mem::{self, ManuallyDrop};
use core::ptr;
use spin::{Mutex, MutexGuard};

/// This is the output stream type.
///
/// Note that this is a trait object meaning it uses dynamic dispatch. This type was chosen because
/// it is a fat pointer (can be statically allocated).
type Stream<'a> = &'a mut dyn Write;

/// Standard output.
///
/// Used by [`print!`] and [`println!`] macros. See [`StdIo`] for documentation.
///
/// [`print!`]: crate::print
/// [`println!`]: crate::println
pub static STDOUT: StdIo = StdIo::new();

/// Standard error.
///
/// Used by [`eprint!`], [`eprintln!`], and [`dbg!`] macros. See [`StdIo`] for documentation.
///
/// [`dbg!`]: crate::dbg
/// [`eprint!`]: crate::eprint
/// [`eprintln!`]: crate::eprintln
pub static STDERR: StdIo = StdIo::new();

/// The Error type for I/O.
///
/// This type does not capture any context. The only method that can fail is [`StdIo::set_once()`].
#[derive(Copy, Clone, Debug)]
pub struct Error;

/// Statically allocated size for supported `Stream` types.
///
/// Holds 8 pointer-sized inner types on platforms with 32-bit pointers, or 4 pointer-sized types on
/// 64-bit. This is a best guess for reasonably sized types, and is subject to change.
const STORAGE_SIZE: usize = 32;

/// The layout of `&dyn Trait` is described in the [Unsafe Code Guidelines]. This type is used to
/// construct `Stream` from static storage at runtime.
///
/// [Unsafe Code Guideline]: https://rust-lang.github.io/unsafe-code-guidelines/layout/pointers.html#notes
#[repr(C)]
#[derive(Copy, Clone)]
struct MutDynTrait {
    data: *mut u8,
    vtable: *const u8,
}

impl MutDynTrait {
    const fn new() -> Self {
        Self {
            data: ptr::null_mut(),
            vtable: ptr::null(),
        }
    }
}

// SAFETY: Access to `MutDynTrait` fields are carefully controlled with an internal `Mutex`. These
// marker traits are necessary to prove to the compiler that it is safe to send `MutDynTrait` (which
// contains raw pointers) across thread boundaries.
//
// A safe alternative would be `core::sync::atomic::AtomicPtr`, at the cost of unnecessary overhead.
unsafe impl Send for MutDynTrait {}
unsafe impl Sync for MutDynTrait {}

/// The storage type for our statically allocated `Stream`.
///
/// This holds a copy of the `T` type initialized with [`set`]/[`set_once`] and a pointer to it.
#[repr(C, align(8))]
struct Storage {
    data: [u8; STORAGE_SIZE],
    ptr: MutDynTrait,
}

impl Storage {
    const fn new() -> Self {
        Self {
            data: [0; STORAGE_SIZE],
            ptr: MutDynTrait::new(),
        }
    }
}

/// Stream owner for I/O.
///
/// The [`STDOUT`] and [`STDERR`] statics are available to users for configuring I/O streams. This
/// type cannot be constructed by user code.
pub struct StdIo {
    mutex: Mutex<RefCell<Option<Storage>>>,
}

impl StdIo {
    /// Privately construct a new `StdIo`.
    const fn new() -> Self {
        Self {
            mutex: Mutex::new(RefCell::new(None)),
        }
    }

    /// Locks the internal `Mutex` and calls the provided closure, passing a mutable reference to
    /// the `Stream`.
    #[doc(hidden)]
    pub fn with_lock<'a>(&self, f: impl Fn(Stream<'a>)) {
        let guard = self.mutex.lock();

        let mut opt = guard.borrow_mut();
        if let Some(storage) = opt.as_mut() {
            // SAFETY: The pointer in internal `storage` is guaranteed by `set_stream()` to be a
            // valid `Stream`.
            let stream = unsafe { mem::transmute::<_, Stream<'a>>(storage.ptr) };

            f(stream);
        }
    }

    /// Initialize the `Stream` if one has not yet been configured, leaving any existing stream
    /// in place.
    ///
    /// # Errors
    ///
    /// An error indicates that a `Stream` has already been configured and it will not be replaced.
    ///
    /// # Panics
    ///
    /// The size of `T` must be less than or equal to the size defined in the private
    /// `STORAGE_SIZE` const (currently 32 bytes).
    ///
    /// The alignment of `T` must be less than or equal to 8 bytes.
    pub fn set_once<T>(&self, stream: T) -> Result<(), Error>
    where
        T: Write + 'static,
    {
        let length = mem::size_of::<ManuallyDrop<T>>();
        assert!(length <= STORAGE_SIZE);
        assert!(mem::align_of::<ManuallyDrop<T>>() <= 8);

        let guard = self.mutex.lock();

        if guard.borrow().is_some() {
            Err(Error)
        } else {
            // SAFETY: Type size and alignment have been checked for fitness with the static
            // allocation.
            unsafe { Self::set_stream(guard, stream) };
            Ok(())
        }
    }

    /// Initialize the `Stream`, replacing any existing value.
    ///
    /// This method _does not_ run the destructor on any previous value that gets replaced.
    /// For a more robust method of configuring the `Stream`, see [`StdIo::set_once`].
    ///
    /// This should be used with caution since it mutates global state. Using this method is sound
    /// but it may lead to unexpected results, particularly when concurrency is involved.
    ///
    /// # Panics
    ///
    /// The size of `T` must be less than or equal to the size defined in the private
    /// `STORAGE_SIZE` const (currently 32 bytes).
    ///
    /// The alignment of `T` must be less than or equal to 8 bytes.
    pub fn set<T>(&self, stream: T)
    where
        T: Write + 'static,
    {
        let length = mem::size_of::<ManuallyDrop<T>>();
        assert!(length <= STORAGE_SIZE);
        assert!(mem::align_of::<ManuallyDrop<T>>() <= 8);

        let guard = self.mutex.lock();

        // SAFETY: Type size and alignment have been checked for fitness with the static
        // allocation.
        unsafe { Self::set_stream(guard, stream) };
    }

    /// Private static method that moves the `T` type into the `StdIo` container and constructs a
    /// `Stream` from it.
    ///
    /// # Safety
    ///
    /// The `MutexGuard` prevents data races and panics from exclusive borrows on the internal
    /// `RefCell`. The `stream` is guaranteed by the compiler to impl `Write` so it is safe to
    /// transform into a `Stream` as long as the layout of `&mut dyn Trait` is stable. See the
    /// `MutDynTrait` type.
    ///
    /// Type size and alignment must be checked for fitness with the static allocation.
    unsafe fn set_stream<'a, T>(guard: MutexGuard<'a, RefCell<Option<Storage>>>, mut stream: T)
    where
        T: Write + 'static,
    {
        let mut storage = Storage::new();
        let object: Stream<'_> = &mut stream;

        storage.ptr.vtable = mem::transmute::<_, MutDynTrait>(object).vtable;

        ptr::copy_nonoverlapping::<ManuallyDrop<T>>(
            &ManuallyDrop::new(stream),
            storage.data.as_mut_ptr() as *mut ManuallyDrop<T>,
            1,
        );

        // Replaces any existing value without calling the destructor.
        guard.replace(Some(storage));

        // After `storage` is moved to its final location, reach inside and patch the pointer.
        let mut opt = guard.borrow_mut();
        let mut storage = opt.as_mut().unwrap();

        storage.ptr.data = storage.data.as_mut_ptr();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_set_once() {
        let stdio = StdIo::new();
        assert!(stdio.set_once(String::new()).is_ok());
        assert!(stdio.set_once(String::new()).is_err());
    }
}
