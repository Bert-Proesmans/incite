use futures::prelude::*;
use slog;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio_codec::Framed;
use tokio_tcp::TcpStream;

use protocol::bnet::frame::BNetCodec;
use protocol::bnet::session::router::BNetRouter;
use protocol::bnet::session::Error;
use servers::lobby::LobbyState;
use services::bnet::ConnectionService;

pub struct ClientData {
    pub lobby_state: Arc<Mutex<LobbyState>>,
    pub logger: slog::Logger,
}

impl ClientData {
    fn new(lobby_state: Arc<Mutex<LobbyState>>, logger: slog::Logger) -> Self {
        ClientData {
            lobby_state,
            logger,
        }
    }
}

pub struct LobbyClientSession {
    address: SocketAddr,
    codec: Framed<TcpStream, BNetCodec>,
    router: Option<BNetRouter<ClientData>>,
}

impl LobbyClientSession {
    pub fn new(
        address: SocketAddr,
        codec: Framed<TcpStream, BNetCodec>,
        logger: slog::Logger,
        shared_state: Arc<Mutex<LobbyState>>,
    ) -> Self {
        let client_hash = Self::hash_address(&address);
        let services = Default::default();
        let client_data = ClientData::new(shared_state, logger);
        let router = Some(BNetRouter::new(client_hash, services, client_data));
        LobbyClientSession {
            address,
            codec,
            router,
        }
    }

    fn hash_address(address: &SocketAddr) -> u64 {
        let mut hasher = DefaultHasher::new();
        address.hash(&mut hasher);
        hasher.finish()
    }
}

impl Future for LobbyClientSession {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        // This future must NOT implement a loop!
        // The outer system implements the loop while we simply perform as much
        // work as possible!

        // Extract router, because we're going to consume it later.
        // Do not forget to reinstall the router before returning!
        let mut router = self.router.take().unwrap();

        // Async::NotReady, when stream is idle.
        while let Async::Ready(packet_option) = self.codec.poll()? {
            trace!(router.data.logger, "Received packet"; "data" => ?packet_option);

            if let Some(packet) = packet_option {
                // TODO; Invoke service based on packet header information.
            } else {
                // None indicates EOF, we exit cleanly.
                info!(router.data.logger, "Client closed connection");
                return Ok(Async::Ready(()));
            }
        }

        // TODO; Push pending packets from queue to client.

        // Re-install router
        self.router = Some(router);

        // Let the caller know that we still want to read additionall data.
        //
        // NEVER RETURN Async::NotReady without ensuring an inner future has returned
        // `NotReady`!
        // This is because the eventloop will check the semaphore attached to the current
        // task (=cohesion of this and inner futures). When WE or an inner future are NOT
        // increasing the semaphore count before returning `NotReady`, we'll be blackholed!
        //
        // This can be done manually by calling `task::current().notify()`, but that defeats
        // the purpose. We should make sure inner futures have some signal handler attached
        // which notifies the eventloop for us.
        //
        Ok(Async::NotReady)
    }
}
