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
        eprintln!("\x1B[1;34;1;36mDone but\x1B[0m {}", format_args!($($arg)*));
    }};
}
pub fn succeed() {
    eprintln!("\x1B[1;34;1;32mSuccess\x1B[0m");
}
pub fn finish(string: &str) {
    eprintln!("\x1b[1m{string} complete\x1b[0m");
    std::process::exit(0);
}
