extern crate incite;
extern crate failure;
extern crate dotenv;
extern crate futures_await as futures;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_envlogger;
extern crate slog_json;
extern crate slog_term;

use dotenv::dotenv;
use futures::future::lazy;
use incite::setup::ServerConfig;
use slog::Drain;
use std::env;
use std::fs::OpenOptions;
use std::net::SocketAddr;
use std::path::Path;

const FALLBACK_SRV_ADDR: &str = "127.0.0.1:1119";
const FALLBACK_LOG_PATH: &str = "./server.log";

fn main() -> Result<(), failure::Error> {
    dotenv().ok();

    let server_address: SocketAddr = match env::var("SERVER_ADDRESS") {
        Ok(v) => v.parse()?,
        Err(_) => FALLBACK_SRV_ADDR.parse().unwrap(),
    };
    let log_path = match env::var("LOG_FILEPATH") {
        Ok(v) => v,
        Err(_) => FALLBACK_LOG_PATH.into(),
    };
    let log_path = Path::new(&log_path);

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

    let server = ServerConfig::builder()
        .bind_address(server_address)
        .application_logger(logger)
        .build();
    //
    let post_setup_callback = lazy(|| Ok::<(), std::io::Error>(()));
    server.build_lobby_server(post_setup_callback).run();

    Ok(())
}
