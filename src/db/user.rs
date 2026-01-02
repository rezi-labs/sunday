use crate::config::DatabaseClient;
use actix_web::error::ErrorInternalServerError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// User Model
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_active: bool,
    pub is_admin: bool,
}

pub async fn create(
    user_name: String,
    email: String,
    db_client: &DatabaseClient,
    is_active: bool,
) -> Result<(), actix_web::Error> {
    let client = db_client.lock().await;

    let user_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    client.execute(
        "INSERT INTO sunday_user (id, username, email, created_at, updated_at, last_login_at, is_active, is_admin) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        &[&user_id, &user_name, &email, &now, &None::<chrono::DateTime<chrono::Utc>>, &None::<chrono::DateTime<chrono::Utc>>, &is_active, &false],
    ).await
    .map_err(|_| ErrorInternalServerError("Failed to create user"))?;

    Ok(())
}

pub async fn create_admin(
    user_name: String,
    email: String,
    db_client: &DatabaseClient,
) -> Result<(), actix_web::Error> {
    let client = db_client.lock().await;

    let user_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    client.execute(
        "INSERT INTO sunday_user (id, username, email, created_at, updated_at, last_login_at, is_active, is_admin) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        &[&user_id, &user_name, &email, &now, &None::<chrono::DateTime<chrono::Utc>>, &None::<chrono::DateTime<chrono::Utc>>, &true, &true],
    ).await
    .map_err(|_| ErrorInternalServerError("Failed to create admin user"))?;

    Ok(())
}

pub async fn find_by_email(
    email: &str,
    db_client: &DatabaseClient,
) -> Result<Option<Model>, actix_web::Error> {
    let client = db_client.lock().await;

    let row = client.query_opt(
        "SELECT id, username, email, created_at, updated_at, last_login_at, is_active, is_admin FROM sunday_user WHERE email = $1",
        &[&email],
    ).await
    .map_err(|e| {
        log::error!("Failed to find user by email {email}: {e:?}");
        ErrorInternalServerError("Database query failed")
    })?;

    match row {
        Some(row) => {
            let user = Model {
                id: row.get(0),
                username: row.get(1),
                email: row.get(2),
                created_at: row.get(3),
                updated_at: row.get(4),
                last_login_at: row.get(5),
                is_active: row.get(6),
                is_admin: row.get(7),
            };
            Ok(Some(user))
        }
        None => Ok(None),
    }
}

pub async fn find_by_username(
    username: &str,
    db_client: &DatabaseClient,
) -> Result<Option<Model>, actix_web::Error> {
    let client = db_client.lock().await;

    let row = client.query_opt(
        "SELECT id, username, email, created_at, updated_at, last_login_at, is_active, is_admin FROM sunday_user WHERE username = $1",
        &[&username],
    ).await
    .map_err(|e| {
        log::error!("Failed to find user by username {username}: {e:?}");
        ErrorInternalServerError("Database query failed")
    })?;

    match row {
        Some(row) => {
            let user = Model {
                id: row.get(0),
                username: row.get(1),
                email: row.get(2),
                created_at: row.get(3),
                updated_at: row.get(4),
                last_login_at: row.get(5),
                is_active: row.get(6),
                is_admin: row.get(7),
            };
            Ok(Some(user))
        }
        None => Ok(None),
    }
}

pub async fn update_last_login(
    user: &Model,
    db_client: &DatabaseClient,
) -> Result<(), actix_web::Error> {
    let client = db_client.lock().await;

    let now = chrono::Utc::now();

    client
        .execute(
            "UPDATE sunday_user SET last_login_at = $1 WHERE id = $2",
            &[&now, &user.id],
        )
        .await
        .map_err(|_| ErrorInternalServerError("Failed to update last login"))?;

    Ok(())
}

pub async fn list_all(
    db_client: &DatabaseClient,
    search_term: Option<&str>,
) -> Result<Vec<Model>, actix_web::Error> {
    let client = db_client.lock().await;

    let rows = match search_term {
        Some(term) => {
            let search_pattern = format!("%{}%", term.to_lowercase());
            client.query(
                "SELECT id, username, email, created_at, updated_at, last_login_at, is_active, is_admin 
                 FROM sunday_user 
                 WHERE LOWER(username) LIKE $1 OR LOWER(email) LIKE $1 
                 ORDER BY created_at DESC",
                &[&search_pattern],
            ).await
        }
        None => {
            client.query(
                "SELECT id, username, email, created_at, updated_at, last_login_at, is_active, is_admin FROM sunday_user ORDER BY created_at DESC",
                &[],
            ).await
        }
    };

    let rows = rows.map_err(|e| {
        log::error!("Failed to list users: {e:?}");
        ErrorInternalServerError("Database query failed")
    })?;

    let users = rows
        .iter()
        .map(|row| Model {
            id: row.get(0),
            username: row.get(1),
            email: row.get(2),
            created_at: row.get(3),
            updated_at: row.get(4),
            last_login_at: row.get(5),
            is_active: row.get(6),
            is_admin: row.get(7),
        })
        .collect();

    Ok(users)
}

pub async fn delete_by_username(
    username: &str,
    db_client: &DatabaseClient,
) -> Result<(), actix_web::Error> {
    let client = db_client.lock().await;

    client
        .execute("DELETE FROM sunday_user WHERE username = $1", &[&username])
        .await
        .map_err(|_| ErrorInternalServerError("Failed to delete user"))?;

    Ok(())
}

pub async fn update_user(
    user_id: &Uuid,
    username: &str,
    email: &str,
    db_client: &DatabaseClient,
) -> Result<(), actix_web::Error> {
    let client = db_client.lock().await;
    let now = chrono::Utc::now();

    client
        .execute(
            "UPDATE sunday_user SET username = $1, email = $2, updated_at = $3 WHERE id = $4",
            &[&username, &email, &now, &user_id],
        )
        .await
        .map_err(|_| ErrorInternalServerError("Failed to update user"))?;

    Ok(())
}

pub async fn update(user: &Model, db_client: &DatabaseClient) -> Result<(), actix_web::Error> {
    let client = db_client.lock().await;

    client
        .execute(
            "UPDATE sunday_user SET username = $1, email = $2, updated_at = $3 WHERE id = $4",
            &[&user.username, &user.email, &user.updated_at, &user.id],
        )
        .await
        .map_err(|_| ErrorInternalServerError("Failed to update user"))?;

    Ok(())
}

pub async fn toggle_user_active_status(
    user_id: &Uuid,
    db_client: &DatabaseClient,
) -> Result<(), actix_web::Error> {
    let client = db_client.lock().await;
    let now = chrono::Utc::now();

    client
        .execute(
            "UPDATE sunday_user SET is_active = NOT is_active, updated_at = $1 WHERE id = $2",
            &[&now, &user_id],
        )
        .await
        .map_err(|_| ErrorInternalServerError("Failed to toggle user active status"))?;

    Ok(())
}

pub async fn toggle_user_admin_status(
    user_id: &Uuid,
    db_client: &DatabaseClient,
) -> Result<(), actix_web::Error> {
    let client = db_client.lock().await;
    let now = chrono::Utc::now();

    client
        .execute(
            "UPDATE sunday_user SET is_admin = NOT is_admin, updated_at = $1 WHERE id = $2",
            &[&now, &user_id],
        )
        .await
        .map_err(|_| ErrorInternalServerError("Failed to toggle user admin status"))?;

    Ok(())
}

pub async fn delete_by_id(
    user_id: &Uuid,
    db_client: &DatabaseClient,
) -> Result<(), actix_web::Error> {
    let client = db_client.lock().await;

    client
        .execute("DELETE FROM sunday_user WHERE id = $1", &[&user_id])
        .await
        .map_err(|_| ErrorInternalServerError("Failed to delete user"))?;

    Ok(())
}

pub async fn find_by_id(
    user_id: &Uuid,
    db_client: &DatabaseClient,
) -> Result<Option<Model>, actix_web::Error> {
    let client = db_client.lock().await;

    let row = client.query_opt(
        "SELECT id, username, email, created_at, updated_at, last_login_at, is_active, is_admin FROM sunday_user WHERE id = $1",
        &[&user_id],
    ).await
    .map_err(|e| {
        log::error!("Failed to find user by id {user_id}: {e:?}");
        ErrorInternalServerError("Database query failed")
    })?;

    match row {
        Some(row) => {
            let user = Model {
                id: row.get(0),
                username: row.get(1),
                email: row.get(2),
                created_at: row.get(3),
                updated_at: row.get(4),
                last_login_at: row.get(5),
                is_active: row.get(6),
                is_admin: row.get(7),
            };
            Ok(Some(user))
        }
        None => Ok(None),
    }
}

pub async fn create_oidc_user(
    email: &str,
    username: &str,
    _display_name: &str,
    db_client: &DatabaseClient,
) -> Result<Model, actix_web::Error> {
    let client = db_client.lock().await;

    let user_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    client.execute(
        "INSERT INTO sunday_user (id, username, email, created_at, updated_at, last_login_at, is_active, is_admin) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        &[&user_id, &username, &email, &now, &None::<chrono::DateTime<chrono::Utc>>, &None::<chrono::DateTime<chrono::Utc>>, &true, &false],
    ).await
    .map_err(|e| {
        log::error!("Failed to create OIDC user: {e:?}");
        ErrorInternalServerError("Failed to create user")
    })?;

    // Return the newly created user
    let user = Model {
        id: user_id,
        username: username.to_string(),
        email: email.to_string(),
        created_at: now,
        updated_at: None,
        last_login_at: None,
        is_active: true,
        is_admin: false,
    };

    log::info!("Created new OIDC user: {} ({})", email, user_id);
    Ok(user)
}

pub async fn ensure_admin_user(
    admin_username: &str,
    _admin_password: &str, // Keep parameter for compatibility but ignore it
    db_client: &DatabaseClient,
    reset_admin: bool,
) -> Result<(), actix_web::Error> {
    // Check if admin user already exists
    let existing_admin = find_by_username(admin_username, db_client).await?;

    if reset_admin && existing_admin.is_some() {
        log::info!("Resetting admin user: {admin_username}");
        delete_by_username(admin_username, db_client).await?;
        log::info!("Existing admin user deleted");
    }

    // Check again after potential deletion
    let existing_admin = find_by_username(admin_username, db_client).await?;

    if existing_admin.is_none() {
        log::info!("Creating admin user: {admin_username}");

        let admin_email = format!("{admin_username}@admin.local");

        create_admin(admin_username.to_string(), admin_email, db_client).await?;

        log::info!("Admin user created successfully");
    } else {
        log::debug!("Admin user already exists");
    }

    Ok(())
}
