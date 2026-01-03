use crate::chat::{ChatRequest, ChatResponse};
use crate::models::AiModels;
use rig::completion::Prompt;

pub struct Context {
    _session_id: String,
    service_name: String,
    tenant: String,
    knowledge_base: Vec<String>,
}

impl Context {
    pub fn new(
        session_id: &str,
        service_name: &str,
        tenant: &str,
        knowledge_base: Vec<String>,
    ) -> Self {
        Context {
            _session_id: session_id.to_owned(),
            service_name: service_name.to_owned(),
            tenant: tenant.to_owned(),
            knowledge_base,
        }
    }
}

pub fn message(
    r: ChatRequest,
    context: &Context,
    ai_models: Option<&AiModels>,
    fake: bool,
) -> ChatResponse {
    // Get the latest user message
    let latest_message = r
        .messages
        .iter()
        .rev()
        .find(|msg| msg.role.as_deref() == Some("user") || msg.role.is_none())
        .and_then(|msg| msg.text.clone());

    if fake || ai_models.is_none() {
        fake_message(context, latest_message)
    } else {
        // Use async runtime for real message handling
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                // We're in an async context, use block_in_place
                tokio::task::block_in_place(|| {
                    handle.block_on(async_real_message(
                        context,
                        latest_message,
                        ai_models.unwrap(),
                    ))
                })
            }
            Err(_) => {
                // No runtime, fall back to fake message
                log::warn!("No tokio runtime available for LLM call, falling back to fake message");
                fake_message(context, None)
            }
        }
    }
}

pub fn fake_message(context: &Context, msg: Option<String>) -> ChatResponse {
    let msg = msg.unwrap_or("Nothing".to_string());
    let res = format!(
        "Hello! You said: '{}'. This is {} AI responding to your message in tenant '{}'. In the future, this will be connected to a proper AI service.",
        msg, context.service_name, context.tenant
    );
    ChatResponse {
        text: res,
        error: None,
    }
}

async fn async_real_message(
    context: &Context,
    msg: Option<String>,
    ai_models: &AiModels,
) -> ChatResponse {
    // Build the system prompt with context
    let mut system_prompt = format!(
        "You are {} AI, an assistant for tenant '{}'. ",
        context.service_name, context.tenant
    );

    if !context.knowledge_base.is_empty() {
        system_prompt.push_str("Use the following context to answer questions:\n");
        for (i, k) in context.knowledge_base.iter().enumerate() {
            system_prompt.push_str(&format!("{}. {}\n", i + 1, k));
        }
    }

    // Get the user message
    let user_message = msg.unwrap_or_else(|| "Hello!".to_string());

    // Create an agent with the system prompt
    let agent = ai_models.agent(&system_prompt);

    // Send prompt and get response
    match agent.prompt(&user_message).await {
        Ok(response) => ChatResponse {
            text: response,
            error: None,
        },
        Err(e) => {
            log::error!("Failed to get LLM response: {}", e);
            ChatResponse {
                text: String::new(),
                error: Some(format!("Failed to get AI response: {}", e)),
            }
        }
    }
}
