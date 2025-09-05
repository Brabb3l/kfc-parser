use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

const ENV_LOG: &str = "EMM_LOG";

pub fn setup() {
    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_level(true)
        .with_target(true)
        .with_ansi(true);

    tracing_subscriber::registry()
        .with(env_filter(ENV_LOG))
        .with(stdout_layer)
        .init();
}

fn env_filter(name: &str) -> EnvFilter {
    EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .with_env_var(name)
        .from_env_lossy()
}
