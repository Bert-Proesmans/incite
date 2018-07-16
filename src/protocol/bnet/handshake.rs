use futures::prelude::await;
use futures::prelude::*;
use slog;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio;
use tokio_codec::Decoder;
use tokio_tcp::TcpStream;
use tokio_timer::Deadline;

use incite_gen::prost::Message;
use incite_gen::proto::bnet::protocol::connection::ConnectRequest;
use protocol::bnet::frame::BNetCodec;
use protocol::bnet::session::{Error, ErrorKind, LightWeightSession, Result};
use servers::lobby::LobbyState;

const SESSION_SETUP_DEADLINE_SECS: u64 = 5;

struct OwnedValueWrapper<T: fmt::Debug + Send + Sync>(pub T);
impl<T: fmt::Debug + Send + Sync> slog::Value for OwnedValueWrapper<T> {
    fn serialize(
        &self,
        _record: &slog::Record,
        key: slog::Key,
        serializer: &mut slog::Serializer,
    ) -> slog::Result {
        serializer.emit_str(key, &format!("{:?}", self.0))
    }
}

pub fn handle_client(
    client: TcpStream,
    shared_state: Arc<Mutex<LobbyState>>,
    root_logger: slog::Logger,
) -> Result<()> {
    let address = client.peer_addr()?;
    // Creates a new child logger for assignment to the session.
    let child_logger = root_logger.new(o!("connection" => OwnedValueWrapper(address)));
    let (logger_handshake, logger_op) = (child_logger.clone(), child_logger.clone());
    let codec = BNetCodec::new().framed(client);
    let session = LightWeightSession::build(address, codec, child_logger);

    trace!(logger_op, "Attempting client handshake"; "address" => ?session.address);

    let handshake = handshake_internal(session);
    // Wrap the handshake with a deadline. The deadline allows us to time-out the connection
    // and cleanup the resources allocated to accomodate a new client connection.
    let handshake_deadline = Instant::now() + Duration::from_secs(SESSION_SETUP_DEADLINE_SECS);
    let handshake = Deadline::new(handshake, handshake_deadline)
        .map_err(|deadline_err| match deadline_err.into_inner() {
            Some(setup_error) => setup_error,
            _ => Error::from_kind(ErrorKind::Timeout),
        })
        // Only look at the error without touching the future-chain.
        .inspect_err(move |error| {
            warn!(logger_handshake, "Handshake failed"; "error" => ?error);
        })
        .and_then(move |session| LightWeightSession::build_session(session, shared_state))
        .map_err(move |error| {
            error!(logger_op, "Client handling returned an error"; "error" => ?error);
        });
    // tokio:spawn() requires a future which returns '()' as Item AND Error.
    tokio::spawn(handshake);
    Ok(())
}

#[async]
fn handshake_internal(session: LightWeightSession) -> Result<LightWeightSession> {
    let LightWeightSession {
        codec,
        address,
        // logger,
    } = session;
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
    // trace!(logger, "ConnectRequest"; "data" => ?request);

    // TODO
    // trace!(logger, "Handshake finished");

    Ok(LightWeightSession {
        codec,
        address,
        // logger,
    })
}
