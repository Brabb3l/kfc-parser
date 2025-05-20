use std::sync::atomic::AtomicBool;

static LOGGING: AtomicBool = AtomicBool::new(true);

#[inline]
pub fn set_logging(enabled: bool) {
    LOGGING.store(enabled, std::sync::atomic::Ordering::SeqCst);
}

#[inline]
pub fn is_logging_enabled() -> bool {
    LOGGING.load(std::sync::atomic::Ordering::SeqCst)
}

macro_rules! info {
    ($message:expr) => {
        if $crate::logging::is_logging_enabled() {
            println!("{} {}", colored::Colorize::bold(colored::Colorize::blue("info:")), $message)
        }
    };
    ($message:expr, $($arg:tt)*) => {
        if $crate::logging::is_logging_enabled() {
            println!("{} {}", colored::Colorize::bold(colored::Colorize::blue("info:")), format!($message, $($arg)*))
        }
    };
}

macro_rules! warning {
    ($message:expr) => {
        if $crate::logging::is_logging_enabled() {
            println!("{} {}", colored::Colorize::bold(colored::Colorize::yellow("warning:")), $message)
        }
    };
    ($message:expr, $($arg:tt)*) => {
        if $crate::logging::is_logging_enabled() {
            println!("{} {}", colored::Colorize::bold(colored::Colorize::yellow("warning:")), format!($message, $($arg)*))
        }
    };
}

macro_rules! error {
    ($message:expr) => {
        if $crate::logging::is_logging_enabled() {
            println!("{} {}", colored::Colorize::bold(colored::Colorize::red("error:")), $message)
        }
    };
    ($message:expr, $($arg:tt)*) => {
        if $crate::logging::is_logging_enabled() {
            println!("{} {}", colored::Colorize::bold(colored::Colorize::red("error:")), format!($message, $($arg)*))
        }
    };
}

pub(crate) use info;
pub(crate) use warning as warn; // use alias to resolve ambiguity with builtin attribute
pub(crate) use error;
