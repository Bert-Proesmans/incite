use bytes::BytesMut;
use futures::prelude::await;
use futures::prelude::*;
use slog;
use std::net::SocketAddr;
use std::panic;
use std::sync::{Arc, Mutex};
use tokio_codec::Framed;
use tokio_tcp::TcpStream;

use protocol::bnet::frame::{BNetCodec, BNetPacket};
use protocol::bnet::session::full::LobbyClientSession;
use protocol::bnet::session::{ErrorKind, Result};
use servers::lobby::LobbyState;
use service::{Request, Response};

pub struct LightWeightSession {
    pub address: SocketAddr,
    pub codec: Option<Framed<TcpStream, BNetCodec>>,
    pub logger: slog::Logger,
    buffer: BytesMut,
}

impl LightWeightSession
where
    slog::Logger: Send + Sync + panic::UnwindSafe,
{
    pub fn build(
        address: SocketAddr,
        codec: Framed<TcpStream, BNetCodec>,
        logger: slog::Logger,
    ) -> Self {
        let buffer = BytesMut::new();
        let codec = Some(codec);
        LightWeightSession {
            address,
            codec,
            logger,
            buffer,
        }
    }

    #[async]
    pub fn read_request(mut self) -> Result<(Self, Request<BNetPacket>)> {
        let codec = self.codec.take().unwrap();
        let (packet, codec) = match await!(codec.into_future()) {
            Ok((Some(frame), stream)) => (frame, stream),
            Ok((None, _)) => Err(ErrorKind::ClientDisconnected)?,
            Err((err, _)) => Err(err)?,
        };

        let packet = Request::from_bnet(packet)?;
        self.codec = Some(codec);
        Ok((self, packet))
    }

    #[async]
    pub fn send_response(mut self, response: Response<BNetPacket>) -> Result<Self> {
        let response = response.into_inner();
        // Codec must be taken from self because the future operation consumes it.
        let sink = self.codec.take().unwrap();
        self.codec = Some(await!(sink.send(response))?);
        Ok(self)
    }
}

impl LightWeightSession {
    pub fn build_session(self, shared_state: Arc<Mutex<LobbyState>>) -> LobbyClientSession {
        let LightWeightSession {
            address,
            codec,
            // logger,
            ..
        } = self;
        LobbyClientSession::new(address, codec.unwrap(), /* logger, */ shared_state)
    }
}
