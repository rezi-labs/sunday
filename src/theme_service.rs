use reqwest;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

// Cache for CSS to avoid hitting external API on every request
#[derive(Debug, Clone)]
struct CssCache {
    css: String,
    last_updated: std::time::Instant,
}

pub struct ThemeService {
    client: reqwest::Client,
    external_api_url: Option<String>,
    fallback_css: String,
    cache: Arc<RwLock<Option<CssCache>>>,
    cache_duration: Duration,
}

impl ThemeService {
    pub fn new(external_api_url: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        let fallback_css = Self::get_fallback_css();
        let cache_duration = Duration::from_secs(300); // 5 minutes

        Self {
            client,
            external_api_url,
            fallback_css,
            cache: Arc::new(RwLock::new(None)),
            cache_duration,
        }
    }

    /// Get CSS (from cache if fresh, otherwise fetch from external API)
    pub async fn get_css(&self) -> String {
        // Check cache first
        {
            let cache_read = self.cache.read().await;
            if let Some(cache) = cache_read.as_ref() {
                if cache.last_updated.elapsed() < self.cache_duration {
                    log::debug!("Theme service: Returning cached CSS");
                    return cache.css.clone();
                }
            }
        }

        // Cache is empty or stale, fetch from external API
        match &self.external_api_url {
            Some(url) => match self.fetch_css_from_external_api(url).await {
                Ok(css) => {
                    log::info!("Successfully fetched CSS from external API");
                    self.update_cache(css.clone()).await;
                    css
                }
                Err(e) => {
                    log::warn!(
                        "Failed to fetch CSS from external API: {}, using fallback",
                        e
                    );
                    self.update_cache(self.fallback_css.clone()).await;
                    self.fallback_css.clone()
                }
            },
            None => {
                log::debug!("No external theme API URL configured, using fallback CSS");
                self.update_cache(self.fallback_css.clone()).await;
                self.fallback_css.clone()
            }
        }
    }

    /// Update the internal cache with CSS
    async fn update_cache(&self, css: String) {
        let cache = CssCache {
            css,
            last_updated: std::time::Instant::now(),
        };

        let mut cache_write = self.cache.write().await;
        *cache_write = Some(cache);
        log::debug!("Theme service: CSS cache updated");
    }

    /// Fetch CSS directly from the external API
    async fn fetch_css_from_external_api(
        &self,
        url: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        log::info!("Fetching CSS from external API: {}", url);

        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(format!("External API returned status: {}", response.status()).into());
        }

        let css = response.text().await?;
        Ok(css)
    }

    /// Get fallback CSS (Swiss theme)
    fn get_fallback_css() -> String {
        r#"/* Fallback Swiss Theme CSS */
:root {
    --rounded-box: 0.25rem;
    --rounded-btn: 0.25rem;
    --rounded-badge: 0.25rem;
    --tab-radius: 0.25rem;
}

[data-theme="swiss"] {
    color-scheme: light;
    --color-base-100: oklch(97% 0.014 343.198);
    --color-base-200: oklch(89% 0.026 342.55);
    --color-base-300: oklch(80% 0.037 342.176);
    --color-base-content: oklch(25% 0.027 342.334);
    --color-primary: oklch(65% 0.241 354.308);
    --color-primary-content: oklch(89% 0.026 342.55);
    --color-secondary: oklch(62% 0.265 303.9);
    --color-secondary-content: oklch(89% 0.026 342.55);
    --color-accent: oklch(76% 0.184 183.61);
    --color-accent-content: oklch(15% 0.042 183.61);
    --color-neutral: oklch(25% 0.027 342.334);
    --color-neutral-content: oklch(89% 0.026 342.55);
    --color-info: oklch(63% 0.181 231.729);
    --color-info-content: oklch(15% 0.042 231.729);
    --color-success: oklch(64% 0.155 160.463);
    --color-success-content: oklch(15% 0.042 160.463);
    --color-warning: oklch(75% 0.166 83.618);
    --color-warning-content: oklch(15% 0.042 83.618);
    --color-error: oklch(61% 0.204 22.207);
    --color-error-content: oklch(89% 0.026 342.55);
}

/* Override DaisyUI rounded corners for angular design */
.btn {
    border-radius: 0.25rem !important;
    border-width: 2px;
}

.card {
    border-radius: 0.25rem !important;
}

.input,
.textarea,
.select {
    border-radius: 0.25rem !important;
    border-width: 2px;
}

.badge {
    border-radius: 0.25rem !important;
}
"#
        .to_string()
    }

    /// Force refresh of cache (useful for debugging)
    pub async fn refresh_cache(&self) {
        let mut cache_write = self.cache.write().await;
        *cache_write = None;
        drop(cache_write);

        // Trigger a fetch
        self.get_css().await;
        log::info!("Theme service: CSS cache forcefully refreshed");
    }
}
