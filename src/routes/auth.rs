use crate::config::{DatabaseClient, Server};
use crate::{db, oidc, view};
use actix_session::{Session, SessionExt};
use actix_web::{HttpRequest, HttpResponse, Result, error::ErrorInternalServerError, web};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct AuthError {
    pub message: String,
}

pub async fn login_form(req: HttpRequest, config: web::Data<Server>) -> Result<HttpResponse> {
    // If LOCAL=true, automatically inject a local user
    if config.local() {
        return inject_local_user(req).await;
    }

    let query = req.query_string();
    let message = if query.contains("error=") {
        let error_msg = query
            .split("error=")
            .nth(1)
            .map(|s| s.split('&').next().unwrap_or("Authentication failed"))
            .and_then(|s| urlencoding::decode(s).ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Authentication failed".to_string());
        Some(error_msg)
    } else {
        None
    };

    let html = view::auth::login_form(message.as_deref(), &config);
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html.into_string()))
}

async fn inject_local_user(req: HttpRequest) -> Result<HttpResponse> {
    let session = req.get_session();
    let db_client = req
        .app_data::<web::Data<DatabaseClient>>()
        .ok_or_else(|| ErrorInternalServerError("Database client not found"))?;

    // Create or find the local development user
    let local_user = match db::user::find_by_email("local@dev.local", db_client).await? {
        Some(user) => user,
        None => {
            log::info!("Creating local development user");
            db::user::create(
                "local".to_string(),
                "local@dev.local".to_string(),
                db_client,
                true, // is_active
            )
            .await?;

            // Make the user an admin
            if let Some(mut user) = db::user::find_by_email("local@dev.local", db_client).await? {
                db::user::toggle_user_admin_status(&user.id, db_client).await?;
                user.is_admin = true; // Update local copy
                user
            } else {
                return Err(ErrorInternalServerError("Failed to create local user"));
            }
        }
    };

    // Update last login
    db::user::update_last_login(&local_user, db_client).await?;

    // Create session
    log::info!(
        "Creating database session for local user: {}",
        local_user.id
    );
    let db_session = db::session::create(local_user.id, &req, db_client)
        .await
        .map_err(|_e| ErrorInternalServerError("Failed to create session"))?;

    session
        .insert("sunday_session_id", &db_session.id)
        .map_err(|_e| ErrorInternalServerError("Session creation failed"))?;

    log::info!("Local user automatically logged in: {}", local_user.email);

    Ok(HttpResponse::SeeOther()
        .append_header(("Location", "/"))
        .finish())
}

pub async fn logout(
    session: Session,
    db_client: web::Data<DatabaseClient>,
) -> Result<HttpResponse> {
    if let Ok(Some(session_id)) = session.get::<String>("sunday_session_id") {
        let db_client = db_client.as_ref();

        match db::session::delete_by_id(&session_id, db_client).await {
            Ok(_delete_result) => {}
            Err(e) => {
                log::warn!("Failed to delete session from database: {e:?}");
            }
        }
    }

    // Clear OIDC user session as well
    oidc::clear_user_from_session(&session);

    session.clear();
    Ok(HttpResponse::SeeOther()
        .append_header(("Location", "/auth/login"))
        .finish())
}

// OIDC Authentication Routes

pub async fn oidc_login(
    session: Session,
    oidc_client: web::Data<oidc::OidcClientArc>,
) -> Result<HttpResponse> {
    let mut client = oidc_client.lock().await;

    // Perform OIDC discovery if not done already
    if let Err(e) = client.discover().await {
        log::error!("OIDC discovery failed: {}", e);
        return Ok(HttpResponse::InternalServerError().json(AuthError {
            message: "Authentication service unavailable".to_string(),
        }));
    }

    let state = Uuid::new_v4().to_string();
    let (code_verifier, code_challenge) = oidc::OidcClient::generate_pkce();

    // Store state and code verifier in session for later verification
    session
        .insert("oidc_state", &state)
        .map_err(|e| ErrorInternalServerError(format!("Session error: {}", e)))?;
    session
        .insert("oidc_code_verifier", &code_verifier)
        .map_err(|e| ErrorInternalServerError(format!("Session error: {}", e)))?;

    let auth_url = client
        .build_auth_url(&state, &code_challenge, None)
        .map_err(|e| ErrorInternalServerError(format!("Failed to build auth URL: {}", e)))?;

    log::debug!("Redirecting to OIDC auth URL: {}", auth_url);

    Ok(HttpResponse::SeeOther()
        .append_header(("Location", auth_url))
        .finish())
}

pub async fn oidc_callback(
    req: HttpRequest,
    session: Session,
    oidc_client: web::Data<oidc::OidcClientArc>,
    db_client: web::Data<DatabaseClient>,
) -> Result<HttpResponse> {
    let query = req.query_string();
    let params: std::collections::HashMap<String, String> =
        url::form_urlencoded::parse(query.as_bytes())
            .into_owned()
            .collect();

    // Check for error parameter
    if let Some(error) = params.get("error") {
        log::warn!("OIDC callback error: {}", error);
        let error_description = params
            .get("error_description")
            .map(|s| s.as_str())
            .unwrap_or("Authentication failed");
        return Ok(HttpResponse::SeeOther()
            .append_header((
                "Location",
                format!(
                    "/auth/login?error={}",
                    urlencoding::encode(error_description)
                ),
            ))
            .finish());
    }

    // Extract authorization code and state
    let code = params
        .get("code")
        .ok_or_else(|| ErrorInternalServerError("Missing authorization code"))?;
    let returned_state = params
        .get("state")
        .ok_or_else(|| ErrorInternalServerError("Missing state parameter"))?;

    // Verify state parameter
    let stored_state: String = session
        .get("oidc_state")
        .map_err(|e| ErrorInternalServerError(format!("Session error: {}", e)))?
        .ok_or_else(|| ErrorInternalServerError("No stored state found"))?;

    if returned_state != &stored_state {
        log::warn!("State mismatch in OIDC callback");
        return Ok(HttpResponse::BadRequest().json(AuthError {
            message: "Invalid state parameter".to_string(),
        }));
    }

    // Get code verifier from session
    let code_verifier: String = session
        .get("oidc_code_verifier")
        .map_err(|e| ErrorInternalServerError(format!("Session error: {}", e)))?
        .ok_or_else(|| ErrorInternalServerError("No code verifier found"))?;

    let client = oidc_client.lock().await;

    // Exchange authorization code for tokens
    let token_response = client
        .exchange_code(code, &code_verifier)
        .await
        .map_err(|e| ErrorInternalServerError(format!("Token exchange failed: {}", e)))?;

    // Get user info
    let user_info = client
        .get_user_info(&token_response.access_token)
        .await
        .map_err(|e| ErrorInternalServerError(format!("Failed to get user info: {}", e)))?;

    log::info!(
        "OIDC authentication successful for user: {} ({})",
        user_info.email,
        user_info.sub
    );

    // Store user info in session
    oidc::store_user_in_session(&session, &user_info)?;

    // Find or create user in database
    let user_model = match db::user::find_by_email(&user_info.email, db_client.as_ref()).await? {
        Some(existing_user) => {
            log::debug!("Found existing user: {}", existing_user.id);
            // Update last login
            db::user::update_last_login(&existing_user, db_client.as_ref()).await?;
            existing_user
        }
        None => {
            log::info!("Creating new user for OIDC user: {}", user_info.email);
            // Create new user
            let username = user_info
                .email
                .split('@')
                .next()
                .unwrap_or(&user_info.email)
                .to_string();
            let display_name = user_info.name.clone().unwrap_or_else(|| {
                format!(
                    "{} {}",
                    user_info
                        .given_name
                        .clone()
                        .unwrap_or_else(|| "".to_string()),
                    user_info
                        .family_name
                        .clone()
                        .unwrap_or_else(|| "".to_string())
                )
                .trim()
                .to_string()
            });

            db::user::create_oidc_user(
                &user_info.email,
                &username,
                &display_name,
                db_client.as_ref(),
            )
            .await
            .map_err(|e| ErrorInternalServerError(format!("Failed to create user: {}", e)))?
        }
    };

    // Create session
    let db_session = db::session::create(user_model.id, &req, db_client.as_ref())
        .await
        .map_err(|e| ErrorInternalServerError(format!("Failed to create session: {}", e)))?;

    session
        .insert("sunday_session_id", &db_session.id)
        .map_err(|e| ErrorInternalServerError(format!("Session creation failed: {}", e)))?;

    // Clean up OIDC session data
    session.remove("oidc_state");
    session.remove("oidc_code_verifier");

    Ok(HttpResponse::SeeOther()
        .append_header(("Location", "/"))
        .finish())
}
