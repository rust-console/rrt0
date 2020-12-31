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

        $crate::io::STDOUT.with_lock(|stream| write(stream, format_args!($($args),+)).unwrap());
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

        $crate::io::STDERR.with_lock(|stream| write(stream, format_args!($($args),+)).unwrap());
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
    use spin::{Lazy, Mutex};

    // This lock provides synchronized access to the inner `Stream` for testing
    static MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    // Testing boilerplate.
    macro_rules! io_test {
        (let stdout=$stdout:expr; let stderr=$stderr:expr; $body:tt) => {
            let guard = crate::macros::test::MUTEX.lock();

            // Setup the `STDOUT` and `STDERR` streams.
            static mut STDOUT: String = String::new();
            static mut STDERR: String = String::new();

            // SAFETY: These statics are only accessible to individual threads, not globally.
            // Each `Stream` is only set while this thread holds a mutex.
            unsafe { crate::io::STDOUT.set(&mut STDOUT); }
            unsafe { crate::io::STDERR.set(&mut STDERR); }

            $body

            // SAFETY: These statics are only accessible to individual threads, not globally.
            // Each `Stream` is only read while this thread holds a mutex.
            assert_eq!(unsafe { &STDOUT }, $stdout);
            assert_eq!(unsafe { &STDERR }, $stderr);

            drop(guard);
        };
    }

    #[test]
    fn test_dbg() {
        io_test! {
            let stdout = "";
            // XXX: Note the line number could change easily...
            let stderr = &[
                "[src/macros.rs:132] a = 1\n",
                "[src/macros.rs:132] b = 2\n",
                "[src/macros.rs:132] b = 2\n",
                "[src/macros.rs:132] a = 1\n",
                "[src/macros.rs:132] crate::dbg!(b, a,) = (\n",
                "    2,\n",
                "    1,\n",
                ")\n",
            ]
            .iter()
            .cloned()
            .collect::<String>();

            {
                let a = 1;
                let b = 2;

                crate::dbg!(a, b, crate::dbg!(b, a,));
            }
        };
    }

    #[test]
    fn test_print() {
        io_test! {
            let stdout = "Foo bar!";
            let stderr = "";

            { crate::print!("Foo bar!"); }
        };
    }

    #[test]
    fn test_println() {
        io_test! {
            let stdout = "Foo bar!\n";
            let stderr = "";

            { crate::println!("Foo bar!"); }
        };
    }

    #[test]
    fn test_eprint() {
        io_test! {
            let stdout = "";
            let stderr = "Foo bar!";

            { crate::eprint!("Foo bar!"); }
        };
    }

    #[test]
    fn test_eprintln() {
        io_test! {
            let stdout = "";
            let stderr = "Foo bar!\n";

            { crate::eprintln!("Foo bar!"); }
        };
    }
}
