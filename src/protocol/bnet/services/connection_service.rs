
use bytes::Bytes;
use futures::prelude::*;
use protocol::bnet::service_util::{RPCService, hash_service_name, RPCError};
use incite_gen::prost::Message;
use incite_gen::proto::bnet::protocol::connection::ConnectRequest;

#[derive(Debug)]
pub struct ConnectionService {}

impl ConnectionService {
	#[async]
    fn connect(&self, data: Bytes) -> Result<Option<Bytes>, RPCError> {
    	let _request = ConnectRequest::decode(data)?;
    	Ok(None)
    }

    #[async]
    fn bind(&mut self, data: Bytes) -> Result<Option<Bytes>, RPCError> {
    	Ok(None)
    }

    #[async]
    fn echo(&mut self, data: Bytes) -> Result<Option<Bytes>, RPCError> {
    	Ok(None)
    }

    #[async]
    fn force_disconnect(&mut self, data: Bytes) -> Result<Option<Bytes>, RPCError> {
    	Ok(None)
    }

    #[async]
    fn keep_alive(&mut self, data: Bytes) -> Result<Option<Bytes>, RPCError> {
    	Ok(None)
    }

    #[async]
    fn encrypt(&mut self, data: Bytes) -> Result<Option<Bytes>, RPCError> {
    	Ok(None)
    }

    #[async]
    fn request_disconnect(&mut self, data: Bytes) -> Result<Option<Bytes>, RPCError> {
    	Ok(None)
    }
}

impl RPCService for ConnectionService {
    fn get_name() -> &'static str {
    	"bnet.protocol.connection.ConnectionService"
    }

    fn get_hash() -> u32 {
    	hash_service_name(Self::get_name())
    }

    fn get_methods() -> [&'static str; 10] {
    	["Connect", "Bind", "Echo", "ForceDisconnect", "KeepAlive", "Encrypt", "RequestDisconnect", "", "", ""]
    }

    /*
    fn invoke(&mut self, method: u32, data: Bytes) -> impl Future<Item=Option<Bytes>, Error=RPCError> {
    	match method {
    		0 => self.connect(data),
    		_ => panic!("TODO"),
    	}
    }
    */
}
