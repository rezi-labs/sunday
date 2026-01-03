use crate::chat::{ChatRequest, ChatResponse};

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

pub fn message(r: ChatRequest, context: &Context, fake: bool) -> ChatResponse {
    // Get the latest user message
    let latest_message = r
        .messages
        .iter()
        .rev()
        .find(|msg| msg.role.as_deref() == Some("user") || msg.role.is_none())
        .and_then(|msg| msg.text.clone());

    if fake {
        fake_message(context, latest_message)
    } else {
        real_message(context, latest_message)
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

pub fn real_message(context: &Context, msg: Option<String>) -> ChatResponse {
    let context_with_prios: String = context
        .knowledge_base
        .iter()
        .enumerate()
        .map(|(i, k)| format!("Priority: {i} Context: {}", k))
        .collect::<Vec<String>>()
        .join(", ");

    // todo actually send to llm
    let _send_to_llm = match msg {
        Some(msg) => {
            format!(
                "The user sent: {msg}, answer it with this context: {}, you are {} AI responding to your message in tenant '{}'",
                context_with_prios, context.service_name, context.tenant
            )
        }
        None => {
            format!(
                "No message provided, try with this context: {}, you are {} AI responding to your message in tenant '{}'",
                context_with_prios, context.service_name, context.tenant
            )
        }
    };

    // todo send to llm

    // todo replace with real message
    fake_message(context, None)
}
