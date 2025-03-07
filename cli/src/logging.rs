macro_rules! info {
    ($message:expr) => {
        println!("{} {}", colored::Colorize::bold(colored::Colorize::blue("info:")), $message)
    };
    ($message:expr, $($arg:tt)*) => {
        println!("{} {}", colored::Colorize::bold(colored::Colorize::blue("info:")), format!($message, $($arg)*))
    };
}

macro_rules! warning {
    ($message:expr) => {
        println!("{} {}", colored::Colorize::bold(colored::Colorize::yellow("warning:")), $message)
    };
    ($message:expr, $($arg:tt)*) => {
        println!("{} {}", colored::Colorize::bold(colored::Colorize::yellow("warning:")), format!($message, $($arg)*))
    };
}

macro_rules! error {
    ($message:expr) => {
        println!("{} {}", colored::Colorize::bold(colored::Colorize::red("error:")), $message)
    };
    ($message:expr, $($arg:tt)*) => {
        println!("{} {}", colored::Colorize::bold(colored::Colorize::red("error:")), format!($message, $($arg)*))
    };
}

pub(crate) use info;
pub(crate) use warning as warn; // use alias to resolve ambiguity with builtin attribute
pub(crate) use error;
