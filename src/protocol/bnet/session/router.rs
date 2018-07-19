use frunk::hlist::HList;
use frunk::{HCons, HNil};
use futures::prelude::*;
use std::mem;

use protocol::bnet::frame::BNetPacket;
use protocol::bnet::session::full::ClientData;
use protocol::bnet::session::{Error, Result};
use router::RoutablePacket;
use rpc::system::{RPCReceiver, RPCResponder, RPCResult, RPCService, Request, Response};
use services::bnet::service_info::ServiceStorage;
use services::bnet::*;

pub struct BNetRouter<Data> {
    self_id: u64,
    services: Option<ServiceStorage>,
    packets_forward: Vec<u32>,
    pub packets_out: Vec<u32>,
    pub data: Data,
}

impl<Data> BNetRouter<Data> {
    pub fn new(self_id: u64, services: ServiceStorage, data: Data) -> Self {
        BNetRouter {
            self_id,
            services: Some(services),
            data,
            packets_forward: vec![],
            packets_out: vec![],
        }
    }

    #[async]
    pub fn do_the_thing(self, packet: BNetPacket) -> Result<Self> {
        unimplemented!()
    }
}

pub use self::responder::RPC_RESPONDER_TABLE;

macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + count!($($xs)*));
}

mod responder {
    use super::*;

    type ResponderInstance = fn(
        BNetRouter<ClientData>, packet: Request<BNetPacket>
    ) -> Result<(
        BNetRouter<ClientData>,
        Option<Box<dyn Future<Item = RPCResult<BNetPacket>, Error = Error>>>,
    )>;

    macro_rules! routes_responder {
        ($($service:ident),*) => {
            mashup! {
                $(
                    m["responder_core_impl" $service] = responder_ $service;
                    m["responder_impl" $service] = routed_responder_ $service;
                )*
            }

            m! {
                $(
                #[allow(non_snake_case)]
                fn "responder_core_impl" $service (
                    method: <$service as RPCService>::Method, services: ServiceStorage, packet: Request<BNetPacket>
                ) -> (ServiceStorage, impl Future<Item = RPCResult<BNetPacket>, Error = Error>) {
                    // Extract service data
                    let (mut service_obj, services_remainder): ($service, _) = services.pluck();
                    // Execute service, this returns a future we must await.
                    let fut_response = service_obj.call(method, packet);
                    // Make response compatible with router.
                    let fut_response = fut_response.map(Into::into).map_err(Into::into);
                    // Resculpt services
                    let (services, _) = services_remainder.prepend(service_obj).sculpt();
                    (services, fut_response)
                }
                )*

                $(
                #[allow(non_snake_case)]
                fn "responder_impl" $service (
                        mut router: BNetRouter<ClientData>, packet: Request<BNetPacket>,
                ) -> Result<(BNetRouter<ClientData>, Option<Box<dyn Future<Item = RPCResult<BNetPacket>, Error = Error>>>)>
                where
                    $service: RPCResponder<Incoming = BNetPacket>,
                    RPCResult<<$service as RPCService>::Outgoing>: Into<RPCResult<BNetPacket>>,
                {
                    if let Ok(method) = $service::validate_request(&packet) {
                        // Extract services
                        let services = router.services.take().unwrap();
                        let (services, fut_response) = "responder_core_impl" $service(method, services, packet);
                        // Reinstall services
                        router.services = Some(services);
                        // Notify caller that the service must be ran.
                        return Ok((router, Some(Box::new(fut_response))))
                    }

                    Ok((router, None))
                }
                )*

                pub const RPC_RESPONDER_TABLE: [ResponderInstance; count!($($service )*)] = [
                     $("responder_impl" $service),*
                ];
            }
        }
    }

    routes_responder![ConnectionService, ResponseService];
}

mod receiver {}

fn routed_receiver_instance<Service>(
    router: BNetRouter<ClientData>,
    packet: Response<BNetPacket>,
) -> Result<BNetRouter<ClientData>>
where
    Service: RPCReceiver<Incoming = BNetPacket>,
{
    unimplemented!()
}

/*
type ReceiverInstance = fn(RouterData<ClientData, Response<BNetPacket>>)
    -> Result<RouterData<ClientData, Response<BNetPacket>>>;
*/

/*
pub const RPC_RECEIVER_TABLE: [ReceiverInstance; 1] = [routed_receiver_instance::<ResponseService>];
*/
