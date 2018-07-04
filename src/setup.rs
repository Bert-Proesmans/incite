use futures::prelude::await;
use futures::prelude::*;
use incite_gen::version;
use slog;
use std::net::SocketAddr;
use std::panic::UnwindSafe;
use std::sync::{Arc, Mutex};
use tokio;
use tokio::net::TcpListener;
use tokio_signal;

use super::protocol;
use super::{ServerControl, Signal};

mod error {
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

pub struct SharedLobbyState {
    logger: slog::Logger,
    last_signal: Signal,
}

impl SharedLobbyState {
    pub fn logger(&self) -> &slog::Logger {
        &self.logger
    }
}

#[async]
pub fn setup_lobby_server(control: ServerControl) -> Result<(TcpListener, SharedLobbyState)> {
    let ServerControl {
        application_logger,
        bind_address,
        ..
    } = control;

    let server = TcpListener::bind(&bind_address)?;
    info!(application_logger, "Server bound"; "endpoint" => ?bind_address);

    let shared_state = SharedLobbyState {
        logger: application_logger,
        last_signal: Signal::None,
    };
    Ok((server, shared_state))
}

#[async]
pub fn lobby_handle_connections(server: TcpListener, shared_state: SharedLobbyState) -> Result<()> {
    let ctrl_c = await!(tokio_signal::ctrl_c())?
        .map(|_| (None, Some(Signal::CtrlC)))
        .map_err(Into::<Error>::into);
    let client_stream = server
        .incoming()
        .map(|client| (Some(client), None))
        .map_err(Into::<Error>::into);

    let combo_stream = client_stream.select(ctrl_c);
    let shared_state = Arc::new(Mutex::new(shared_state));

    // This will keep looping forever until one of the futures transitions into the error state.
    // Alternatively a received signal breaks the loop cleanly.
    #[async]
    for select_result in combo_stream {
        match select_result {
            (Some(client), None) => {
                let client_address = client.peer_addr().unwrap();
                trace!(shared_state.lock().unwrap().logger(), "Client accepted"; "address" => ?client_address);
                let entry_result =
                    protocol::bnet::session::entry(client, shared_state.clone());
                match entry_result {
                    Err(error) => error!(
                        shared_state.lock().map_err(|_| ErrorKind::StatePoisoning)?.logger(),
                        "Entry failed for client"; "error" => ?error
                    ),
                    _ => {}
                }
            }
            (_, Some(_signal)) => break,
            (None, None) => unreachable!(),
        };
    }

    info!(
        shared_state.lock().unwrap().logger(),
        "Finished accept loop"
    );

    // NOTE: This future, representing acceptance of new clients, finishes here.
    // The program itself will only stop running after all spawned client futures finished.
    Ok(())
}
