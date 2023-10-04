use diesel::Queryable;

// @generated automatically by Diesel CLI.

diesel::table! {
    user_websites (user_id, website_id) {
        user_id -> Integer,
        website_id -> Integer,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        name -> Nullable<Text>,
        plan_type -> Nullable<Text>,
        telegram_id -> Integer,
    }
}

diesel::table! {
    websites (id) {
        id -> Integer,
        last_checked_time -> Text,
        status -> Integer,
        url -> Text,
    }
}

diesel::joinable!(user_websites -> users (user_id));
diesel::joinable!(user_websites -> websites (website_id));

diesel::allow_tables_to_appear_in_same_query!(
    user_websites,
    users,
    websites,
);

#[derive(Queryable, Clone)]
pub struct User {
    pub id: i32,
    pub name: Option<String>,
    pub plan_type: Option<String>,
    pub telegram_id: i32,
}

#[derive(Queryable, Clone)]
pub struct Website {
    pub id: i32,
    pub last_checked_time: String,
    pub status: i32,
    pub url: String,
}

#[derive(Queryable, Clone)]
pub struct UserWebsite {
    pub user_id: i32,
    pub website_id: i32,
}
