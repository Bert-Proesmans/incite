extern crate incite;

use incite::incite_gen::prost::Message;
use incite::incite_gen::proto::bnet::protocol::Header;

#[test]
fn header_decode() {
    let data = include_bytes!("connect_header.bin");
    let _header = Header::decode(&data[..]).expect("Error decoding header");
}
