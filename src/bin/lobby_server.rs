extern crate incite;
#[macro_use]
extern crate failure;
extern crate dotenv;
#[macro_use]
extern crate futures_await as futures;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_envlogger;
extern crate slog_json;
extern crate slog_term;

use dotenv::dotenv;
use futures::future::lazy;
use futures::prelude::*;
use incite::ServerControl;
use slog::Drain;
use std::env;
use std::fs::OpenOptions;
use std::net::SocketAddr;
use std::path::Path;

fn main() -> Result<(), failure::Error> {
    dotenv().ok();

    let server_address: SocketAddr = env::var("SERVER_ADDRESS")?.parse()?;
    let log_path_var = env::var("LOG_FILEPATH")?;
    let log_path = Path::new(&log_path_var);

    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)?;
    let file_logger = slog_json::Json::default(log_file);

    let console_decorator = slog_term::TermDecorator::new().build();
    let console_logger = slog_term::CompactFormat::new(console_decorator).build();

    let multiplexed_logger = slog::Duplicate::new(
        slog::LevelFilter::new(console_logger, slog::Level::Trace),
        file_logger,
    ).ignore_res();

    let logger = slog_async::Async::new(multiplexed_logger).build();
    let logger = slog::Logger::root(logger.fuse(), o!());

    let server = ServerControl::builder()
        .bind_address(server_address)
        .application_logger(logger)
        .build();
    //
    let post_setup_callback = lazy(|| Ok::<(), std::io::Error>(()));
    server.build_lobby_server(post_setup_callback).run();

    Ok(())
}
