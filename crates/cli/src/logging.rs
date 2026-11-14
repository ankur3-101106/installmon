use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize structured logging using tracing-subscriber.
pub fn init_logging(verbose: u8, quiet: bool, json_output: bool) {
    if quiet {
        return;
    }

    let default_filter = match verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(default_filter))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let registry = tracing_subscriber::registry().with(env_filter);

    if json_output {
        let json_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_target(false)
            .with_writer(std::io::stderr);
        let _ = registry.with(json_layer).try_init();
    } else {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_target(false)
            .with_writer(std::io::stderr);
        let _ = registry.with(fmt_layer).try_init();
    }
}
