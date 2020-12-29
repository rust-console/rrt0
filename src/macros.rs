/// A `no_std` implementation of [`std::dbg!`].
///
/// See [`io`][crate::io] for information on configuring Standard I/O.
///
/// [`std::dbg!`]: https://doc.rust-lang.org/std/macro.dbg.html
#[macro_export]
macro_rules! dbg {
    () => ($crate::eprintln!("[{}:{}]", file!(), line!()));

    ($arg:expr $(,)?) => {{
        // Use of `match` here is intentional because it affects the lifetimes of temporaries
        // See: https://stackoverflow.com/a/48732525/1063961
        match $arg {
            val => {
                $crate::eprintln!("[{}:{}] {} = {:#?}", file!(), line!(), stringify!($arg), &val);

                val
            }
        }
    }};

    ($($arg:expr),+ $(,)?) => {($($crate::dbg!($arg),)+)};
}

/// A `no_std` implementation of [`std::print!`].
///
/// See [`io`][crate::io] for information on configuring Standard I/O.
///
/// [`std::print!`]: https://doc.rust-lang.org/std/macro.print.html
#[macro_export]
macro_rules! print {
    ($($args:expr),+ $(,)?) => ({
        use core::fmt::write;

        $crate::io::STDOUT.with_lock(|stream| write(stream(), format_args!($($args),+)).unwrap());
    });
}

/// A `no_std` implementation of [`std::println!`].
///
/// See [`io`][crate::io] for information on configuring Standard I/O.
///
/// [`std::println!`]: https://doc.rust-lang.org/std/macro.println.html
#[macro_export]
macro_rules! println {
    ($($args:expr),+ $(,)?) => ($crate::print!("{}\n", format_args!($($args),+)));

    () => ($crate::print!("\n"));
}

/// A `no_std` implementation of [`std::eprint!`].
///
/// See [`io`][crate::io] for information on configuring Standard I/O.
///
/// [`std::eprint!`]: https://doc.rust-lang.org/std/macro.eprint.html
#[macro_export]
macro_rules! eprint {
    ($($args:expr),+ $(,)?) => ({
        use core::fmt::write;

        $crate::io::STDERR.with_lock(|stream| write(stream(), format_args!($($args),+)).unwrap());
    });
}

/// A `no_std` implementation of [`std::eprintln!`].
///
/// See [`io`][crate::io] for information on configuring Standard I/O.
///
/// [`std::eprintln!`]: https://doc.rust-lang.org/std/macro.eprintln.html
#[macro_export]
macro_rules! eprintln {
    ($($args:expr),+ $(,)?) => ($crate::eprint!("{}\n", format_args!($($args),+)));

    () => ($crate::eprint!("\n"));
}

#[cfg(test)]
mod test {
    use spin::{Lazy, Mutex, MutexGuard};

    static mut STDOUT: String = String::new();
    static mut STDERR: String = String::new();

    // This lock provides safe access to the inner String for testing
    static LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    fn lock<'a>() -> MutexGuard<'a, ()> {
        let guard = LOCK.lock();

        unsafe {
            STDOUT.clear();
            STDERR.clear();

            crate::io::STDOUT.set_once(|| &mut STDOUT).ok();
            crate::io::STDERR.set_once(|| &mut STDERR).ok();
        }

        guard
    }

    #[test]
    fn test_dbg() {
        let guard = lock();

        let a = 1;
        let b = 2;

        crate::dbg!(a, b, crate::dbg!(b, a,));

        // XXX: Note the line number could change easily...
        let expected = [
            "[src/macros.rs:108] a = 1\n",
            "[src/macros.rs:108] b = 2\n",
            "[src/macros.rs:108] b = 2\n",
            "[src/macros.rs:108] a = 1\n",
            "[src/macros.rs:108] crate::dbg!(b, a,) = (\n",
            "    2,\n",
            "    1,\n",
            ")\n",
        ]
        .iter()
        .cloned()
        .collect::<String>();

        unsafe {
            assert_eq!(&STDOUT, "");
            assert_eq!(&STDERR, &expected);
        }

        drop(guard);
    }

    #[test]
    fn test_print() {
        let guard = lock();

        crate::print!("Foo bar!");
        unsafe {
            assert_eq!(&STDOUT, "Foo bar!");
            assert_eq!(&STDERR, "");
            STDOUT.clear();
        }
        crate::print!("Foo bar!",);
        assert_eq!(unsafe { &STDOUT }, "Foo bar!");

        drop(guard);
    }

    #[test]
    fn test_println() {
        let guard = lock();

        crate::println!("Foo bar!");
        unsafe {
            assert_eq!(&STDOUT, "Foo bar!\n");
            assert_eq!(&STDERR, "");
            STDOUT.clear();
        }
        crate::println!("Foo bar!",);
        assert_eq!(unsafe { &STDOUT }, "Foo bar!\n");

        drop(guard);
    }

    #[test]
    fn test_eprint() {
        let guard = lock();

        crate::eprint!("Foo bar!");
        unsafe {
            assert_eq!(&STDOUT, "");
            assert_eq!(&STDERR, "Foo bar!");
            STDERR.clear();
        }
        crate::eprint!("Foo bar!",);
        assert_eq!(unsafe { &STDERR }, "Foo bar!");

        drop(guard);
    }

    #[test]
    fn test_eprintln() {
        let guard = lock();

        crate::eprintln!("Foo bar!");
        unsafe {
            assert_eq!(&STDOUT, "");
            assert_eq!(&STDERR, "Foo bar!\n");
            STDERR.clear();
        }
        crate::eprintln!("Foo bar!",);
        assert_eq!(unsafe { &STDERR }, "Foo bar!\n");

        drop(guard);
    }
}
