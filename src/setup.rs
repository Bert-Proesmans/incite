use futures::prelude::await;
use futures::prelude::*;
use slog;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio_signal;
use tokio;

use super::protocol;
use super::{ServerControl, Signal};

mod error {
    use std::io;

    error_chain!{
        foreign_links {
            Io(io::Error) #[doc = "Error during IO"];
        }
    }
}

pub use self::error::*;

pub struct SharedLobbyState {
    system_logger: slog::Logger,
    application_logger: slog::Logger,
    last_signal: Signal,
}

#[async]
pub fn setup_lobby_server(control: ServerControl) -> Result<(TcpListener, SharedLobbyState)> {
    let server = TcpListener::bind(&control.bind_address)?;
    let shared_state = SharedLobbyState {
        system_logger: control.system_logger,
        application_logger: control.application_logger,
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

    #[async]
    for select_result in combo_stream {
        match select_result {
            (Some(client), None) => {
                tokio::spawn(protocol::bnet::session::entry(client, shared_state.clone()));
            }
            (_, Some(signal)) => break,
            (None, None) => unreachable!(),
        }
    }

    Ok(())
}
