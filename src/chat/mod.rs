use crate::api_key_middleware::AuthState;
use crate::config::Server;
use crate::models::AiModels;
use actix_web::{HttpResponse, Result as AwResult, post, web};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

mod handler;

#[derive(Serialize)]
pub struct ChatResponse {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ChatRequest {
    pub messages: Vec<MessageContent>,
    pub entity_id: String,
}

#[derive(Deserialize, Debug)]
pub struct MessageContent {
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
}

#[post("/chat")]
pub async fn chat_endpoint(
    // just check if not how caller is authenticated
    _auth_state: AuthState,
    server: web::Data<Server>,
    ai_models: Option<web::Data<AiModels>>,
    pool: Option<web::Data<PgPool>>,
    body: web::Json<ChatRequest>,
) -> AwResult<HttpResponse> {
    let entity_id = &body.entity_id;

    // Validate entity_id is not empty
    if entity_id.is_empty() {
        log::error!("Attempted to access chat without entity_id");
        return Ok(HttpResponse::BadRequest().json(ChatResponse {
            text: String::new(),
            error: Some("entity_id is required".to_string()),
        }));
    }

    let context = handler::Context::new("", server.sunday_name(), entity_id);

    let res = handler::message_async(
        body.0,
        &context,
        ai_models.as_ref().map(|m| m.as_ref()),
        pool.as_ref().map(|p| p.as_ref()),
        server.fake_ai(),
    )
    .await;

    Ok(HttpResponse::Ok().json(ChatResponse {
        text: res.text,
        error: res.error,
    }))
}
