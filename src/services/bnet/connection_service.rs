use bytes::{Bytes, BytesMut};
use futures::future;
use futures::prelude::*;
use incite_gen::prost::Message;
use protocol::bnet::frame::BNetPacket;
use protocol::bnet::session::light::LightWeightSession;
use rpc::system::{RPCResponder, RPCResult, RPCService, Request, MAX_METHODS};
use rpc::util::hash_service_name;
use rpc::{Error, ErrorKind, Result};
use services::bnet::service_info::{ExportedServiceID, SERVICES_EXPORTED, SERVICES_IMPORTED};
use std::default::Default;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum ConnectionMethod {
    Connect = 1,
    Bind = 2,
    Echo = 3,
    ForceDisconnect = 4,
    KeepAlive = 5,
    Encrypt = 6,
    RequestDisconnect = 7,
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
    fn connect_op(payload: Bytes) -> Result<RPCResult<BNetPacket>> {
        unimplemented!()
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
    type Incoming = BNetPacket;
    type Outgoing = BNetPacket;

    fn get_id() -> u32 {
        ExportedServiceID::ConnectionService as u32
    }

    fn get_name() -> &'static str {
        "bnet.protocol.connection.ConnectionService"
    }

    fn get_hash() -> u32 {
        hash_service_name(Self::get_name())
    }

    fn get_methods() -> [Option<(&'static str, &'static Self::Method)>; MAX_METHODS] {
        [
            Some(("Connect", &Self::METHOD_CONNECT)),
            Some(("Bind", &Self::METHOD_BIND)),
            Some(("Echo", &Self::METHOD_ECHO)),
            Some(("ForceDisconnect", &Self::METHOD_FORCE_DISCONNECT)),
            Some(("KeepAlive", &Self::METHOD_KEEP_ALIVE)),
            Some(("Encrypt", &Self::METHOD_ENCRYPT)),
            Some(("RequestDisconnect", &Self::METHOD_REQUEST_DISCONNECT)),
            None,
            None,
            None,
        ]
    }
}

impl RPCResponder for ConnectionService {
    type Future = Box<dyn Future<Item = RPCResult<Self::Outgoing>, Error = Error>>;

    fn validate_request(packet: &Request<Self::Incoming>) -> Result<Self::Method> {
        let ref header = packet.as_ref().into_inner().header;
        let method = header
            .method_id
            .ok_or(ErrorKind::UnknownRequest(Self::get_name()))?;

        let known_methods = Self::get_methods();
        let mut matched_method = known_methods
            .iter()
            .filter_map(move |x| {
                if let Some((_, &m_id)) = x {
                    if (m_id as u32) == method {
                        return Some(m_id);
                    }
                }
                None
            })
            .take(1);

        if let Some(found_method) = matched_method.next() {
            return Ok(found_method);
        }

        Err(ErrorKind::InvalidRequest(method, Self::get_name()).into())
    }

    fn call(&mut self, method: Self::Method, payload: Bytes) -> Self::Future {
        match method {
            ConnectionMethod::Connect => {
                let response = Self::connect_op(payload);
                Box::new(response)
            }
            // TODO; Remove catch all.
            // _ => unreachable!(),
            _ => Box::new(future::err(
                ErrorKind::InvalidRequest(0, Self::get_name()).into(),
            )),
        }
    }
}

// Separate impl block to not introduce one-of noise into our service.
impl ConnectionService {
    fn is_connect_request(packet: &Request<BNetPacket>) -> Result<()> {
        let ref header = packet.as_ref().into_inner().header;
        let method = header
            .method_id
            .ok_or(ErrorKind::UnknownRequest(Self::get_name()))?;

        if method == (ConnectionMethod::Connect as u32) {
            return Ok(());
        }

        Err(ErrorKind::InvalidRequest(method, Self::get_name()).into())
    }

    #[async]
    pub fn connect_direct(
        session: LightWeightSession,
        packet: Request<BNetPacket>,
    ) -> Result<(LightWeightSession, Request<BNetPacket>, Option<Bytes>)> {
        use chrono::Local;
        use incite_gen::proto::bnet::protocol::connection::{
            BindRequest, BindResponse, ConnectRequest, ConnectResponse,
        };
        use incite_gen::proto::bnet::protocol::ProcessId;
        Self::is_connect_request(&packet)?;

        let packet_body = packet.as_ref().into_inner().body.clone();
        let request = ConnectRequest::decode(packet_body)?;
        trace!(session.logger, "Handshake"; "message" => ?request);

        let bind_request = request.bind_request;
        if bind_request.is_none() {
            Err(ErrorKind::InvalidRequest(
                ConnectionService::METHOD_CONNECT as u32,
                Self::get_name(),
            ))?;
        }

        let BindRequest {
            imported_service_hash: exported_services,
            exported_service: imported_services,
        } = bind_request.unwrap();
        // All imported service IDs must match our service info.
        let match_imported_service = imported_services.into_iter().all(|s| {
            if let Some(id) = SERVICES_IMPORTED.get(&s.hash).map(|m| (*m) as u32) {
                if id == s.id {
                    return true;
                }
            }
            false
        });
        if !match_imported_service {
            Err(ErrorKind::InvalidRequest(
                ConnectionService::METHOD_CONNECT as u32,
                Self::get_name(),
            ))?;
        }
        // We can build our own mapping for exported services according to the service info.
        let service_bindings: Vec<u32> = exported_services
            .into_iter()
            .map(|hash| {
                SERVICES_EXPORTED
                    .get(&hash)
                    .map(|m| (*m) as u32)
                    .unwrap_or(0)
            })
            .collect();
        let bind_response = BindResponse {
            imported_service_id: service_bindings,
        };
        let time = Local::now().timestamp();
        let precise_time = Local::now().timestamp_nanos();
        let response_msg = ConnectResponse {
            server_id: ProcessId {
                label: 3868510373,
                epoch: time as u32,
            },
            client_id: Some(ProcessId {
                label: 1255760,
                epoch: time as u32,
            }),
            bind_result: Some(0),
            bind_response: Some(bind_response),
            server_time: Some(precise_time as u64),
            ..Default::default()
        };

        trace!(session.logger, "handshake response ready"; "response" => ?response_msg);
        let mut buffer = BytesMut::new();
        buffer.reserve(response_msg.encoded_len());
        response_msg.encode(&mut buffer)?;

        Ok((session, packet, Some(buffer.freeze())))
    }
}
