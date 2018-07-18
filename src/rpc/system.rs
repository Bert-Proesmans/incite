use super::{Error, Result};
use bytes::Bytes;
use futures::prelude::*;

pub const MAX_METHODS: usize = 10;

pub trait RPCService {
    type Method;
    type Incoming;
    type Outgoing;

    fn get_id() -> u32;
    fn get_name() -> &'static str;
    fn get_hash() -> u32;
    fn get_methods() -> [Option<(&'static str, &'static Self::Method)>; MAX_METHODS];
}

// Implemented by Server exported services.
pub trait RPCResponder: RPCService {
    type Future: Future<Item = RPCResult<Self::Outgoing>, Error = Error>;

    fn validate_request(packet: &Request<Self::Incoming>) -> Result<Self::Method>;
    fn call(&mut self, method: Self::Method, payload: Bytes) -> Self::Future;
}

// Implemented by Server imported services.
pub trait RPCReceiver: RPCService {
    type Future: Future<Item = RPCResult<Self::Outgoing>, Error = Error>;

    fn validate_response(packet: &Response<Self::Incoming>) -> Result<Self::Method>;
    fn revisit(&mut self, payload: Bytes) -> Self::Future;
}

#[derive(Debug)]
pub enum RPCResult<Packet> {
    Route(Route<Packet>),
    Response(Response<Packet>),
    Empty,
}

impl<Packet> From<Response<Packet>> for RPCResult<Packet> {
    fn from(x: Response<Packet>) -> Self {
        RPCResult::Response(x)
    }
}

impl<Packet> From<Route<Packet>> for RPCResult<Packet> {
    fn from(x: Route<Packet>) -> Self {
        RPCResult::Route(x)
    }
}

#[derive(Debug)]
pub struct Request<Packet>(Packet);

impl<Packet> Request<Packet> {
    pub fn new(data: Packet) -> Self {
        Request(data)
    }

    pub fn into_inner(self) -> Packet {
        self.0
    }

    pub fn as_ref(&self) -> Request<&Packet> {
        Request::new(&self.0)
    }
}

#[derive(Debug)]
pub struct Response<Packet>(Packet);

impl<Packet> Response<Packet> {
    pub fn new(data: Packet) -> Self {
        Response(data)
    }

    pub fn into_inner(self) -> Packet {
        self.0
    }

    pub fn as_ref(&self) -> Response<&Packet> {
        Response::new(&self.0)
    }
}

#[derive(Debug)]
// This packet wrapper will get additionall fields to route
// between clients as well.. soon(TM)
pub struct Route<Packet>(Packet);

impl<Packet> Route<Packet> {
    pub fn new(data: Packet) -> Self {
        Route(data)
    }

    pub fn into_inner(self) -> Packet {
        self.0
    }

    pub fn as_ref(&self) -> Route<&Packet> {
        Route::new(&self.0)
    }

    pub fn into_request(self) -> Request<Packet> {
        Request::new(self.0)
    }
}
