use chrono::Utc;
use futures::StreamExt;
use mongodb::{
    Client, Collection,
    bson::{Document, doc},
    options::ClientOptions,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Website {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    pub url: String,
    pub last_updated: String,
    pub status: i32,
    pub telegram_id: String,
}

pub async fn init_mongo() -> Arc<Collection<Document>> {
    let uri = dotenvy::var("MONGODB_URI").expect("MONGODB_URI must be set");
    let mut client_options = ClientOptions::parse(&uri)
        .await
        .expect("Failed to parse MongoDB URI");

    // Configure timeouts to prevent deadline exceeded errors
    client_options.server_selection_timeout = Some(Duration::from_secs(5));
    client_options.connect_timeout = Some(Duration::from_secs(5));
    client_options.max_pool_size = Some(10);
    client_options.min_pool_size = Some(1);
    client_options.max_idle_time = Some(Duration::from_secs(300)); // 5 minutes
    client_options.retry_writes = Some(true);
    client_options.retry_reads = Some(true);

    let client = Client::with_options(client_options).expect("Failed to create MongoDB client");
    let db = client.database("mandown");
    Arc::new(db.collection::<Document>("websites"))
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

pub async fn clear_user_websites(
    collection: &Collection<Document>,
    user_telegram_id: i32,
) -> Result<u64, mongodb::error::Error> {
    let filter = doc! {
        "telegram_id": user_telegram_id.to_string()
    };

    let result = collection.delete_many(filter).await?;
    Ok(result.deleted_count)
}

pub async fn delete_sites_by_hostname(
    collection: &Collection<Document>,
    hostname: &str,
    user_telegram_id: i32,
) -> Result<u64, mongodb::error::Error> {
    if hostname.len() < 3 {
        return Ok(0);
    }
    let pattern = format!("://{hostname}");
    let filter = doc! {
        "url": { "$regex": pattern },
        "telegram_id": user_telegram_id.to_string()
    };

    let result = collection.delete_many(filter).await?;
    Ok(result.deleted_count)
}

pub async fn get_sites(
    collection: &Collection<Document>,
    skip: u64,
    limit: i64,
) -> Result<Vec<Website>, mongodb::error::Error> {
    let mut websites = Vec::new();
    let find_options = mongodb::options::FindOptions::builder()
        .skip(skip)
        .limit(limit)
        .build();
    let mut cursor = collection.find(doc! {}).with_options(find_options).await?;

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

pub async fn get_user_websites(
    collection: &Collection<Document>,
    telegram_id: i32,
) -> Result<Vec<Website>, mongodb::error::Error> {
    let filter = doc! { "telegram_id": format!("{telegram_id}") };

    let mut cursor = collection.find(filter).await.map_err(|e| {
        log::error!("Failed to query MongoDB: {e}");
        mongodb::error::Error::from(std::io::Error::other(e))
    })?;

    let mut websites: Vec<Website> = Vec::new();
    while let Some(doc_result) = cursor.next().await {
        match doc_result {
            Ok(doc) => {
                if let Ok(website) = mongodb::bson::from_document::<Website>(doc) {
                    websites.push(website);
                }
            }
            Err(e) => {
                log::error!("Failed to read document: {e}");
                return Err(mongodb::error::Error::from(std::io::Error::other(e)));
            }
        }
    }

    websites.sort_by(|a, b| a.url.cmp(&b.url));
    Ok(websites)
}
