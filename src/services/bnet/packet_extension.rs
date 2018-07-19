use bytes::Bytes;
use incite_gen::proto::bnet::protocol::Header;
use incite_gen::prost;

use protocol::bnet::frame::BNetPacket;
use rpc::system::{RPCService, RPCResult, Request, Response};
use services::bnet::ResponseService;
use router::{RoutablePacket, InternalPacket, RouteHeader};

impl BNetPacket {
    pub fn try_as_request(self) -> Result<Request<Self>, Self> {
        // If it's not a response, it's a request!
        match self.try_as_response() {
            Ok(response) => Err(response.into_inner()),
            Err(packet) => Ok(Request::new(packet)),
        }
    }

    pub fn try_as_response(self) -> Result<Response<Self>, Self> {
        match (self.header.service_id, self.header.method_id) {
            (s_id, _) if s_id == ResponseService::get_id() => Ok(Response::new(self)),
            _ => Err(self),
        }
    }
}

impl Request<BNetPacket> {
    pub fn build_response(self, body: Bytes) -> Response<BNetPacket> {
        let BNetPacket { header, .. } = self.into_inner();
        let Header { token, .. } = header;

        let method_id = ResponseService::METHOD_RESPOND as u32;
        let header = Header {
            service_id: ResponseService::get_id(),
            method_id: Some(method_id),
            token: token,
            size: Some(body.len() as u32),
            ..Default::default()
        };

        let packet = BNetPacket::new(header, body);
        Response::new(packet)
    }
}
