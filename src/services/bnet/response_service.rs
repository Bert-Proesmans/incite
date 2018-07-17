use futures::prelude::*;
use futures::future;
use std::u32::MAX;

use protocol::bnet::frame::BNetPacket;
use service::{hash_service_name, Error, RPCService, ErrorKind, Request, Response, Result, MAX_METHODS};

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum ResponseMethod {
    Respond = 0,
}

pub struct ResponseService {}

impl ResponseService {
    pub const METHOD_RESPOND: ResponseMethod = ResponseMethod::Respond;
}

impl RPCService for ResponseService {
    type Method = ResponseMethod;
    type Packet = BNetPacket;
    type Future = Box<dyn Future<Item = Option<Response<Self::Packet>>, Error = Error>>;
    // type FutureRevisit = Box<Future<Item = (), Error = Error>>;

    fn get_id() -> u32 {
        super::ServiceIDs::ResponseService as u32
    }

    fn get_name() -> &'static str {
        "bnet.protocol.ResponseService"
    }

    fn get_hash() -> u32 {
        hash_service_name(Self::get_name())
    }

    fn get_methods() -> [(&'static str, u32); MAX_METHODS] {
        [
            ("Respond", ResponseMethod::Respond as u32),
            ("", MAX),
            ("", MAX),
            ("", MAX),
            ("", MAX),
            ("", MAX),
            ("", MAX),
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

    fn call(&mut self, method: Self::Method, packet: Request<Self::Packet>) -> Self::Future {
        if let Err(e) = Self::validate_request(method as u32, &packet) {
            return Box::new(future::err(e));
        }

        unimplemented!()
    }

    /*
    fn revisit(&mut self, packet: Response<Self::Packet>) -> Self::FutureRevisit {
        unimplemented!()
    }
    */
}
