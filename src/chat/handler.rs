use crate::chat::{ChatRequest, ChatResponse};
use crate::models::AiModels;
use rig::completion::Prompt;
use sqlx::PgPool;
use uuid::Uuid;

pub struct Context {
    _session_id: String,
    service_name: String,
    entity_id: String,
}

impl Context {
    pub fn new(session_id: &str, service_name: &str, entity_id: &str) -> Self {
        Context {
            _session_id: session_id.to_owned(),
            service_name: service_name.to_owned(),
            entity_id: entity_id.to_owned(),
        }
    }
}

pub async fn message_async(
    r: ChatRequest,
    context: &Context,
    ai_models: Option<&AiModels>,
    pool: Option<&PgPool>,
    fake: bool,
) -> ChatResponse {
    // Get the latest user message
    let latest_message = r
        .messages
        .iter()
        .rev()
        .find(|msg| msg.role.as_deref() == Some("user") || msg.role.is_none())
        .and_then(|msg| msg.text.clone());

    if fake {
        fake_message(context, latest_message)
    } else if let (Some(models), Some(pool)) = (ai_models, pool) {
        async_real_message(context, latest_message, models, pool).await
    } else {
        panic!("oops")
    }
}

pub fn fake_message(context: &Context, msg: Option<String>) -> ChatResponse {
    let msg = msg.unwrap_or("Nothing".to_string());
    let res = format!(
        "Hello! You said: '{}'. This is {} AI responding",
        msg, context.service_name
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
    pool: &PgPool,
) -> ChatResponse {
    // Get the user message
    let user_message = msg.unwrap_or_else(|| "Hello!".to_string());

    // Get entity UUID from entity_id string
    let entity_uuid = match get_entity_uuid(pool, &context.entity_id).await {
        Ok(Some(uuid)) => uuid,
        Ok(None) => {
            log::warn!(
                "Entity '{}' not found, proceeding without RAG",
                context.entity_id
            );
            // Proceed without RAG context
            return simple_chat(context, &user_message, ai_models).await;
        }
        Err(e) => {
            log::error!("Failed to get entity UUID: {}", e);
            return simple_chat(context, &user_message, ai_models).await;
        }
    };

    // Query relevant documents
    let knowledge_base = match ai_models
        .query_entity_documents(pool, entity_uuid, &user_message, 5)
        .await
    {
        Ok(docs) => docs,
        Err(e) => {
            log::error!("Failed to query entity documents: {}", e);
            Vec::new()
        }
    };

    // Build the system prompt with context
    let mut system_prompt = format!("You are {} AI ", context.service_name);

    if !knowledge_base.is_empty() {
        system_prompt.push_str("Use the following context to answer questions:\n\n");
        for (i, k) in knowledge_base.iter().enumerate() {
            system_prompt.push_str(&format!("{}. {}\n\n", i + 1, k));
        }
        system_prompt.push_str(
            "Answer based on the context provided above. If the context doesn't contain relevant information, say so.\n",
        );
    } else {
        system_prompt.push_str("No specific context is available for this entity yet.\n");
    }

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

async fn simple_chat(context: &Context, message: &str, ai_models: &AiModels) -> ChatResponse {
    let system_prompt = format!("You are {} AI.", context.service_name);

    let agent = ai_models.agent(&system_prompt);

    match agent.prompt(message).await {
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

async fn get_entity_uuid(pool: &PgPool, entity_name: &str) -> Result<Option<Uuid>, sqlx::Error> {
    let result: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM entities WHERE name = $1")
        .bind(entity_name)
        .fetch_optional(pool)
        .await?;

    Ok(result.map(|(id,)| id))
}
