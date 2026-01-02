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
async fn chat_endpoint(req: HttpRequest, body: web::Json<ChatRequest>) -> AwResult<HttpResponse> {
    // Check if user is authenticated
    let user = get_user(req);
    if user.is_none() {
        return Ok(HttpResponse::Unauthorized().json(ChatResponse {
            text: String::new(),
            error: Some("Authentication required".to_string()),
        }));
    }

    let user = user.unwrap();
    log::info!("Chat request from user: {}", user.email());
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
            format!("Hello {}! You said: '{}'. This is Sunday AI responding to your message. In the future, this will be connected to a proper AI service.", 
                user.email(),
                user_text
            )
        }
        None => {
            "Hello! I'm Sunday AI. I didn't receive any text in your message. Could you please try again?".to_string()
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
