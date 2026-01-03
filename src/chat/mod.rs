use crate::api_key_middleware::AuthState;
use crate::config::Server;
use crate::tenant_middleware::get_tenant_from_path;
use actix_web::{HttpRequest, HttpResponse, Result as AwResult, post, web};
use serde::{Deserialize, Serialize};

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
    req: HttpRequest,
    body: web::Json<ChatRequest>,
) -> AwResult<HttpResponse> {
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

    let context = handler::Context::new("", server.sunday_name(), &tenant, Vec::new());

    let res = handler::message(body.0, &context, server.fake_ai());

    Ok(HttpResponse::Ok().json(ChatResponse {
        text: res.text,
        error: res.error,
    }))
}
