table! {
    accounts (id) {
        id -> Integer,
        email -> Text,
        web_credential -> Text,
        battle_tag -> Text,
        full_name -> Nullable<Text>,
        last_online -> Timestamp,
        last_away_time -> Timestamp,
        last_invisible_time -> Timestamp,
        dnd -> Nullable<Bool>,
        flags -> Nullable<BigInt>,
    }
}
