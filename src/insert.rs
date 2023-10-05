use std::time::SystemTime;

use crate::schema::*;
use chrono::DateTime;
use chrono::Utc;
use diesel::insert_into;
use diesel::prelude::*;

// Models based on schema.rs
#[derive(Insertable)]
#[diesel(table_name = users)]
struct NewUser {
    telegram_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = websites)]
struct NewWebsite {
    url: String,
    last_checked_time: String,
    status: i32,
}

#[derive(Insertable)]
#[diesel(table_name = user_websites)]
struct NewUserWebsite {
    user_id: i32,
    website_id: i32,
}

// Helper function to find or insert a user
async fn put_user(conn: &mut SqliteConnection, teleg_id: i32) -> QueryResult<i32> {
    use crate::schema::users::dsl::*;

    match users.filter(telegram_id.eq(teleg_id)).first::<User>(conn) {
        Ok(user) => Ok(user.id),
        Err(_) => {
            let new_user = NewUser {
                telegram_id: teleg_id,
            };
            insert_into(users).values(&new_user).execute(conn)?;
            users.order(id.desc()).first::<User>(conn).map(|u| u.id)
        }
    }
}

// Helper function to find or insert a website
async fn put_website(conn: &mut SqliteConnection, website_url: &str) -> QueryResult<i32> {
    use crate::schema::websites::dsl::*;
    let datetime: DateTime<Utc> = SystemTime::now().into();

    match websites.filter(url.eq(website_url)).first::<Website>(conn) {
        Ok(website) => Ok(website.id),
        Err(_) => {
            let new_website = NewWebsite {
                url: website_url.to_string(),
                status: 200,
                last_checked_time: datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
            };
            insert_into(websites).values(&new_website).execute(conn)?;
            websites
                .order(id.desc())
                .first::<Website>(conn)
                .map(|w| w.id)
        }
    }
}

// Main function to insert user and website
pub async fn put_user_website(
    conn: &mut SqliteConnection,
    website_url: &str,
    user_telegram_id: i32,
) -> QueryResult<usize> {
    use crate::schema::user_websites::dsl::*;

    let curr_user_id = put_user(conn, user_telegram_id).await.expect("msg");
    let curr_website_id = put_website(conn, website_url).await.expect("msg");

    println!("{} {}", curr_user_id, curr_website_id);

    // Check if the relationship already exists
    match user_websites
        .filter(user_id.eq(curr_user_id).and(website_id.eq(curr_website_id)))
        .first::<UserWebsite>(conn)
    {
        Ok(_) => Ok(0), // relationship already exists
        Err(_) => {
            let new_user_website = NewUserWebsite {
                user_id: curr_user_id,
                website_id: curr_website_id,
            };
            insert_into(user_websites)
                .values(&new_user_website)
                .execute(conn)
        }
    }
}
