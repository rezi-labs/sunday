use crate::config::DatabaseClient;
use actix_web::HttpRequest;
use actix_web::error::ErrorInternalServerError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub id: String,    // Session ID (random string)
    pub user_id: Uuid, // Foreign key to user
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

pub async fn validate(
    session_id: &str,
    db_client: &DatabaseClient,
) -> Result<Option<crate::db::user::Model>, actix_web::Error> {
    let client = db_client.lock().await;

    log::debug!("Validating session ID: {session_id}");

    let row = client.query_opt(
        "SELECT u.id, u.username, u.email, u.created_at, u.updated_at, u.last_login_at, u.is_active, u.is_admin 
         FROM session s JOIN sunday_user u ON s.user_id = u.id 
         WHERE s.id = $1 AND s.expires_at > NOW() AND u.is_active = true",
        &[&session_id],
    ).await
    .map_err(|e| {
        log::error!("Failed to validate session {session_id}: {e:?}");
        ErrorInternalServerError("Database query failed")
    })?;

    match row {
        Some(row) => {
            let user = crate::db::user::Model {
                id: row.get(0),
                username: row.get(1),
                email: row.get(2),
                created_at: row.get(3),
                updated_at: row.get(4),
                last_login_at: row.get(5),
                is_active: row.get(6),
                is_admin: row.get(7),
            };

            log::info!("Valid session found for user: {} ({})", user.email, user.id);

            // Update last accessed time
            let now = chrono::Utc::now();
            match client
                .execute(
                    "UPDATE session SET last_accessed_at = $1 WHERE id = $2",
                    &[&now, &session_id],
                )
                .await
            {
                Ok(_) => log::debug!("Session last_accessed_at updated successfully"),
                Err(e) => log::warn!("Failed to update last_accessed_at: {e:?}"),
            }

            Ok(Some(user))
        }
        None => {
            log::debug!(
                "No valid session found for ID: {session_id} (session may be expired or not exist)"
            );
            Ok(None)
        }
    }
}

// Session creation utility
pub async fn create(
    user_id: Uuid,
    request: &HttpRequest,
    db_client: &DatabaseClient,
) -> Result<Model, actix_web::Error> {
    let client = db_client.lock().await;

    let now = chrono::Utc::now();
    let expires_at = now + chrono::Duration::days(30); // 30-day session

    // Extract user agent from request headers
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // Extract IP address from request
    let ip_address = request
        .connection_info()
        .realip_remote_addr()
        .map(|s| s.to_string());

    let session_id = Uuid::new_v4().to_string();
    log::info!("Creating database session with ID: {session_id} for user: {user_id}");
    log::debug!(
        "Session details - expires_at: {expires_at}, user_agent: {user_agent:?}, ip_address: {ip_address:?}"
    );

    client.execute(
        "INSERT INTO session (id, user_id, created_at, expires_at, last_accessed_at, user_agent, ip_address) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        &[&session_id, &user_id, &now, &expires_at, &Some(now), &user_agent, &ip_address],
    ).await
    .map_err(|e| {
        log::error!("Failed to insert session into database: {e:?}");
        ErrorInternalServerError("Failed to create session")
    })?;

    log::info!("Database session successfully created with ID: {session_id}");

    Ok(Model {
        id: session_id,
        user_id,
        created_at: now,
        expires_at,
        last_accessed_at: Some(now),
        user_agent,
        ip_address,
    })
}

pub async fn delete_by_id(
    session_id: &str,
    db_client: &DatabaseClient,
) -> Result<u64, actix_web::Error> {
    let client = db_client.lock().await;

    let rows_affected = client
        .execute("DELETE FROM session WHERE id = $1", &[&session_id])
        .await
        .map_err(|_| ErrorInternalServerError("Failed to delete session"))?;

    Ok(rows_affected)
}
