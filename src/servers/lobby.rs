use futures::prelude::await;
use futures::prelude::*;
use slog;
use std::default::Default;
use std::sync::{Arc, Mutex};
use tokio_signal;
use tokio_tcp::TcpListener;
use Signal;

pub mod error {
    use std::io;

    error_chain!{
        errors {
            StatePoisoning {
                description("Some thread holding the lock panicked, resulting in an invalid state")
                display("The shared server state has been poisoned")
            }
        }

        foreign_links {
            Io(io::Error) #[doc = "Error during IO"];
        }
    }
}
pub use self::error::*;

pub struct LobbyState {
    _last_signal: Signal,
}

impl Default for LobbyState {
    fn default() -> Self {
        LobbyState {
            _last_signal: Signal::None,
        }
    }
}

impl LobbyState {}

#[async]
pub fn handle_connections(
    server: TcpListener,
    shared_state: LobbyState,
    logger: slog::Logger,
) -> Result<()> {
    use protocol::bnet::handshake::handle_client;

    let shared_state = Arc::new(Mutex::new(shared_state));
    let ctrl_c = await!(tokio_signal::ctrl_c())?
        .map(|_| (None, Some(Signal::TermQuit)))
        .map_err(Into::<Error>::into);
    let client_stream = server
        .incoming()
        .map(|client| (Some(client), None))
        .map_err(Into::<Error>::into);

    // This will keep looping forever until a received terminal signal breaks the loop.
    #[async]
    for select_result in client_stream.select(ctrl_c) {
        match select_result {
            (Some(client), None) => {
                let client_result = handle_client(client, shared_state.clone(), logger.clone());
                match client_result {
                    Err(error) => {
                        error!(logger, "Handling new client failed"; "error" => ?error);
                    }
                    _ => {}
                }
            }
            (_, Some(_signal)) => break,
            (None, None) => unreachable!(),
        };
    }

    info!(logger, "Finished accept loop");

    // NOTE: This future, which keeps accepting clients, finishes here.
    // The program itself will only stop running after all spawned client futures finished.
    Ok(())
}
