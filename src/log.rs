use slog::{self, Drain};
use slog_async;
use slog_envlogger;
use slog_term;

use incite_gen::version;

pub fn system_logger() -> slog::Logger {
    let decorator = slog_term::TermDecorator::new().stderr().build();

    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let env_middleware = slog_envlogger::new(drain);
    let drain = slog_async::Async::default(env_middleware).fuse();

    let logger = slog::Logger::root(drain, o!("version" => version::short_sha()));
    logger
}

pub fn application_logger() -> slog::Logger {
    slog::Logger::root(slog::Discard.fuse(), o!())
}
