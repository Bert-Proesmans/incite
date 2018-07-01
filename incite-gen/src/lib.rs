pub extern crate prost;

#[macro_use]
extern crate prost_derive;

#[allow(dead_code)]
pub mod version {
    // Provisioning by the pre_build script.
    include!(concat!(env!("OUT_DIR"), "/version.rs"));
}

#[allow(dead_code)]
pub mod proto {
    // Provisioning by the pre_build script.
    include!(concat!(env!("OUT_DIR"), "/combined_generated_proto.rs"));
}
