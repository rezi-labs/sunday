use crate::api_key_middleware::AuthState;
use crate::config::Server;
use crate::tenant_middleware::get_tenant_from_path;
use actix_web::{HttpRequest, HttpResponse, Result as AwResult, post, web};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct MessageContent {
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ChatRequest {
    pub messages: Vec<MessageContent>,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[post("/chat")]
pub async fn chat_endpoint(
    auth_state: AuthState,
    server: web::Data<Server>,
    req: HttpRequest,
    body: web::Json<ChatRequest>,
) -> AwResult<HttpResponse> {
    // Auth is enforced by AuthState extractor - if we reach here, we're authenticated
    log::debug!("Authenticated chat request: {:?}", auth_state);

    // Get tenant from URL path
    let tenant = match get_tenant_from_path(&req) {
        Some(t) => t,
        None => {
            log::error!("Attempted to access chat without tenant in URL");
            return Ok(HttpResponse::BadRequest().json(ChatResponse {
                text: String::new(),
                error: Some("Invalid tenant in URL".to_string()),
            }));
        }
    };

    log::info!("Chat request for tenant: {}", tenant);
    log::debug!("Chat request body: {:?}", body.messages);

    // Get the latest user message
    let latest_message = body
        .messages
        .iter()
        .rev()
        .find(|msg| msg.role.as_deref() == Some("user") || msg.role.is_none())
        .and_then(|msg| msg.text.as_ref());

    let response_text = match latest_message {
        Some(user_text) => {
            // Simple echo response for now - you can integrate with AI services here
            format!(
                "Hello! You said: '{}'. This is {} AI responding to your message in tenant '{}'. In the future, this will be connected to a proper AI service.",
                user_text,
                server.sunday_name(),
                tenant
            )
        }
        None => {
            format!(
                "Hello! I'm {} AI. I didn't receive any text in your message. Could you please try again?",
                server.sunday_name()
            )
        }
    };

    Ok(HttpResponse::Ok().json(ChatResponse {
        text: response_text,
        error: None,
    }))
}
