use incite_gen::prost;

error_chain!{
    foreign_links {
        ProtoEncode(prost::EncodeError) #[doc = "Error during Protobuffer encoding"];
        ProtoDecode(prost::DecodeError) #[doc = "Error during Protobuffer decoding"];
    }

    errors {
        UnknownRequest(service_name: &'static str) {
            description("Not enough information is provided to execute the request")
            display("A malformed request for service {} has been received", service_name)
        }
        InvalidRequest(method_id: u32, service_name: &'static str) {
            description("The request is malformed")
            display("A malformed request for service {}, method {} was received", service_name, method_id)
        }
        InvalidResponse(token: u32) {
            description("Unexpected response")
            display("The client sent an ad-hoc response")
        }
    }
}
