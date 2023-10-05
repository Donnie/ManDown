use crate::schema::{User, Website};
use diesel::{dsl::*, prelude::*, sqlite::SqliteConnection};
use url::Url;

pub fn get_all_websites(
    conn: &mut SqliteConnection,
) -> Result<Vec<Website>, diesel::result::Error> {
    use crate::schema::websites::dsl::*;
    // Fetch all the Websites from the websites table
    let webs: Vec<Website> = websites.load(conn)?;

    Ok(webs)
}

pub async fn list_websites_by_user(
    conn: &mut SqliteConnection,
    telegram_id: i32,
) -> Result<Vec<Website>, diesel::result::Error> {
    use crate::schema::user_websites::dsl::*;
    use crate::schema::users::dsl::{id as uid, telegram_id as tele_id, users};
    use crate::schema::websites::dsl::{id as wid, last_checked_time, status, url, websites};

    let webs: Vec<Website> = users
        .inner_join(user_websites.on(user_id.eq(uid)))
        .inner_join(websites.on(wid.eq(website_id)))
        .filter(tele_id.eq(telegram_id))
        .select((wid, last_checked_time, status, url))
        .load(conn)?;

    Ok(webs)
}

pub async fn get_user_by_telegram_id(
    conn: &mut SqliteConnection,
    tele_id: i32,
) -> Result<User, diesel::result::Error> {
    use crate::schema::users::dsl::*;
    users.filter(telegram_id.eq(tele_id)).first(conn)
}

pub async fn get_websites_by_url(
    conn: &mut SqliteConnection,
    wurl: &str,
) -> Result<Vec<Website>, diesel::result::Error> {
    use crate::schema::websites::dsl::*;
    websites
        .filter(url.like(format!("htt%://{}", wurl)))
        .load(conn)
}

pub async fn delete_user_website(
    conn: &mut SqliteConnection,
    w_id: i32,
    u_id: i32,
) -> QueryResult<usize> {
    use crate::schema::user_websites::dsl::*;
    diesel::delete(user_websites.filter(user_id.eq(u_id).and(website_id.eq(w_id)))).execute(conn)
}

pub async fn delete_website(conn: &mut SqliteConnection, w_id: i32) -> QueryResult<usize> {
    use crate::schema::websites::dsl::*;
    diesel::delete(websites.filter(id.eq(w_id))).execute(conn)
}

pub async fn delete_user(conn: &mut SqliteConnection, u_id: i32) -> QueryResult<usize> {
    use crate::schema::users::dsl::*;
    diesel::delete(users.filter(id.eq(u_id))).execute(conn)
}

pub async fn list_users_by_website(
    conn: &mut SqliteConnection,
    website_id: i32,
) -> Result<Vec<User>, diesel::result::Error> {
    use crate::schema::user_websites::dsl::{user_id, user_websites, website_id as wid};
    use crate::schema::users::dsl::{id as uid, name, user_type, telegram_id as tele_id, users};

    let uss: Vec<User> = user_websites
        .filter(wid.eq(website_id))
        .inner_join(users.on(user_id.eq(uid)))
        .select((uid, name, user_type, tele_id))
        .load(conn)?;

    Ok(uss)
}

pub fn compare_websites(
    conn: &mut SqliteConnection,
    webs: Vec<Website>,
) -> Result<Vec<Website>, diesel::result::Error> {
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

pub fn write_all_websites(
    conn: &mut SqliteConnection,
    webs: Vec<Website>,
) -> Result<Vec<Website>, diesel::result::Error> {
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
    sql += &ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<String>>()
        .join(",");
    sql += ")";

    // Executing the SQL update statement
    sql_query(sql).execute(conn)?;

    Ok(webs)
}

fn try_parse_url(input: &str) -> Option<Url> {
    Url::parse(input)
        .or_else(|_| Url::parse(&format!("http://{}", input)))
        .ok()
}

pub fn extract_hostname(input: &str) -> String {
    let host = try_parse_url(input)
        .and_then(|url| url.host_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "".to_string());

    // Ensure that the host contains a dot (indicating presence of a TLD)
    if host.contains('.') {
        host
    } else {
        "".to_string()
    }
}

pub fn read_url(input: &str) -> (bool, String, String) {
    let url = extract_hostname(input);
    if url == "" {
        return (false, "".to_string(), "".to_string());
    }
    return (true, format!("http://{}", url), format!("https://{}", url));
}
