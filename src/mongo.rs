use chrono::Utc;
use mongodb::{
    Collection,
    bson::{Document, doc},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Website {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    pub url: String,
    pub last_updated: String,
    pub status: i32,
    pub telegram_id: String,
}

pub async fn put_site(
    collection: &Collection<Document>,
    website_url: &str,
    user_telegram_id: i32,
) -> Result<mongodb::bson::oid::ObjectId, mongodb::error::Error> {
    if website_url.is_empty() {
        return Err(mongodb::error::Error::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Website URL is empty",
        )));
    }

    // Check if website already exists for this user
    if let Some(existing_website) = collection
        .find_one(doc! {
            "url": website_url,
            "telegram_id": user_telegram_id.to_string()
        })
        .await?
    {
        let id = existing_website
            .get("_id")
            .ok_or_else(|| {
                mongodb::error::Error::from(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "No _id field found",
                ))
            })?
            .as_object_id()
            .ok_or_else(|| {
                mongodb::error::Error::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid ObjectId",
                ))
            })?;
        return Ok(id);
    }

    // Create new website document
    let new_website = doc! {
        "url": website_url,
        "last_updated": Utc::now().to_rfc3339(),
        "status": 200,
        "telegram_id": user_telegram_id.to_string()
    };

    // Insert the new website
    let result = collection.insert_one(new_website).await?;
    Ok(result.inserted_id.as_object_id().unwrap())
}

pub async fn delete_sites_by_hostname(
    collection: &Collection<Document>,
    hostname: &str,
    user_telegram_id: i32,
) -> Result<u64, mongodb::error::Error> {
    if hostname.len() < 3 {
        return Ok(0);
    }
    let pattern = format!("://{}", hostname);
    let filter = doc! {
        "url": { "$regex": pattern },
        "telegram_id": user_telegram_id.to_string()
    };

    let result = collection.delete_many(filter).await?;
    Ok(result.deleted_count)
}

pub async fn get_all_sites(
    collection: &Collection<Document>,
) -> Result<Vec<Website>, mongodb::error::Error> {
    let mut websites = Vec::new();
    let mut cursor = collection.find(doc! {}).await?;

    while cursor.advance().await? {
        let doc = cursor.deserialize_current()?;
        if let Ok(website) = mongodb::bson::from_document::<Website>(doc) {
            websites.push(website);
        }
    }

    Ok(websites)
}

pub async fn update_db(
    collection: &Collection<Document>,
    websites: &[Website],
) -> Result<(), mongodb::error::Error> {
    for website in websites {
        if let Some(id) = website.id {
            collection
                .update_one(
                    doc! { "_id": id },
                    doc! {
                        "$set": {
                            "status": website.status,
                            "last_updated": &website.last_updated,
                        },
                    },
                )
                .await?;
        }
    }

    Ok(())
}
