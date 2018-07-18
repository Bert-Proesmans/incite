use futures::prelude::*;
use slog;
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
    pub lobby_state: Arc<Mutex<LobbyState>>,
    pub logger: slog::Logger,
    packets_forward: Vec<u32>,
    packets_out: Vec<u32>,
}

impl ClientData {
    fn new(lobby_state: Arc<Mutex<LobbyState>>, logger: slog::Logger) -> Self {
        ClientData {
            lobby_state,
            logger,
            packets_forward: vec![],
            packets_out: vec![],
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
        logger: slog::Logger,
        shared_state: Arc<Mutex<LobbyState>>,
    ) -> Self {
        let services = Default::default();
        let client_data = ClientData::new(shared_state, logger);
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
        // This future must NOT implement a loop!
        // The outer system implements the loop while we simply perform as much
        // work as possible!

        // Async::NotReady, when stream is idle.
        while let Async::Ready(packet_option) = self.codec.poll()? {
            trace!(self.client_data.logger, "Received packet"; "data" => ?packet_option);

            if let Some(packet) = packet_option {
                // TODO; Invoke service based on packet header information.
            } else {
                // None indicates EOF, we exit cleanly.
                info!(self.client_data.logger, "Client closed connection");
                return Ok(Async::Ready(()));
            }
        }

        // TODO; Push pending packets from queue to client.

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
