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

use protocol::bnet::frame::BNetCodec;
use protocol::bnet::session::light::LightWeightSession;
use protocol::bnet::session::{Error, ErrorKind, Result};
use servers::lobby::LobbyState;
use services::bnet::ConnectionService;

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
    let handshake = Deadline::new(handshake, handshake_deadline);
    let handshake = handshake
        .map_err(|deadline_err| match deadline_err.into_inner() {
            Some(setup_error) => setup_error,
            _ => Error::from_kind(ErrorKind::Timeout),
        })
        /*
        // Only look at the error without touching the future-chain.
        .inspect_err(move |error| {
            warn!(logger_handshake, "Handshake failed"; "error" => ?error);
        })
        */
        // Todo: remove this map_err in favor of inspect_err (above). The latter is currently not
        // supported within futures-0.1 by mistake.
        .map_err(move |error| {
            warn!(logger_handshake, "Handshake failed"; "error" => ?error);
            error
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
    let (session, connect_request) = await!(session.read_request())?;
    {
        let ref request_header = connect_request.as_ref().into_inner().header;
        trace!(session.logger, "Handshake initiated"; "header" => ?request_header);
    }

    let (session, connect_request, connect_response) =
        await!(ConnectionService::connect_direct(session, connect_request))?;
    // Mutably rebind session so the response logic can overwrite it.
    let mut session = session;
    if let Some(response_data) = connect_response {
        let response_packet = connect_request.into_response(response_data);
        session = await!(session.send_response(response_packet))?;
    }

    trace!(session.logger, "Handshake completed");

    // DEBUG: Read the next request from the stream
    let (session, request) = await!(session.read_request())?;
    trace!(session.logger, "Next request received"; "packet" => ?request);

    Ok(session)
}
