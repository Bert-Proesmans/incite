
/*
 * Known hashes
 *
 * ConnectionService is the only service bound two-ways
 * 1698982289	- bnet.protocol.connection.ConnectionService
 *
 * Client imported (from server)
 * 233634817	- bnet.protocol.authentication.AuthenticationServer
 * 1069623117	- bnet.protocol.game_utilities.GameUtilities
 * 2165092757	- bnet.protocol.game_master.GameMaster
 * 4194801407	- bnet.protocol.presence.PresenceService
 * 3073563442	- bnet.protocol.channel.Channel
 * 101490829	- bnet.protocol.channel.ChannelOwner
 * 2198078984	- bnet.protocol.channel_invitation.ChannelInvitationService
 * 213793859	- bnet.protocol.notification.NotificationService
 * 2749215165	- bnet.protocol.friends.FriendsService
 * 3686756121	- bnet.protocol.challenge.ChallengeService
 * 1658456209	- bnet.protocol.account.AccountService
 * 3971904954	- bnet.protocol.resources.Resources
 * 1128824125	- bnet.protocol.achievements.AchievementsService
 * 3612349579	- bnet.protocol.exchange.ExchangeService
 * 2091868617	- bnet.protocol.report.ReportService
 * 170173073	- bnet.protocol.search.SearchService
 * 1041835658	- bnet.protocol.user_manager.UserManagerService
 *
 * Client exported (from client)
 * 1898188341	- bnet.protocol.authentication.AuthenticationClient
 * 4035247136	- bnet.protocol.channel_invitation.ChannelInvitationNotify
 * 3213656212	- bnet.protocol.channel.ChannelSubscriber
 * 3788189352	- bnet.protocol.notification.NotificationListener
 * 1864735251	- bnet.protocol.friends.FriendsNotify
 * 3151632159	- bnet.protocol.challenge.ChallengeNotify
 * 1423956503	- bnet.protocol.account.AccountNotify
 * 3571241107	- bnet.protocol.achievements.AchievementsNotify
 * 3111080599	- bnet.protocol.diag.DiagService
 * 376431777	- bnet.protocol.exchange.ExchangeNotify
 * 3162975266	- bnet.protocol.user_manager.UserManagerNotify
 *
 *
 */

/*
 * Unused services
 * 689160787	- bnet.protocol.achievements.AchievementsUtils
 * 3338259653	- bnet.protocol.game_master.GameFactorySubscriber
 * 3826086206	- bnet.protocol.game_master.GameRequestSubscriber
 */

// use bytes::Bytes;
// use futures::prelude::*;

const FNV1A_INIT: u32 = 0x811c9dc5;
const FNV1A_PRIME: u32 = 0x01000193;

mod error {
    use incite_gen::prost;

    error_chain!{
        foreign_links {
            ProtoEncode(prost::EncodeError) #[doc = "Error during Protobuffer encoding"];
            ProtoDecode(prost::DecodeError) #[doc = "Error during Protobuffer decoding"];
        }

        errors {
            InvalidRequest(method_id: u32, service_name: &'static str) {
                description("The request is malformed")
                display("Received a malformed request for service {}, method {}", service_name, method_id)
            }
        }
    }
}

pub use self::error::Error as RPCError;

pub fn hash_service_name<S: AsRef<str>>(name: S) -> u32 {
	let mut hash = FNV1A_INIT;
	for byte in name.as_ref().bytes() {
		hash = hash ^ byte as u32;
		hash = hash.overflowing_mul(FNV1A_PRIME).0;
	}

	return hash;
}

pub trait RPCService {
	fn get_name() -> &'static str;
	fn get_hash() -> u32;
	fn get_methods() -> [&'static str; 10];

	// fn invoke(&mut self, method: u32, data: Bytes) -> impl Future<Item=Option<Bytes>, Error=RPCError>;
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
	}
}
