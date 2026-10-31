/* Github Automated Installation Application
 C opyright (C) 2026 Ralyrona                    *

 This program is free software: you can redistribute it and/or modify
 it under the terms of the GNU General Public License as published by
 the Free Software Foundation, either version 3 of the License, or
 (at your option) any later version.

 This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details. */

#[macro_export]
macro_rules! fail {
    ($($arg:tt)*) => {{
        eprintln!("\x1B[1;34;1;31mFaliure\x1B[0m: {}", format_args!($($arg)*));
        std::process::exit(1);
    }};
}
#[macro_export]
macro_rules! err {
    ($($arg:tt)*) => {{
        eprintln!("\x1B[1;34;1;33mError\x1B[0m: {}", format_args!($($arg)*));
        std::process::exit(2);
    }};
}
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        print!("{} ... ", format_args!($($arg)*));
        let _ = std::io::stdout().flush();
    }};
}
#[macro_export]
macro_rules! done_but {
    ($($arg:tt)*) => {{
        eprint!("\x1B[1;34;1;36mDone but\x1B[0m {}", format_args!($($arg)*));
    }};
}
pub fn succeed() {
    eprintln!("\x1B[1;34;1;32mSuccess\x1B[0m");
}
pub fn finish(string: &str) {
    eprintln!("\x1b[1m{string} complete\x1b[0m");
    std::process::exit(0);
}
pub fn handle_result<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
    return match result {
        Ok(x) => x,
        Err(e) => fail!("{e}"),
    }
}
