use bytes::Bytes;
use futures::future;
use futures::prelude::*;
use incite_gen::prost::Message;
use incite_gen::proto::bnet::protocol::connection::ConnectRequest;
use protocol::bnet::frame::BNetPacket;
use service::{
    hash_service_name, Error, ErrorKind, RPCService, Request, Response, Result, MAX_METHODS,
};
use std::u32::MAX;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum ConnectionMethod {
    Connect = 0,
    Bind = 1,
    Echo = 2,
    ForceDisconnect = 3,
    KeepAlive = 4,
    Encrypt = 5,
    RequestDisconnect = 6,
}

#[derive(Debug, Default)]
pub struct ConnectionService {}

impl ConnectionService {
    pub const METHOD_CONNECT: ConnectionMethod = ConnectionMethod::Connect;
    pub const METHOD_BIND: ConnectionMethod = ConnectionMethod::Bind;
    pub const METHOD_ECHO: ConnectionMethod = ConnectionMethod::Echo;
    pub const METHOD_FORCE_DISCONNECT: ConnectionMethod = ConnectionMethod::ForceDisconnect;
    pub const METHOD_KEEP_ALIVE: ConnectionMethod = ConnectionMethod::KeepAlive;
    pub const METHOD_ENCRYPT: ConnectionMethod = ConnectionMethod::Encrypt;
    pub const METHOD_REQUEST_DISCONNECT: ConnectionMethod = ConnectionMethod::RequestDisconnect;

    #[async]
    fn connect_op(packet: Request<BNetPacket>) -> Result<(Request<BNetPacket>, Option<Bytes>)> {
        let packet_body = packet.as_ref().into_inner().body.clone();
        let _request = ConnectRequest::decode(packet_body)?;

        Ok((packet, None))
    }

    #[async]
    pub fn connect_direct(
        packet: Request<BNetPacket>,
    ) -> Result<(Request<BNetPacket>, Option<Bytes>)> {
        Self::validate_request(ConnectionService::METHOD_CONNECT as u32, &packet)?;
        let packet_body = packet.as_ref().into_inner().body.clone();
        let _request = ConnectRequest::decode(packet_body)?;

        Ok((packet, None))
    }

    #[async]
    fn bind_op(_data: Bytes) -> Result<Option<Bytes>> {
        Ok(None)
    }

    #[async]
    fn echo_op(_data: Bytes) -> Result<Option<Bytes>> {
        Ok(None)
    }

    #[async]
    fn force_disconnect_op(_data: Bytes) -> Result<Option<Bytes>> {
        Ok(None)
    }

    #[async]
    fn keep_alive_op(_data: Bytes) -> Result<Option<Bytes>> {
        Ok(None)
    }

    #[async]
    fn encrypt_op(_data: Bytes) -> Result<Option<Bytes>> {
        Ok(None)
    }

    #[async]
    fn request_disconnect_op(_data: Bytes) -> Result<Option<Bytes>> {
        Ok(None)
    }
}

impl RPCService for ConnectionService {
    type Method = ConnectionMethod;
    type Packet = BNetPacket;
    type Future = Box<dyn Future<Item = Option<Response<Self::Packet>>, Error = Error>>;
    // type FutureRevisit = Box<Future<Item = (), Error = Error>>;

    fn get_id() -> u32 {
        super::ServiceIDs::ConnectionService as u32
    }

    fn get_name() -> &'static str {
        "bnet.protocol.connection.ConnectionService"
    }

    fn get_hash() -> u32 {
        hash_service_name(Self::get_name())
    }

    fn get_methods() -> [(&'static str, u32); MAX_METHODS] {
        [
            ("Connect", ConnectionMethod::Connect as u32),
            ("Bind", ConnectionMethod::Bind as u32),
            ("Echo", ConnectionMethod::Echo as u32),
            ("ForceDisconnect", ConnectionMethod::ForceDisconnect as u32),
            ("KeepAlive", ConnectionMethod::KeepAlive as u32),
            ("Encrypt", ConnectionMethod::Encrypt as u32),
            (
                "RequestDisconnect",
                ConnectionMethod::RequestDisconnect as u32,
            ),
            ("", MAX),
            ("", MAX),
            ("", MAX),
        ]
    }

    fn validate_request(request_method_id: u32, packet: &Request<Self::Packet>) -> Result<()> {
        let ref header = packet.as_ref().into_inner().header;
        let method_id = header.method_id.ok_or(ErrorKind::UnknownRequest(Self::get_name()))?;
        if request_method_id != method_id {
            Err(ErrorKind::InvalidRequest(method_id, Self::get_name()))?;
        }

        let is_known_method = Self::get_methods()
                .iter()
                .map(|(_, m_id)| *m_id)
                .any(move |m| m < (MAX_METHODS as u32) && m == request_method_id);
        if ! is_known_method {
            Err(ErrorKind::InvalidRequest(method_id, Self::get_name()))?;
        }

        Ok(())
    }

    fn call(&mut self, method: Self::Method, packet: Request<BNetPacket>) -> Self::Future {
        if let Err(e) = Self::validate_request(method as u32, &packet) {
            return Box::new(future::err(e));
        }

        let response_mapper = |resp| {
            let (request, response): (Request<BNetPacket>, Option<Bytes>) = resp;
            response.map(|bytes| request.into_response(bytes))
        };

        match method {
            ConnectionMethod::Connect => {
                let response = Self::connect_op(packet).map(response_mapper);
                Box::new(response)
            }
            // TODO; Remove catch all.
            // _ => unreachable!(),
            _ => {
                Box::new(future::err(ErrorKind::InvalidRequest(0, Self::get_name()).into()))
            },
        }
    }

    /*
    fn revisit(&mut self, packet: Response<BNetPacket>) -> Self::FutureRevisit {
        unimplemented!()
    }
    */
}
