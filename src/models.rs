use crate::config::AzureOpenAIConfig;
use rig::client::{CompletionClient, ProviderClient};
use rig::providers::azure;

/// Container for Azure OpenAI client and deployment names
pub struct AiModels {
    client: azure::Client,
    pub deployment_name: String,
}

impl AiModels {
    /// Create AI models from Azure OpenAI configuration
    pub fn from_azure_config(config: &AzureOpenAIConfig) -> Result<Self, String> {
        // Set environment variables for Azure OpenAI
        unsafe {
            std::env::set_var("AZURE_API_KEY", &config.api_key);
            std::env::set_var("AZURE_API_VERSION", &config.api_version);
            std::env::set_var("AZURE_ENDPOINT", &config.endpoint);
        }

        // Create Azure OpenAI client from environment
        let client: azure::Client = ProviderClient::from_env();

        Ok(AiModels {
            client,
            deployment_name: config.deployment_name.clone(),
        })
    }

    /// Create an agent for chat completion
    pub fn agent(&self, preamble: &str) -> rig::agent::Agent<azure::CompletionModel> {
        self.client
            .agent(&self.deployment_name)
            .preamble(preamble)
            .build()
    }
}
