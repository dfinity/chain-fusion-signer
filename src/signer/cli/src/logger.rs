use anyhow::Context;

/// Initalize the logger
///
/// Default log level is WARN, can be turned up to TRCE by adding -v flags
/// and down to CRIT by adding -q flags
pub(super) fn init_logger(verbose: u8, quiet: u8) -> anyhow::Result<slog::Logger> {
    use slog::Drain;

    let verbose = verbose.clamp(0, 3);
    let quiet = quiet.clamp(0, 2);
    let level = 3 + verbose - quiet;
    let level = slog::Level::from_usize(level as usize).with_context(|| "Invalid log level")?;

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator)
        .build()
        .filter_level(level)
        .fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    Ok(slog::Logger::root(drain, slog::o!()))
}
