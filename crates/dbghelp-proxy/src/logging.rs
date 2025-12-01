use std::env;

use tracing::level_filters::LevelFilter;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

const ENV_LOG_FILE_FILTER: &str = "EML_LOG_FILE_FILTER";
const ENV_LOG_FILE_ENABLED: &str = "EML_LOG_FILE_ENABLED";
const ENV_LOG_FILE_PATH: &str = "EML_LOG_FILE_PATH";
const ENV_LOG_FILE_MAX: &str = "EML_LOG_FILE_MAX";

const ENV_LOG_STDOUT_FILTER: &str = "EML_LOG_STDOUT_FILTER";
const ENV_LOG_STDOUT_ENABLED: &str = "EML_LOG_STDOUT_ENABLED";

const DEFAULT_LOG_PATH: &str = "./logs";
const DEFAULT_MAX_LOG_FILES: usize = 128;

pub fn setup() {
    let enabled_file = env_flag(ENV_LOG_FILE_ENABLED, true);
    let enabled_stdout = env_flag(ENV_LOG_STDOUT_ENABLED, true);

    let file_layer = if enabled_file {
        let path = env_string(ENV_LOG_FILE_PATH, DEFAULT_LOG_PATH);
        let max_log_files = env_usize(ENV_LOG_FILE_MAX, DEFAULT_MAX_LOG_FILES);

        let appender = RollingFileAppender::builder()
            .rotation(Rotation::DAILY)
            .filename_suffix("eml.log")
            .max_log_files(max_log_files)
            .build(path)
            .expect("failed to create log appender");

        let file_layer = fmt::layer()
            .json()
            .with_writer(appender)
            .with_filter(env_filter(ENV_LOG_FILE_FILTER));

        Some(file_layer)
    } else {
        None
    };

    let stdout_layer = if enabled_stdout {
        let stdout_layer = fmt::layer()
            .with_writer(std::io::stdout)
            .with_level(true)
            .with_target(true)
            .with_ansi(false)
            .with_filter(env_filter(ENV_LOG_STDOUT_FILTER));

        Some(stdout_layer)
    } else {
        None
    };

    tracing_subscriber::registry()
        .with(file_layer)
        .with(stdout_layer)
        .init();
}

fn env_filter(name: &str) -> EnvFilter {
    EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .with_env_var(name)
        .from_env_lossy()
}

fn env_flag(name: &str, default: bool) -> bool {
    env::var(name)
        .map(|s| matches!(s.to_lowercase().as_str(), "true" | "1" | "yes"))
        .unwrap_or(default)
}

fn env_usize(name: &str, default: usize) -> usize {
    env::var(name)
        .map(|s| s.parse::<usize>().unwrap_or(default))
        .unwrap_or(default)
}

fn env_string(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}
