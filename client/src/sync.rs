use serde::{Deserialize, Serialize};
use crate::db;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    pub id: String,
    pub user_id: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
    pub content: String,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub completed: bool,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub pinned: bool,
    pub version: i32,
    pub device_id: String,
}

fn deserialize_bool_from_anything<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum BoolOrInt {
        Bool(bool),
        Int(i32),
    }

    match BoolOrInt::deserialize(deserializer)? {
        BoolOrInt::Bool(b) => Ok(b),
        BoolOrInt::Int(i) => Ok(i != 0),
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SyncRequest {
    pub last_sync: i64,
    pub changes: Vec<Task>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SyncResponse {
    pub updates: Vec<Task>,
    pub server_time: i64,
}

pub async fn sync_with_server(server_url: &str, request: SyncRequest) -> Result<SyncResponse, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let res = client.post(format!("{}/sync", server_url))
        .header("x-user-id", "dev-user") // TODO: real auth
        .json(&request)
        .send()
        .await?;

    // Check for success status before trying to parse JSON
    if !res.status().is_success() {
         let error_text = res.text().await?;
         return Err(format!("Server error: {}", error_text).into());
    }

    let response = res.json::<SyncResponse>().await?;
    Ok(response)
}

pub async fn run_sync_cycle() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Init DB connection
    let conn = db::init()?;
    
    // 2. Get last sync time
    let last_sync = db::get_last_sync_timestamp(&conn)?;
    
    // 3. Get local changes
    // Issue: last_sync is the time of the last SUCCESSFUL sync from server.
    // If we use it to find local changes, we might re-send changes we just got from server?
    // No, server handles that (idempotency or timestamps checks).
    // Better logic: track `local_last_sync` vs `server_last_sync`. 
    // For V1 simple logic: send all tasks modified after `last_sync`. 
    // Ideally we should track a "dirty" flag or a separate pointer.
    // For now, let's just use `last_sync` minus a buffer, or send everything "new/modified".
    let changes = db::get_changes_since(&conn, last_sync)?;

    // 4. Send to server
    // TODO: move URL to config
    let server_url = "http://localhost:8787"; 
    
    let request = SyncRequest {
        last_sync,
        changes,
    };

    println!("Syncing..."); // Heartbeat

    let response = sync_with_server(server_url, request).await?;

    // 5. Apply updates
    let mut applied_count = 0;
    for task in response.updates {
        db::upsert_task(&conn, &task)?;
        applied_count += 1;
    }

    if applied_count > 0 {
        println!("Synced: Applied {} updates from server.", applied_count);
    } else {
        println!("Synced: No new updates.");
    }
    
    // 6. Update last sync time
    db::set_last_sync_timestamp(&conn, response.server_time)?;

    Ok(())
}
