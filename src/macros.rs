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
    use core::fmt::{self, Write};
    use spin::{Lazy, Mutex};
    use std::sync::mpsc::{channel, Sender};

    // This lock provides synchronized access to the inner `Stream` for testing
    static MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    struct TxStream(Sender<String>);

    impl Write for TxStream {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            self.0.send(s.to_string()).map_err(|_| fmt::Error)
        }
    }

    // Testing boilerplate.
    macro_rules! io_test {
        (let stdout=$stdout:expr; let stderr=$stderr:expr; $body:tt) => {
            let guard = crate::macros::test::MUTEX.lock();

            // Setup the `STDOUT` and `STDERR` streams.
            let (stdout_tx, stdout_rx) = channel();
            let (stderr_tx, stderr_rx) = channel();
            let mut stdout_box = Box::new(TxStream(stdout_tx));
            let mut stderr_box = Box::new(TxStream(stderr_tx));

            // SAFETY: Both `Sender<T>` are leaked intentionally to create static references.
            let stdout_tx = unsafe { Box::leak(Box::from_raw(&mut *stdout_box)) };
            let stderr_tx = unsafe { Box::leak(Box::from_raw(&mut *stderr_box)) };

            crate::io::STDOUT.set(stdout_tx);
            crate::io::STDERR.set(stderr_tx);

            $body

            // Assertions can be safely access Stream from the receiving end.
            assert_eq!(&stdout_rx.try_iter().collect::<String>(), $stdout);
            assert_eq!(&stderr_rx.try_iter().collect::<String>(), $stderr);

            // Plug the leak. This is safe because `stdout_tx` and `stderr_tx` will not
            // be used again.
            drop(stdout_box);
            drop(stderr_box);

            drop(guard);
        };
    }

    #[test]
    fn test_dbg() {
        io_test! {
            let stdout = "";
            // XXX: Note the line number could change easily...
            let stderr = &[
                "[src/macros.rs:150] a = 1\n",
                "[src/macros.rs:150] b = 2\n",
                "[src/macros.rs:150] b = 2\n",
                "[src/macros.rs:150] a = 1\n",
                "[src/macros.rs:150] crate::dbg!(b, a,) = (\n",
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
