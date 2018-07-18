use super::*;
use std::collections::HashMap;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum ExportedServiceID {
    ConnectionService = 0,

    AccountService = 1,
    AchievementsService = 2,
    AuthenticationServer = 3,
    ChallengeService = 4,
    ChannelInvitationService = 5,
    Channel = 6,
    ChannelOwner = 7,
    ExchangeService = 8,
    FriendsService = 9,
    GameMaster = 10,
    GameUtilities = 11,
    NotificationService = 12,
    PresenceService = 13,
    ReportService = 14,
    Resources = 15,
    SearchService = 16,
    UserManagerService = 17,

    ResponseService = 254,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum ImportedServiceID {
    ConnectionService = 0,

    AccountNotify = 1,
    AchievementsNotify = 2,
    AuthenticationClient = 3,
    ChallengeNotify = 4,
    ChannelInvitationNotify = 5,
    ChannelSubscriber = 6,
    ExchangeNotify = 7,
    DiagService = 8,
    FriendsNotify = 9,
    NotificationListener = 10,
    UserManagerNotify = 11,

    ResponseService = 254,
}

lazy_static! {
    pub static ref SERVICES_EXPORTED: HashMap<u32, ExportedServiceID> = {
        hashmap!{
            1698982289 => ExportedServiceID::ConnectionService,

            1658456209 => ExportedServiceID::AccountService,
            1128824125 => ExportedServiceID::AchievementsService,
            233634817 => ExportedServiceID::AuthenticationServer,
            3686756121 => ExportedServiceID::ChallengeService,
            2198078984 => ExportedServiceID::ChannelInvitationService,
            3073563442 => ExportedServiceID::Channel,
            101490829 => ExportedServiceID::ChannelOwner,
            3612349579 => ExportedServiceID::ExchangeService,
            2749215165 => ExportedServiceID::FriendsService,
            2165092757 => ExportedServiceID::GameMaster,
            1069623117 => ExportedServiceID::GameUtilities,
            213793859 => ExportedServiceID::NotificationService,
            4194801407 => ExportedServiceID::PresenceService,
            2091868617 => ExportedServiceID::ReportService,
            3971904954 => ExportedServiceID::Resources,
            170173073 => ExportedServiceID::SearchService,
            1041835658 => ExportedServiceID::UserManagerService,

            3625566374 => ExportedServiceID::ResponseService,
        }
    };
    pub static ref SERVICES_IMPORTED: HashMap<u32, ImportedServiceID> = {
        hashmap!{
            1698982289 => ImportedServiceID::ConnectionService,

            1423956503 =>  ImportedServiceID::AccountNotify,
            3571241107 =>  ImportedServiceID::AchievementsNotify,
            1898188341 =>  ImportedServiceID::AuthenticationClient,
            3151632159 =>  ImportedServiceID::ChallengeNotify,
            4035247136 =>  ImportedServiceID::ChannelInvitationNotify,
            3213656212 =>  ImportedServiceID::ChannelSubscriber,
            376431777 =>  ImportedServiceID::ExchangeNotify,
            3111080599 =>  ImportedServiceID::DiagService,
            1864735251 =>  ImportedServiceID::FriendsNotify,
            3788189352 => ImportedServiceID::NotificationListener,
            3162975266 => ImportedServiceID::UserManagerNotify,

            3625566374 => ImportedServiceID::ResponseService,
        }
    };
}
