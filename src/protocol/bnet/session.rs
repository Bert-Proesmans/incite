use futures::prelude::await;
use futures::prelude::*;
use incite_gen::prost::Message;
use slog;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio;
use tokio_codec::{Decoder, Framed};
use tokio_tcp::TcpStream;
use tokio_timer::Deadline;

use incite_gen::proto::bnet::protocol::connection::ConnectRequest;
use protocol::frame::BNetCodec;
use servers::lobby::SharedLobbyState;

const SESSION_SETUP_DEADLINE_SECS: u64 = 5;

mod error {
    use protocol::frame;
    use std::io;

    error_chain!{
        errors {
            ClientDisconnected {
                description("The client hung-up on us")
                display("The client disconnected unexpectedly")
            }
            ProcedureFail(step: &'static str) {
                description("The client failed to send the proper response")
                display("The client didn't respond correctly during '{}'", step)
            }
            Timeout {
                description("The code triggered a timeout error to prevent rogue clients")
                display("The client failed to respond within the time limit")
            }
            StatePoisoning {
                description("Some thread holding the lock panicked, resulting in an invalid state")
                display("The shared server state has been poisoned")
            }
        }

        links {
            Framing(frame::Error, frame::ErrorKind);
        }

        foreign_links {
            Io(io::Error) #[doc = "Error during IO"];
        }
    }
}

pub use self::error::*;

pub struct ClientSession {
    logger: slog::Logger,
    shared_state: Arc<Mutex<SharedLobbyState>>,
    addr: SocketAddr,
    codec: BNetCodec,
}

impl ClientSession {
    fn new(
        _shared_state: Arc<Mutex<SharedLobbyState>>,
        _addr: SocketAddr,
        _logger: slog::Logger,
    ) -> Self {
        unimplemented!()
    }
}

pub fn entry(client: TcpStream, shared_state: Arc<Mutex<SharedLobbyState>>) -> Result<()> {
    let client_address = client.peer_addr()?;
    let codec = BNetCodec::new().framed(client);

    // Errors can be ignored because the handshake_setup procedure is responsible for
    // allocation+cleanup of resources.
    // Optionally a chain could be made with or_else() to notify some other system
    // about handshake failure.
    let client_handshake =
        handshake_setup(client_address, codec, shared_state.clone()).map_err(move |error| {
            // TODO; Remove unwrap
            let state_guard = shared_state.lock().unwrap();
            let logger = state_guard.logger();
            trace!(logger, "Handshake failed"; "error" => ?error);
        });
    tokio::spawn(client_handshake);
    Ok(())
}

#[async]
fn handshake_setup(
    addr: SocketAddr,
    codec: Framed<TcpStream, BNetCodec>,
    shared_state: Arc<Mutex<SharedLobbyState>>,
) -> Result<()> {
    {
        let state_guard = shared_state.lock().map_err(|_| ErrorKind::StatePoisoning)?;
        let logger = state_guard.logger();
        trace!(logger, "Attempting client handshake"; "address" => ?addr);
    }

    let handshake = handshake_internal(addr.clone(), codec, shared_state.clone());
    let timed_handshake = Deadline::new(
        handshake,
        Instant::now() + Duration::from_secs(SESSION_SETUP_DEADLINE_SECS),
    );
    let handshake = timed_handshake.map_err(|deadline_err| match deadline_err.into_inner() {
        Some(setup_error) => setup_error,
        _ => Error::from_kind(ErrorKind::Timeout),
    });
    let _codec = await!(handshake)?;

    Ok(())
}

#[async]
fn handshake_internal(
    _addr: SocketAddr,
    codec: Framed<TcpStream, BNetCodec>,
    shared_state: Arc<Mutex<SharedLobbyState>>,
) -> Result<Framed<TcpStream, BNetCodec>> {
    let (rpc_connect, codec) = match await!(codec.into_future()) {
        Ok((Some(frame), stream)) => (frame, stream),
        Ok((None, _)) => Err(ErrorKind::ClientDisconnected)?,
        Err((err, _)) => Err(err)?,
    };

    match (rpc_connect.header.service_id, rpc_connect.header.method_id) {
        (s_id, Some(m_id)) if s_id == 0 && m_id == 1 => {}
        _ => Err(ErrorKind::ProcedureFail("connect_request"))?,
    };

    let request = ConnectRequest::decode(&rpc_connect.body)
        .map_err(|_| ErrorKind::ProcedureFail("connect_request"))?;
    {
        let state_guard = shared_state.lock().map_err(|_| ErrorKind::StatePoisoning)?;
        let logger = state_guard.logger();
        trace!(logger, "ConnectRequest"; "data" => ?request);
    }

    {
        let state_guard = shared_state.lock().map_err(|_| ErrorKind::StatePoisoning)?;
        let logger = state_guard.logger();
        trace!(logger, "Handshake finished");
    }
    Ok(codec)
}
