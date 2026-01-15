#![allow(unused_macros, unused_imports)]

macro_rules! info {
    ($($arg:tt)*) => {
        tracing::info!(target: "eml", $($arg)*)
    };
}

macro_rules! warning {
    ($($arg:tt)*) => {
        tracing::warn!(target: "eml", $($arg)*)
    };
}

macro_rules! error {
    ($($arg:tt)*) => {
        tracing::error!(target: "eml", $($arg)*)
    };
}

macro_rules! debug {
    ($($arg:tt)*) => {
        tracing::debug!(target: "eml", $($arg)*)
    };
}

macro_rules! trace {
    ($($arg:tt)*) => {
        tracing::trace!(target: "eml", $($arg)*)
    };
}

macro_rules! info_span {
    ($($arg:tt)*) => {
        tracing::info_span!(target: "eml", $($arg)*)
    };
}

macro_rules! warn_span {
    ($($arg:tt)*) => {
        tracing::warn_span!(target: "eml", $($arg)*)
    };
}

macro_rules! error_span {
    ($($arg:tt)*) => {
        tracing::error_span!(target: "eml", $($arg)*)
    };
}

macro_rules! debug_span {
    ($($arg:tt)*) => {
        tracing::debug_span!(target: "eml", $($arg)*)
    };
}

macro_rules! trace_span {
    ($($arg:tt)*) => {
        tracing::trace_span!(target: "eml", $($arg)*)
    };
}

pub(crate) use info;
pub(crate) use warning as warn; // use alias to resolve ambiguity with builtin attribute
pub(crate) use error;
pub(crate) use debug;
pub(crate) use trace;

pub(crate) use info_span;
pub(crate) use warn_span;
pub(crate) use error_span;
pub(crate) use debug_span;
pub(crate) use trace_span;
