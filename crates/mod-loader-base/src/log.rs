#![allow(unused_macros, unused_imports)]

macro_rules! info {
    ($($arg:tt)*) => {
        tracing::info!(target: "eml::base", $($arg)*)
    };
}

macro_rules! warning {
    ($($arg:tt)*) => {
        tracing::warn!(target: "eml::base", $($arg)*)
    };
}

macro_rules! error {
    ($($arg:tt)*) => {
        tracing::error!(target: "eml::base", $($arg)*)
    };
}

macro_rules! debug {
    ($($arg:tt)*) => {
        tracing::debug!(target: "eml::base", $($arg)*)
    };
}

pub(crate) use info;
pub(crate) use warning as warn; // use alias to resolve ambiguity with builtin attribute
pub(crate) use error;
pub(crate) use debug;
