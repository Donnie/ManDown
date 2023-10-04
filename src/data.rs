use crate::schema::{websites, Website};
use diesel::{prelude::*, sqlite::SqliteConnection, dsl::*};

pub fn get_all_websites(conn: &mut SqliteConnection) -> Result<Vec<Website>, diesel::result::Error> {
    // Fetch all the Websites from the websites table
    let websites: Vec<Website> = websites::dsl::websites.load(conn)?;

    Ok(websites)
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
