use tracing_appender::{non_blocking::WorkerGuard, rolling};
use tracing_subscriber::{
    fmt, fmt::format::PrettyFields, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

/// Initializes the file and stdout logger.
///
/// Returns a `WorkerGuard`.
/// It's used to flush the file-logger once the guard gets dropped.
pub fn init_logging() -> WorkerGuard {
    let file_appender = rolling::never("", "current_song2.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(
            fmt::layer()
                .with_ansi(false)
                .fmt_fields(PrettyFields::new())
                .with_writer(non_blocking),
        )
        .init();

    guard
}
