// @generated automatically by Diesel CLI.

diesel::table! {
    user_websites (user_id, website_id) {
        user_id -> Nullable<Integer>,
        website_id -> Nullable<Integer>,
        last_checked_time -> Nullable<Text>,
    }
}

diesel::table! {
    users (user_id) {
        user_id -> Nullable<Integer>,
        name -> Nullable<Text>,
        plan_type -> Nullable<Text>,
        telegram_id -> Nullable<Text>,
    }
}

diesel::table! {
    websites (website_id) {
        website_id -> Nullable<Integer>,
        status -> Nullable<Integer>,
        url -> Nullable<Binary>,
    }
}

diesel::joinable!(user_websites -> users (user_id));
diesel::joinable!(user_websites -> websites (website_id));

diesel::allow_tables_to_appear_in_same_query!(
    user_websites,
    users,
    websites,
);
