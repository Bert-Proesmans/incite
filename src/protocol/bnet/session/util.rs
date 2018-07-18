use bytes::Bytes;
use std::default::Default;

use incite_gen::proto::bnet::protocol::Header;
use protocol::bnet::frame::BNetPacket;
use protocol::bnet::session::{ErrorKind, Result};
use rpc::system::{RPCService, Request, Response};
use services::bnet::ResponseService;

impl Request<BNetPacket> {
    pub fn from_bnet(packet: BNetPacket) -> Result<Self> {
        match (packet.header.service_id, packet.header.method_id) {
            (s_id, _) if s_id != ResponseService::get_id() => {}
            _ => Err(ErrorKind::MissingRequest)?,
        };

        Ok(Request::new(packet))
    }

    pub fn into_response(self, payload: Bytes) -> Response<BNetPacket> {
        let BNetPacket { header, .. } = self.into_inner();
        let Header { token, .. } = header;

        let method_id = ResponseService::METHOD_RESPOND as u32;
        let header = Header {
            service_id: ResponseService::get_id(),
            method_id: Some(method_id),
            token: token,
            size: Some(payload.len() as u32),
            ..Default::default()
        };

        let packet = BNetPacket::new(header, payload);
        Response::new(packet)
    }
}
