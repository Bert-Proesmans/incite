use incite_gen::prost;
use protocol::bnet::frame;
use rpc;
use std::io;

error_chain!{
    errors {
        ClientDisconnected {
            description("The client hung-up on us")
            display("The client disconnected unexpectedly")
        }
        MissingRequest {
            description("We expected a request, but the client didn't send any")
            display("The client didn't send us correct information")
        }
        ProcedureFail(step: &'static str) {
            description("The client failed to send the proper response")
            display("The client didn't respond correctly during '{}'", step)
        }
        Timeout {
            description("The code triggered a timeout error to prevent rogue clients")
            display("The client failed to respond within the time limit")
        }
        StatePoisoning {
            description("Some thread holding the lock panicked, resulting in an invalid state")
            display("The shared server state has been poisoned")
        }
    }

    links {
        Framing(frame::Error, frame::ErrorKind);
        Service(rpc::Error, rpc::ErrorKind);
    }

    foreign_links {
        Io(io::Error) #[doc = "Error during IO"];
        ProtoEncode(prost::EncodeError) #[doc = "Error during Protobuffer encoding"];

    }
}
