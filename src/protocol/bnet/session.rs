use std;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio;
use tokio::io::AsyncRead;
use tokio::net::TcpStream;
use futures::prelude::*;
use slog;

use setup::SharedLobbyState;
use protocol::frame::BNetCodec;


mod error {
    use std::io;

    error_chain!{
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
    fn new(addr: SocketAddr, logger: slog::Logger, shared_state: Arc<Mutex<SharedLobbyState>>, codec: BNetCodec) -> Self {
    	unimplemented!()
    }
}

#[async]
pub fn entry(client: TcpStream, shared_state: Arc<Mutex<SharedLobbyState>>) -> std::result::Result<(), ()> {
	// let packet_stream = client.framed(BNetCodec::new());
	println!("Started client");

	Ok(())
}
