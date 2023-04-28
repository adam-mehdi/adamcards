
use diesel::prelude::*;
use serde::{
    Serialize, 
    Deserialize
};
use crate::schema::{quotas, cards};


#[derive(Deserialize, Serialize, Insertable, AsChangeset, Debug)]
#[diesel(table_name = quotas)]
pub struct QuotaRecord {
    pub days_to_go: i32,
    pub new_assigned: i32,
    pub review_assigned: i32,
    pub new_quota_initial: i32,
    pub review_quota_initial: i32,
    pub new_practiced: i32,
    pub review_practiced: i32,
}


#[derive(Deserialize, Serialize, Debug, Insertable, AsChangeset)]
#[diesel(table_name = cards)]
pub struct Card {
    pub id: i32,
    pub front: String,
    pub back: String,
    pub explanation: Option<String>
}

#[derive(Deserialize, Serialize, Insertable, Debug, AsChangeset)]
#[diesel(table_name = cards)]
pub struct NewCard {
    pub front: String,
    pub back: String
}





#[derive(Queryable)]
pub struct Deadline {
    pub id: i32,
    pub date_created: chrono::NaiveDateTime,
    pub deadline_date: chrono::NaiveDateTime,
    pub study_intensity: Option<i32>,
    pub num_reset: i32,
}

#[derive(Queryable, PartialEq)]
pub struct Deck {
    pub id: i32,
    pub date_created: chrono::NaiveDateTime,
}

#[derive(Queryable, Debug)]
pub struct Entry {
    pub id: i32,
    pub name: String,
    pub is_expanded: Option<bool>,
}


#[derive(Queryable)]
pub struct Folder {
    pub id: i32,
}


#[derive(Queryable)]
pub struct Parent {
    pub parent_id: i32,
    pub child_id: i32,
}
