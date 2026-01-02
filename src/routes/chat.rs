use crate::config::Server;
use crate::routes::get_user;
use actix_web::{HttpRequest, HttpResponse, Result as AwResult, Scope, post, web};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct MessageContent {
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub files: Option<Vec<FileContent>>,
}

#[derive(Deserialize, Debug)]
pub struct FileContent {
    pub name: String,
    #[serde(default)]
    pub src: Option<String>,
    #[serde(default)]
    pub type_field: Option<String>,
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
async fn chat_endpoint(
    _tenant: web::Path<String>,
    server: web::Data<Server>,
    req: HttpRequest,
    body: web::Json<ChatRequest>,
) -> AwResult<HttpResponse> {
    // Check if user is authenticated
    let user = match get_user(req) {
        Some(user) => user,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ChatResponse {
                text: String::new(),
                error: Some("Authentication required".to_string()),
            }));
        }
    };

    // Check if user has tenant context
    let tenant = match user.tenant() {
        Some(t) => t,
        None => {
            log::error!(
                "User {} attempted to access chat without tenant context",
                user.email()
            );
            return Ok(HttpResponse::Forbidden().json(ChatResponse {
                text: String::new(),
                error: Some("Tenant access required".to_string()),
            }));
        }
    };

    log::info!(
        "Chat request from user: {} in tenant: {}",
        user.email(),
        tenant
    );
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
                "Hello {}! You said: '{}'. This is {} AI responding to your message. In the future, this will be connected to a proper AI service.",
                user.email(),
                user_text,
                server.sunday_name()
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

pub fn scope() -> Scope {
    web::scope("/api").service(chat_endpoint)
}
