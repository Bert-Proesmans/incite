/*
 * Known hashes
 *
 * ConnectionService is the only service bound two-ways
 * 1698982289	- bnet.protocol.connection.ConnectionService
 *
 * Client imported (from server)
 * The client asks us which services map to which ID. The services below
 *  are listed in asking order. We can just return a direct mapping back
 *  so the services are locked to the order asked by the client.
 *
 * 1-   1658456209  - bnet.protocol.account.AccountService
 * 2-   1128824125  - bnet.protocol.achievements.AchievementsService
 * 3-   233634817   - bnet.protocol.authentication.AuthenticationServer
 * 4-   3686756121  - bnet.protocol.challenge.ChallengeService
 * 5-   2198078984  - bnet.protocol.channel_invitation.ChannelInvitationService
 * 6-   3073563442  - bnet.protocol.channel.Channel
 * 7-   101490829   - bnet.protocol.channel.ChannelOwner
 * 8-   3612349579  - bnet.protocol.exchange.ExchangeService
 * 9-   2749215165  - bnet.protocol.friends.FriendsService
 * 10-  2165092757  - bnet.protocol.game_master.GameMaster
 * 11-  1069623117  - bnet.protocol.game_utilities.GameUtilities
 * 12-  213793859   - bnet.protocol.notification.NotificationService
 * 13-  4194801407  - bnet.protocol.presence.PresenceService
 * 14-  2091868617  - bnet.protocol.report.ReportService
 * 15-  3971904954  - bnet.protocol.resources.Resources
 * 16-  170173073   - bnet.protocol.search.SearchService
 * 17-  1041835658  - bnet.protocol.user_manager.UserManagerService
 *
 * Client exported (from client)
 * The client tells us which services map to which ID. The following ID's are
 * absolute, deviation is not allowed here.
 *
 * 1-   1423956503  - bnet.protocol.account.AccountNotify
 * 2-   3571241107  - bnet.protocol.achievements.AchievementsNotify
 * 3-   1898188341  - bnet.protocol.authentication.AuthenticationClient
 * 4-   3151632159  - bnet.protocol.challenge.ChallengeNotify
 * 5-   4035247136  - bnet.protocol.channel_invitation.ChannelInvitationNotify
 * 6-   3213656212  - bnet.protocol.channel.ChannelSubscriber
 * 7-   376431777   - bnet.protocol.exchange.ExchangeNotify
 * 8-   3111080599  - bnet.protocol.diag.DiagService
 * 9-   1864735251  - bnet.protocol.friends.FriendsNotify
 * 10-  3788189352  - bnet.protocol.notification.NotificationListener
 * 11-  3162975266  - bnet.protocol.user_manager.UserManagerNotify
 *
 */

/*
 * Unused services
 * 689160787	- bnet.protocol.achievements.AchievementsUtils
 * 3338259653	- bnet.protocol.game_master.GameFactorySubscriber
 * 3826086206	- bnet.protocol.game_master.GameRequestSubscriber
 */

// use bytes::Bytes;
use futures::prelude::*;

const FNV1A_INIT: u32 = 0x811c9dc5;
const FNV1A_PRIME: u32 = 0x01000193;

pub mod error {
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
}
pub use self::error::*;

pub fn hash_service_name<S: AsRef<str>>(name: S) -> u32 {
    let mut hash = FNV1A_INIT;
    for byte in name.as_ref().bytes() {
        hash = hash ^ byte as u32;
        hash = hash.overflowing_mul(FNV1A_PRIME).0;
    }

    return hash;
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

pub const MAX_METHODS: usize = 10;

pub trait RPCService {
    type Method;
    type Packet;
    type Future: Future<Item = Option<Response<Self::Packet>>, Error = Error>;
    // type FutureRevisit: Future<Item = (), Error = Error>;

    fn get_id() -> u32;
    fn get_name() -> &'static str;
    fn get_hash() -> u32;
    fn get_methods() -> [(&'static str, u32); MAX_METHODS];

    fn validate_request(request_method_id: u32, packet: &Request<Self::Packet>) -> Result<()>;

    fn call(&mut self, method: Self::Method, packet: Request<Self::Packet>) -> Self::Future;
    // fn revisit(&mut self, packet: Response<Self::Packet>) -> Self::FutureRevisit;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn hash_verification() {
        let test_one = "bnet.protocol.authentication.AuthenticationServer";
        let hash_one = hash_service_name(test_one);
        assert_eq!(233634817, hash_one);

        let test_two = "bnet.protocol.channel.ChannelSubscriber";
        let hash_two = hash_service_name(test_two);
        assert_eq!(3213656212, hash_two);

        let response_test = "bnet.protocol.ResponseService";
        let hash_response = hash_service_name(response_test);
        println!("{:} - {:}", hash_response, response_test);
    }
}
