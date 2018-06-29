use futures::prelude::await;
use futures::prelude::*;
use tokio::net::TcpListener;
use tokio_signal;

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

pub struct LobbyState {}

#[async]
pub fn setup_lobby_server(control: ServerControl) -> Result<(TcpListener, LobbyState)> {
    let server = TcpListener::bind(&control.bind_address)?;
    let shared_state = LobbyState {};
    Ok((server, shared_state))
}

#[async]
pub fn lobby_handle_connections(server: TcpListener, shared_state: LobbyState) -> Result<()> {
    let ctrl_c = await!(tokio_signal::ctrl_c())?
        .map(|_| (None, Some(Signal::CtrlC)))
        .map_err(Into::<Error>::into);
    let client_stream = server
        .incoming()
        .map(|client| (Some(client), None))
        .map_err(Into::<Error>::into);

    let combo_stream = client_stream.select(ctrl_c);

    #[async]
    for select_result in combo_stream {
        match select_result {
            (Some(client), None) => {
                // TODO; handle client
            }
            (_, Some(signal)) => break,
            (None, None) => unreachable!(),
        }
    }

    Ok(())
}
