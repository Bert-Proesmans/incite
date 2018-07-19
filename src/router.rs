use bytes::Bytes;

pub struct RouteHeader {
    pub receiver: u64,
    pub service_id: u32,
    pub method_id: Option<u32>,
    pub token: u32,
    pub status: u32,
}

pub struct InternalPacket {
    pub header: RouteHeader,
    pub body: Bytes,
}

pub enum RoutablePacket {
    Stop,
    Return(InternalPacket),
    Next(InternalPacket),
}
