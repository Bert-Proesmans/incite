use slog;

pub fn application_logger() -> slog::Logger {
    slog::Logger::root(slog::Discard, o!())
}
