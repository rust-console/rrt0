/// A `no_std` implementation of [`std::dbg!`].
///
/// I/O must be configured by a higher-level platform crate using [`no_stdout::init`].
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
/// I/O must be configured by a higher-level platform crate using [`no_stdout::init`].
///
/// [`std::print!`]: https://doc.rust-lang.org/std/macro.print.html
#[macro_export]
macro_rules! print {
    ($($args:expr),+ $(,)?) => ({
        $crate::prelude::stdout()
            .write_fmt(format_args!($($args),+))
            .ok();
    });
}

/// A `no_std` implementation of [`std::println!`].
///
/// I/O must be configured by a higher-level platform crate using [`no_stdout::init`].
///
/// [`std::println!`]: https://doc.rust-lang.org/std/macro.println.html
#[macro_export]
macro_rules! println {
    ($($args:expr),+ $(,)?) => ($crate::print!("{}\n", format_args!($($args),+)));

    () => ($crate::print!("\n"));
}

/// A `no_std` implementation of [`std::eprint!`].
///
/// I/O must be configured by a higher-level platform crate using [`no_stdout::init`].
///
/// [`std::eprint!`]: https://doc.rust-lang.org/std/macro.eprint.html
#[macro_export]
macro_rules! eprint {
    ($($args:expr),+ $(,)?) => ({
        $crate::prelude::stdout()
            .write_fmt(format_args!($($args),+))
            .ok();
    });
}

/// A `no_std` implementation of [`std::eprintln!`].
///
/// I/O must be configured by a higher-level platform crate using [`no_stdout::init`].
///
/// [`std::eprintln!`]: https://doc.rust-lang.org/std/macro.eprintln.html
#[macro_export]
macro_rules! eprintln {
    ($($args:expr),+ $(,)?) => ($crate::eprint!("{}\n", format_args!($($args),+)));

    () => ($crate::eprint!("\n"));
}
