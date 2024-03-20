use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

pub fn setup() -> eyre::Result<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));
    let fmt_layer = fmt::layer()
        .with_target(true)
        .without_time()
        .with_filter(env_filter);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .try_init()?;

    Ok(())
}
