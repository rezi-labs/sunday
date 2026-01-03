use std::env;

#[derive(Clone)]
pub struct Server {
    port: u16,
    host: String,
    local: bool,
    fake_ai: bool,
    sunday_name: String,
    // API key authentication
    api_keys: Vec<String>,
}

impl Server {
    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn host(&self) -> String {
        self.host.clone()
    }

    pub fn local(&self) -> bool {
        self.local
    }

    pub fn fake_ai(&self) -> bool {
        self.fake_ai
    }

    pub fn sunday_name(&self) -> &str {
        &self.sunday_name
    }

    #[allow(dead_code)]
    pub fn api_key(&self) -> &str {
        // Return first API key for backward compatibility
        self.api_keys
            .first()
            .map(|s| s.as_str())
            .unwrap_or_default()
    }

    pub fn api_keys(&self) -> &[String] {
        &self.api_keys
    }
}

pub fn from_env() -> Server {
    let local = env::var("LOCAL").unwrap_or("false".to_string());
    let local = local == "true";

    let fake_ai = env::var("FAKE_AI").unwrap_or("false".to_string());
    let fake_ai = fake_ai == "true";

    let port: u16 = env::var("PORT")
        .or_else(|_| env::var("g_port"))
        .map(|e| e.parse().expect("could not parse port"))
        .unwrap_or(3000);

    let host: String = env::var("HOST")
        .or_else(|_| env::var("g_host"))
        .map(|e| e.parse().expect("could not parse host"))
        .unwrap_or("0.0.0.0".to_string());

    // Sunday name (with fallback)
    let sunday_name = env::var("SUNDAY_NAME").unwrap_or_else(|_| "Sunday".to_string());

    // API key authentication - support multiple keys
    let api_key_one = env::var("API_KEY_ONE").expect("API_KEY_ONE must be set for authentication");
    let api_key_two = env::var("API_KEY_TWO").expect("API_KEY_TWO must be set for authentication");

    let api_keys = vec![api_key_one, api_key_two];

    Server {
        port,
        host,
        local,
        fake_ai,
        sunday_name,
        api_keys,
    }
}
