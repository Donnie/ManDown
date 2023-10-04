use crate::schema::{Website, User};
use diesel::{prelude::*, sqlite::SqliteConnection, dsl::*};

pub fn get_all_websites(conn: &mut SqliteConnection) -> Result<Vec<Website>, diesel::result::Error> {
    use crate::schema::websites::dsl::*;
    // Fetch all the Websites from the websites table
    let webs: Vec<Website> = websites.load(conn)?;

    Ok(webs)
}

pub async fn list_websites_by_user(conn: &mut SqliteConnection, telegram_id: i32) -> Result<Vec<Website>, diesel::result::Error> {
    use crate::schema::users::dsl::{users, id as uid, telegram_id as tele_id};
    use crate::schema::user_websites::dsl::*;
    use crate::schema::websites::dsl::{websites, id as wid, last_checked_time, status, url};

    let webs: Vec<Website> = users
        .inner_join(user_websites.on(user_id.eq(uid)))
        .inner_join(websites.on(wid.eq(website_id)))
        .filter(tele_id.eq(telegram_id))
        .select((wid, last_checked_time, status, url))
        .load(conn)?;

    Ok(webs)
}

pub async fn list_users_by_website(conn: &mut SqliteConnection, website_id: i32) -> Result<Vec<User>, diesel::result::Error> {
    use crate::schema::users::dsl::{users, id as uid, name, plan_type, telegram_id as tele_id};
    use crate::schema::user_websites::dsl::{user_websites, website_id as wid, user_id};

    let uss: Vec<User> = user_websites
        .filter(wid.eq(website_id))
        .inner_join(users.on(user_id.eq(uid)))
        .select((uid, name, plan_type, tele_id))
        .load(conn)?;

    Ok(uss)
}

pub fn compare_websites(conn: &mut SqliteConnection, webs: Vec<Website>) -> Result<Vec<Website>, diesel::result::Error> {
    let current_websites: Vec<Website> = get_all_websites(conn)?;

    let mut changed_websites: Vec<Website> = Vec::new();

    // compare and list websites only which have changed status
    for web in &webs {
        if let Some(current) = current_websites.iter().find(|&c| c.id == web.id) {
            if current.status != web.status {
                changed_websites.push(web.clone());
            }
        }
    }

    Ok(changed_websites)
}

pub fn write_all_websites(conn: &mut SqliteConnection, webs: Vec<Website>) -> Result<Vec<Website>, diesel::result::Error> {
    let ids: Vec<i32> = webs.iter().map(|web| web.id).collect();

    // Building the SQL update statement
    let mut sql = "UPDATE websites SET".to_string();

    sql += " last_checked_time = CASE id";
    for web in &webs {
        sql += &format!(" WHEN {} THEN '{}'", web.id, web.last_checked_time);
    }
    sql += " END,";

    sql += " status = CASE id";
    for web in &webs {
        sql += &format!(" WHEN {} THEN {}", web.id, web.status);
    }
    sql += " END WHERE id IN (";
    sql += &ids.iter().map(|id| id.to_string()).collect::<Vec<String>>().join(",");
    sql += ")";

    // Executing the SQL update statement
    sql_query(sql).execute(conn)?;

    Ok(webs)
}
