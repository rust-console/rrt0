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
    use core::str::pattern::{Pattern, SearchStep, Searcher};
    use serial_test::serial;
    use std::sync::mpsc::{channel, Sender};

    // A stream type that can report all values printed to it.
    struct TxStream(Sender<String>);

    impl Write for TxStream {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            self.0.send(s.to_string()).map_err(|_| fmt::Error)
        }
    }

    // Check whether the string matches the needle with simplified wildcard syntax.
    //
    // This allows a special `*` wildcard character to match numbers. It is much faster than
    // a regular expression (which grinds `cargo miri test` to a near halt).
    struct Wildcard<'a> {
        needle: &'a str,
    }

    impl<'a> Wildcard<'a> {
        fn new(needle: &'a str) -> Self {
            Self { needle }
        }
    }

    impl<'a> Pattern<'a> for Wildcard<'_> {
        type Searcher = WildcardSearcher<'a>;

        fn into_searcher(self, haystack: &'a str) -> Self::Searcher {
            Self::Searcher::new(self.needle, haystack)
        }
    }

    struct WildcardSearcher<'a> {
        needle: Vec<String>,
        needle_index: usize,
        haystack: &'a str,
        haystack_pos: usize,
    }

    impl<'a> WildcardSearcher<'a> {
        fn new(needle: &str, haystack: &'a str) -> Self {
            let needle = needle.split('*').map(|s| s.to_string()).collect::<Vec<_>>();

            Self {
                needle,
                needle_index: 0,
                haystack,
                haystack_pos: 0,
            }
        }
    }

    // SAFETY: We only slice `self.haystack` on character boundaries.
    unsafe impl<'a> Searcher<'a> for WildcardSearcher<'a> {
        fn haystack(&self) -> &'a str {
            self.haystack
        }

        fn next(&mut self) -> SearchStep {
            if self.needle_index >= self.needle.len() {
                std::dbg!("early Done");
                return SearchStep::Done;
            }

            let needle = &self.needle[self.needle_index];
            let start = self.haystack_pos;
            let end = start + needle.len();

            self.needle_index += 1;
            self.haystack_pos = end;

            if end > self.haystack.len() || !self.haystack.is_char_boundary(end) {
                SearchStep::Done
            } else if self.haystack[start..end] == needle[..] {
                let index = &self.haystack[end..]
                    .find(|ref c| !char::is_ascii_digit(c))
                    .unwrap_or(0);

                self.haystack_pos += index;

                SearchStep::Match(start, end + index)
            } else {
                SearchStep::Done
            }
        }
    }

    fn matches(expected: &str, actual: &str) -> bool {
        let matcher = Wildcard::new(expected);
        let matches = actual.matches(matcher).collect::<Vec<_>>();
        if matches.is_empty() {
            return false;
        }

        let tail = matches[matches.len() - 1];

        actual.ends_with(tail)
    }

    // Testing boilerplate.
    macro_rules! io_test {
        (let stdout=$stdout:expr; let stderr=$stderr:expr; $body:tt) => {
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

            // Assertions can safely access Stream from the receiving end.
            assert!(matches($stdout, &stdout_rx.try_iter().collect::<String>()));
            assert!(matches($stderr, &stderr_rx.try_iter().collect::<String>()));

            // Plug the leak. This is safe because `stdout_tx` and `stderr_tx` will not
            // be used again.
            drop(stdout_box);
            drop(stderr_box);
        };
    }

    #[test]
    #[serial]
    fn test_dbg() {
        io_test! {
            let stdout = "";
            let stderr = &[
                "[src/macros.rs:*] a = 1\n",
                "[src/macros.rs:*] b = 2\n",
                "[src/macros.rs:*] b = 2\n",
                "[src/macros.rs:*] a = 1\n",
                "[src/macros.rs:*] crate::dbg!(b, a,) = (\n",
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
    #[serial]
    fn test_print() {
        io_test! {
            let stdout = "Foo bar!";
            let stderr = "";

            { crate::print!("Foo bar!"); }
        };
    }

    #[test]
    #[serial]
    fn test_println() {
        io_test! {
            let stdout = "Foo bar!\n";
            let stderr = "";

            { crate::println!("Foo bar!"); }
        };
    }

    #[test]
    #[serial]
    fn test_eprint() {
        io_test! {
            let stdout = "";
            let stderr = "Foo bar!";

            { crate::eprint!("Foo bar!"); }
        };
    }

    #[test]
    #[serial]
    fn test_eprintln() {
        io_test! {
            let stdout = "";
            let stderr = "Foo bar!\n";

            { crate::eprintln!("Foo bar!"); }
        };
    }
}
