use bytes::Bytes;
use futures::future;
use futures::prelude::*;

use protocol::bnet::frame::BNetPacket;
use rpc::system::{
    RPCReceiver, RPCResponder, RPCResult, RPCService, Request, Response, MAX_METHODS,
};
use rpc::util::hash_service_name;
use rpc::{Error, ErrorKind, Result};
use services::bnet::service_info::ExportedServiceID;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum ResponseMethod {
    Respond = 0,
}

#[derive(Debug, Default)]
pub struct ResponseService {}

impl ResponseService {
    pub const METHOD_RESPOND: ResponseMethod = ResponseMethod::Respond;
}

impl RPCService for ResponseService {
    type Method = ResponseMethod;
    type Incoming = BNetPacket;
    type Outgoing = BNetPacket;

    fn get_id() -> u32 {
        ExportedServiceID::ResponseService as u32
    }

    fn get_name() -> &'static str {
        "bnet.protocol.ResponseService"
    }

    fn get_hash() -> u32 {
        hash_service_name(Self::get_name())
    }

    fn get_methods() -> [Option<(&'static str, &'static Self::Method)>; MAX_METHODS] {
        [
            Some(("Respond", &Self::METHOD_RESPOND)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ]
    }
}

impl RPCResponder for ResponseService {
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

    fn call(&mut self, method: Self::Method, packet: Request<Self::Incoming>) -> Self::Future {
        unimplemented!()
    }
}

impl RPCReceiver for ResponseService {
    type Future = Box<dyn Future<Item = RPCResult<Self::Outgoing>, Error = Error>>;

    fn validate_response(packet: &Response<Self::Incoming>) -> Result<Self::Method> {
        unimplemented!()
    }

    fn revisit(&mut self, packet: Response<Self::Incoming>) -> Self::Future {
        unimplemented!()
    }
}
