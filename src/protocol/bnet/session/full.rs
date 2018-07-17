use futures::prelude::*;
// use slog;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio_codec::Framed;
use tokio_tcp::TcpStream;

use protocol::bnet::frame::BNetCodec;
use protocol::bnet::session::Error;
use servers::lobby::LobbyState;
use services::bnet::ConnectionService;

#[derive(Default)]
pub struct Services {
    connect: ConnectionService,
}

pub struct ClientData {
    lobby_state: Arc<Mutex<LobbyState>>,
    // logger: slog::Logger,
}

impl ClientData {
    fn new(lobby_state: Arc<Mutex<LobbyState>> /*, logger: slog::Logger*/) -> Self {
        ClientData {
            lobby_state,
            // logger,
        }
    }
}

pub struct LobbyClientSession {
    address: SocketAddr,
    codec: Framed<TcpStream, BNetCodec>,
    services: Services,
    client_data: ClientData,
}

impl LobbyClientSession {
    pub fn new(
        address: SocketAddr,
        codec: Framed<TcpStream, BNetCodec>,
        // logger: slog::Logger,
        shared_state: Arc<Mutex<LobbyState>>,
    ) -> Self {
        // Services will be initialised after handshaking because they might allocate a lot of resources.
        let services = Default::default();
        let client_data = ClientData::new(shared_state /* logger */);
        LobbyClientSession {
            address,
            codec,
            services,
            client_data,
        }
    }
}

impl Future for LobbyClientSession {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        unimplemented!()
    }
}
