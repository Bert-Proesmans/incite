use failure;
use futures::prelude::await;
use futures::prelude::*;
use slog;
use std::net::SocketAddr;
use tokio;
use tokio_tcp::TcpListener;

use super::{log, servers};

mod error {
    use super::*;
    use std::io;

    error_chain! {
        errors {
            UserCallback(err: failure::Error) {
                description("A user supplied callback failed")
                display("The supplied callback returned an error.\n{}", err)
            }
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
    ) -> ServerHandle<impl Future<Item = (), Error = ::Error>, impl FnOnce(::Error) -> ()>
    where
        F: Future<Item = (), Error = E> + Send + 'static,
        E: failure::Fail,
    {
        let ServerConfig {
            application_logger,
            bind_address,
            ..
        } = self;

        let logger_err = application_logger.clone();
        let logger_build = application_logger.clone();

        let default_error_handler =
            move |err| crit!(logger_err, "Accepting clients failed!"; "error" => %err);

        let server_build = async_block!{
            let server = TcpListener::bind(&bind_address)?;
            info!(logger_build, "Server bound"; "endpoint" => ?bind_address);
            let shared_state = servers::lobby::LobbyState::default();

            await!(on_setup_handler.map_err(|e| {
                let erased_error = failure::Error::from(e);
                ErrorKind::UserCallback(erased_error)
            }))?;
            trace!(logger_build, "Setup complete");
            Ok::<_, Error>((server, shared_state))
        };

        let server_future = server_build
            // Transform the error
            .map_err(Into::into)
            .and_then(move |(server, state)| {
                let logger = application_logger;
                servers::lobby::handle_connections(server, state, logger)
                .map_err(Into::into)
            })
            .map(|_| ());

        ServerHandle::new(server_future, default_error_handler)
    }
}

#[must_use = "The server is not started until run is called on this structure."]
pub struct ServerHandle<F, E>(F, E)
where
    F: Future<Item = ()> + Send + 'static,
    E: FnOnce(<F as Future>::Error) -> () + Send + 'static;

impl<F, E> ServerHandle<F, E>
where
    F: Future<Item = ()> + Send + 'static,
    E: FnOnce(<F as Future>::Error) -> () + Send + 'static,
{
    fn new(setup: F, error_handler: E) -> Self {
        ServerHandle(setup, error_handler)
    }

    pub fn on_error<NewErr>(self, error_handler: NewErr) -> ServerHandle<F, NewErr>
    where
        NewErr: FnOnce(F::Error) -> () + Send + 'static,
    {
        ServerHandle(self.0, error_handler)
    }

    pub fn run(self) {
        let future = self.0.map_err(self.1);
        tokio::run(future);
    }
}
