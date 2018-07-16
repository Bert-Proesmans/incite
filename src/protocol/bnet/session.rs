use futures::prelude::*;
use slog;
use std;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio_codec::Framed;
use tokio_tcp::TcpStream;

use protocol::bnet::frame::BNetCodec;
use servers::lobby::LobbyState;
use services::bnet::ConnectionService;

pub mod error {
    use protocol::bnet::frame;
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
            Framing(frame::error::Error, frame::error::ErrorKind);
        }

        foreign_links {
            Io(io::Error) #[doc = "Error during IO"];
        }
    }
}

pub use self::error::*;

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

pub struct LightWeightSession {
    pub address: SocketAddr,
    pub codec: Framed<TcpStream, BNetCodec>,
    // pub logger: slog::Logger,
}

impl LightWeightSession
where
    slog::Logger: Send + Sync + std::panic::UnwindSafe,
{
    pub fn build(
        address: SocketAddr,
        codec: Framed<TcpStream, BNetCodec>,
        _logger: slog::Logger,
    ) -> Self {
        LightWeightSession {
            address,
            codec,
            //  logger,
        }
    }
}

impl LightWeightSession {
    pub fn build_session(self, shared_state: Arc<Mutex<LobbyState>>) -> LobbyClientSession {
        let LightWeightSession {
            address,
            codec,
            // logger,
        } = self;
        LobbyClientSession::new(address, codec, /* logger, */ shared_state)
    }
}

pub struct LobbyClientSession {
    address: SocketAddr,
    codec: Framed<TcpStream, BNetCodec>,
    services: Services,
    client_data: ClientData,
}

impl LobbyClientSession {
    fn new(
        address: SocketAddr,
        codec: Framed<TcpStream, BNetCodec>,
        // logger: slog::Logger,
        shared_state: Arc<Mutex<LobbyState>>,
    ) -> Self {
        // Services will be initialised after handshaking because they might allocate a lot of resources.
        let services = Default::default();
        let client_data = ClientData::new(shared_state, /* logger */);
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
