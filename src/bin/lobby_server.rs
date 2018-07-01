extern crate incite;
#[macro_use]
extern crate failure;
extern crate dotenv;
#[macro_use]
extern crate futures_await as futures;

use dotenv::dotenv;
use futures::future::lazy;
use futures::prelude::*;
use incite::ServerControl;
use std::env;
use std::net::SocketAddr;

fn main() -> Result<(), failure::Error> {
    dotenv().ok();

    // let decorator = slog_term::TermDecorator::new().build();
    // let drain = slog_term::FullFormat::new(decorator).build().fuse();
    // let drain = slog_async::Async::new(drain).build().fuse();
    // let log = slog::Logger::root(drain, o!("version" => "0.5"));

    let server_address: SocketAddr = env::var("SERVER_ADDRESS")?.parse()?;
    let server = ServerControl::builder()
        .bind_address(server_address)
        .build();
    //
    let post_setup_callback = lazy(|| {
        println!("Server is currently running!");
        Ok::<(), std::io::Error>(())
    });
    server.build_lobby_server(post_setup_callback).run();

    Ok(())
}
