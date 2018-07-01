#![feature(proc_macro, proc_macro_non_items, generators)]

extern crate bytes;
extern crate dotenv;
extern crate incite_gen;
extern crate slog_async;
extern crate slog_envlogger;
extern crate slog_term;
extern crate tokio;
extern crate tokio_codec;
extern crate tokio_signal;

#[macro_use]
extern crate futures_await as futures;
#[macro_use]
extern crate typed_builder;
#[macro_use]
extern crate slog;
#[macro_use]
extern crate error_chain;

#[derive(Debug)]
pub enum Signal {
    None,
    CtrlC,
}

mod error {
    use super::{setup, Signal};
    use std::io;

    error_chain! {
        errors {
            UserCallback {
                description("A user supplied callback failed")
                display("The supplied callback returned an error.")
            }
            Interrupted(s: Signal) {
                description("The user interrupted by sending a signal")
                display("User interrupted program.")
            }
        }

        links {
            Setup(setup::Error, setup::ErrorKind);
        }

        foreign_links {
            Io(io::Error) #[doc = "Error during IO"];
        }
    }
}

mod log;
mod protocol;
mod setup;

pub use self::error::*;
use futures::prelude::*;
use std::net::SocketAddr;

#[derive(Debug, TypedBuilder)]
#[must_use = "Setup a server by consuming this structure."]
pub struct ServerControl {
    bind_address: SocketAddr,
    #[default = "None"]
    connect_address: Option<SocketAddr>,

    #[default = "log::system_logger()"]
    system_logger: slog::Logger,
    #[default = "log::application_logger()"]
    application_logger: slog::Logger,
}

impl ServerControl {
    // NOTE: on_setup_handler should be a lightweight future!
    pub fn build_lobby_server<F, E>(
        self,
        on_setup_handler: F,
    ) -> ServerHandle<impl Future<Item = (), Error = Error>>
    where
        F: Future<Item = (), Error = E> + Send + 'static,
        E: std::error::Error + Send + 'static,
    {
        let on_setup_handler =
            on_setup_handler.map_err(|e| Error::with_chain(e, ErrorKind::UserCallback));

        let server_runner = setup::setup_lobby_server(self)
            .map_err(Into::<Error>::into)
            .join(on_setup_handler)
            .map(|(server_data, _)| server_data)
            .and_then(|(server, state)| {
                setup::lobby_handle_connections(server, state).map_err(Into::<Error>::into)
            });

        ServerHandle::new(server_runner)
    }
}

#[must_use = "The server is not started until run is called on this structure."]
pub struct ServerHandle<F>(F)
where
    F: Future<Item = (), Error = Error> + Send + 'static;

impl<F> ServerHandle<F>
where
    F: Future<Item = (), Error = Error> + Send + 'static,
{
    fn new(setup: F) -> Self {
        ServerHandle(setup)
    }

    pub fn run(self) {
        let future = self.0.map_err(|err| {
            // TODO; Find a way to return this error back to the caller of this method.
            println!("Shutting down system because of following error:\n{:}", err);
        });

        tokio::run(future);
    }
}
