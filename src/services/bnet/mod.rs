pub mod connection_service;
pub mod response_service;

#[repr(u32)]
pub enum ServiceIDs {
    ConnectionService = 1,
    // TODO
    ResponseService = 254,
}

pub use self::connection_service::ConnectionService;
pub use self::response_service::ResponseService;
