use futures::prelude::*;
use slog;
use std;
use std::net::SocketAddr;
use tokio;

use super::{log, servers};

mod error {
    use super::*;
    use std::io;

    error_chain! {
        errors {
            UserCallback {
                description("A user supplied callback failed")
                display("The supplied callback returned an error.")
            }
        }

        links {
            LobbyServer(servers::lobby::Error, servers::lobby::ErrorKind);
        }

        foreign_links {
            Io(io::Error) #[doc = "Error during IO"];
        }
    }
}

pub use self::error::*;

#[derive(Debug, TypedBuilder)]
#[must_use = "Setup a server by consuming this structure."]
pub struct ServerConfig {
    pub bind_address: SocketAddr,
    #[default = "None"]
    pub connect_address: Option<SocketAddr>,

    #[default = "log::application_logger()"]
    pub application_logger: slog::Logger,
}

impl ServerConfig {
    // NOTE: on_setup_handler should be a lightweight future!
    pub fn build_lobby_server<F, E>(
        self,
        on_setup_handler: F,
    ) -> ServerHandle<impl Future<Item = (), Error = Error>, impl FnOnce(Error) -> ()>
    where
        F: Future<Item = (), Error = E> + Send + 'static,
        E: std::error::Error + Send + 'static,
    {
        let logger = self.application_logger.clone();
        let logger_err = self.application_logger.clone();

        let on_setup_handler =
            on_setup_handler.map_err(|e| Error::with_chain(e, ErrorKind::UserCallback));

        let server_runner = servers::lobby::setup_server(self)
            .map_err(Into::<Error>::into)
            .and_then(move |(server, state)| {
                trace!(logger, "Setup complete");
                on_setup_handler.join(
                    servers::lobby::handle_connections(server, state).map_err(Into::<Error>::into),
                )
            })
            .map(|_| ());

        let default_error_handler =
            move |err| crit!(logger_err, "Accepting clients failed!"; "error" => %err);
        ServerHandle::new(server_runner, default_error_handler)
    }
}

#[must_use = "The server is not started until run is called on this structure."]
pub struct ServerHandle<F, E>(F, E)
where
    F: Future<Item = (), Error = Error> + Send + 'static,
    E: FnOnce(Error) -> () + Send + 'static;

impl<F, E> ServerHandle<F, E>
where
    F: Future<Item = (), Error = Error> + Send + 'static,
    E: FnOnce(Error) -> () + Send + 'static,
{
    fn new(setup: F, error_handler: E) -> Self {
        ServerHandle(setup, error_handler)
    }

    pub fn on_error<NewErr>(self, error_handler: NewErr) -> ServerHandle<F, NewErr>
    where
        NewErr: FnOnce(Error) -> () + Send + 'static,
    {
        ServerHandle(self.0, error_handler)
    }

    pub fn run(self) {
        let future = self.0.map_err(self.1);
        tokio::run(future);
    }
}
