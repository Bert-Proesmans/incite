use chrono::NaiveDateTime;

#[derive(Queryable)]
pub struct Account {
    pub id: i32,
    pub email: String,
    pub web_credential: String,
    pub battle_tag: String,
    pub full_name: Option<String>,
    pub last_online: NaiveDateTime,
    pub last_away_time: NaiveDateTime,
    pub last_invisible_time: NaiveDateTime,
    pub dnd: Option<bool>,
    pub flags: Option<i64>,
}
