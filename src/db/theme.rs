use sqlx::{PgPool, Row};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Theme {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_default: bool,
    pub is_custom: bool,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct ThemeColor {
    pub id: Uuid,
    pub theme_id: Uuid,
    pub color_key: String,
    pub color_value: String,
    pub color_category: String,
}

#[derive(Debug, Clone)]
pub struct ThemeWithColors {
    pub theme: Theme,
    pub colors: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ThemeCssCache {
    pub id: Uuid,
    pub css_content: String,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub is_current: bool,
}

#[derive(Debug, Clone)]
pub struct SystemThemeSettings {
    pub id: Uuid,
    pub setting_key: String,
    pub theme_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Theme {
    pub async fn get_all(pool: &PgPool) -> Result<Vec<Theme>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, name, display_name, description, is_default, is_custom, is_active, 
                    created_by, created_at, updated_at 
             FROM themes 
             WHERE is_active = true 
             ORDER BY is_default DESC, display_name ASC",
        )
        .fetch_all(pool)
        .await?;

        let mut themes = Vec::new();
        for row in rows {
            themes.push(Theme {
                id: row.get("id"),
                name: row.get("name"),
                display_name: row.get("display_name"),
                description: row.get("description"),
                is_default: row.get("is_default"),
                is_custom: row.get("is_custom"),
                is_active: row.get("is_active"),
                created_by: row.get("created_by"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
        Ok(themes)
    }

    pub async fn get_by_name(pool: &PgPool, name: &str) -> Result<Option<Theme>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, name, display_name, description, is_default, is_custom, is_active, 
                    created_by, created_at, updated_at 
             FROM themes 
             WHERE name = $1 AND is_active = true",
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(Theme {
                id: row.get("id"),
                name: row.get("name"),
                display_name: row.get("display_name"),
                description: row.get("description"),
                is_default: row.get("is_default"),
                is_custom: row.get("is_custom"),
                is_active: row.get("is_active"),
                created_by: row.get("created_by"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_with_colors(
        pool: &PgPool,
        name: &str,
    ) -> Result<Option<ThemeWithColors>, sqlx::Error> {
        let theme = Self::get_by_name(pool, name).await?;

        if let Some(theme) = theme {
            let colors = ThemeColor::get_by_theme_id(pool, theme.id).await?;
            let colors_map = colors
                .into_iter()
                .map(|color| (color.color_key, color.color_value))
                .collect();

            Ok(Some(ThemeWithColors {
                theme,
                colors: colors_map,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_all_with_colors(pool: &PgPool) -> Result<Vec<ThemeWithColors>, sqlx::Error> {
        let themes = Self::get_all(pool).await?;
        let mut themes_with_colors = Vec::new();

        for theme in themes {
            let colors = ThemeColor::get_by_theme_id(pool, theme.id).await?;
            let colors_map = colors
                .into_iter()
                .map(|color| (color.color_key, color.color_value))
                .collect();

            themes_with_colors.push(ThemeWithColors {
                theme,
                colors: colors_map,
            });
        }

        Ok(themes_with_colors)
    }

    pub async fn create(
        pool: &PgPool,
        name: String,
        display_name: String,
        description: Option<String>,
        created_by: Option<Uuid>,
    ) -> Result<Theme, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        sqlx::query(
            "INSERT INTO themes (id, name, display_name, description, is_custom, created_by, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, true, $5, $6, $7)"
        )
        .bind(id)
        .bind(&name)
        .bind(&display_name)
        .bind(&description)
        .bind(created_by)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(Theme {
            id,
            name,
            display_name,
            description,
            is_default: false,
            is_custom: true,
            is_active: true,
            created_by,
            created_at: now,
            updated_at: now,
        })
    }
}

impl ThemeColor {
    pub async fn get_by_theme_id(
        pool: &PgPool,
        theme_id: Uuid,
    ) -> Result<Vec<ThemeColor>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, theme_id, color_key, color_value, color_category 
             FROM theme_colors 
             WHERE theme_id = $1 
             ORDER BY color_category, color_key",
        )
        .bind(theme_id)
        .fetch_all(pool)
        .await?;

        let mut colors = Vec::new();
        for row in rows {
            colors.push(ThemeColor {
                id: row.get("id"),
                theme_id: row.get("theme_id"),
                color_key: row.get("color_key"),
                color_value: row.get("color_value"),
                color_category: row.get("color_category"),
            });
        }
        Ok(colors)
    }

    pub async fn set_colors(
        pool: &PgPool,
        theme_id: Uuid,
        colors: HashMap<String, String>,
    ) -> Result<(), sqlx::Error> {
        // Delete existing colors for this theme
        sqlx::query("DELETE FROM theme_colors WHERE theme_id = $1")
            .bind(theme_id)
            .execute(pool)
            .await?;

        // Insert new colors
        for (color_key, color_value) in colors {
            let category = get_color_category(&color_key);
            let id = Uuid::new_v4();
            let now = chrono::Utc::now();

            sqlx::query(
                "INSERT INTO theme_colors (id, theme_id, color_key, color_value, color_category, created_at, updated_at) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7)"
            )
            .bind(id)
            .bind(theme_id)
            .bind(color_key)
            .bind(color_value)
            .bind(category)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await?;
        }

        Ok(())
    }
}

impl SystemThemeSettings {
    pub async fn get_theme_for_mode(
        pool: &PgPool,
        is_dark_mode: bool,
    ) -> Result<String, sqlx::Error> {
        let setting_key = if is_dark_mode {
            "default_dark_theme"
        } else {
            "default_light_theme"
        };

        let row =
            sqlx::query("SELECT theme_name FROM system_theme_settings WHERE setting_key = $1")
                .bind(setting_key)
                .fetch_optional(pool)
                .await?;

        Ok(row
            .map(|row| row.get::<String, _>("theme_name"))
            .unwrap_or_else(|| {
                if is_dark_mode {
                    "swiss-dark".to_string()
                } else {
                    "swiss".to_string()
                }
            }))
    }

    pub async fn get_default_light_theme(pool: &PgPool) -> Result<String, sqlx::Error> {
        Self::get_theme_for_mode(pool, false).await
    }

    pub async fn get_default_dark_theme(pool: &PgPool) -> Result<String, sqlx::Error> {
        Self::get_theme_for_mode(pool, true).await
    }

    pub async fn set_default_light_theme(
        pool: &PgPool,
        theme_name: String,
    ) -> Result<(), sqlx::Error> {
        Self::set_theme_setting(pool, "default_light_theme", theme_name).await
    }

    pub async fn set_default_dark_theme(
        pool: &PgPool,
        theme_name: String,
    ) -> Result<(), sqlx::Error> {
        Self::set_theme_setting(pool, "default_dark_theme", theme_name).await
    }

    async fn set_theme_setting(
        pool: &PgPool,
        setting_key: &str,
        theme_name: String,
    ) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now();

        sqlx::query(
            "INSERT INTO system_theme_settings (setting_key, theme_name, updated_at) 
             VALUES ($1, $2, $3)
             ON CONFLICT (setting_key) 
             DO UPDATE SET theme_name = $2, updated_at = $3",
        )
        .bind(setting_key)
        .bind(theme_name)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_all_settings(pool: &PgPool) -> Result<Vec<SystemThemeSettings>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, setting_key, theme_name, created_at, updated_at 
             FROM system_theme_settings 
             ORDER BY setting_key",
        )
        .fetch_all(pool)
        .await?;

        let mut settings = Vec::new();
        for row in rows {
            settings.push(SystemThemeSettings {
                id: row.get("id"),
                setting_key: row.get("setting_key"),
                theme_name: row.get("theme_name"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
        Ok(settings)
    }
}

impl ThemeCssCache {
    pub async fn get_current_css(pool: &PgPool) -> Result<Option<String>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT css_content FROM theme_css_cache WHERE is_current = true ORDER BY generated_at DESC LIMIT 1"
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|row| row.get::<String, _>("css_content")))
    }

    pub async fn save_css(pool: &PgPool, css_content: String) -> Result<(), sqlx::Error> {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        // Begin transaction
        let mut tx = pool.begin().await?;

        // Mark all existing CSS as non-current
        sqlx::query("UPDATE theme_css_cache SET is_current = false")
            .execute(&mut *tx)
            .await?;

        // Insert new current CSS
        sqlx::query(
            "INSERT INTO theme_css_cache (id, css_content, generated_at, is_current) 
             VALUES ($1, $2, $3, true)",
        )
        .bind(id)
        .bind(css_content)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        // Clean up old entries (keep only last 5)
        sqlx::query(
            "DELETE FROM theme_css_cache 
             WHERE id NOT IN (
                 SELECT id FROM theme_css_cache 
                 ORDER BY generated_at DESC 
                 LIMIT 5
             )",
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}

fn get_color_category(color_key: &str) -> String {
    match color_key {
        key if key.starts_with("base") => "base".to_string(),
        key if key.starts_with("primary") => "primary".to_string(),
        key if key.starts_with("secondary") => "secondary".to_string(),
        key if key.starts_with("accent") => "accent".to_string(),
        key if key.starts_with("neutral") => "neutral".to_string(),
        key if key.starts_with("info") => "info".to_string(),
        key if key.starts_with("success") => "success".to_string(),
        key if key.starts_with("warning") => "warning".to_string(),
        key if key.starts_with("error") => "error".to_string(),
        _ => "base".to_string(),
    }
}

/// CSS Generation for themes
pub struct ThemeCssGenerator;

impl ThemeCssGenerator {
    pub fn generate_css(themes: &[ThemeWithColors]) -> String {
        let mut css = String::new();

        // Add CSS header comment
        css.push_str("/* Auto-generated theme CSS from database */\n");
        css.push_str("/* Do not edit this file directly - manage themes via admin console */\n\n");

        for theme_with_colors in themes {
            let theme = &theme_with_colors.theme;
            let colors = &theme_with_colors.colors;

            css.push_str(&format!("[data-theme=\"{}\"] {{\n", theme.name));

            // Add color-scheme property based on theme name
            if theme.name.contains("dark") {
                css.push_str("  color-scheme: dark;\n");
            } else {
                css.push_str("  color-scheme: light;\n");
            }

            // Sort colors for consistent output
            let mut sorted_colors: Vec<_> = colors.iter().collect();
            sorted_colors.sort_by_key(|(key, _)| *key);

            for (color_key, color_value) in sorted_colors {
                css.push_str(&format!("  --color-{}: {};\n", color_key, color_value));
            }

            css.push_str("}\n\n");
        }

        // Add the Swiss design system overrides
        css.push_str(
            r#"/* Swiss Design System Overrides */
:root {
  --rounded-box: 0.25rem;
  --rounded-btn: 0.25rem;
  --rounded-badge: 0.25rem;
  --tab-radius: 0.25rem;
}

/* Enhanced contrast themes */
[data-theme="swiss"],
[data-theme="swiss-dark"] {
  --border-btn: 2px;
  --tab-border: 2px;
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
"#,
        );

        css
    }

    pub async fn generate_and_cache_css(
        pool: &PgPool,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let themes = Theme::get_all_with_colors(pool).await?;
        let css = Self::generate_css(&themes);

        // Save CSS to database cache
        ThemeCssCache::save_css(pool, css.clone()).await?;

        Ok(css)
    }

    pub async fn get_or_generate_css(pool: &PgPool) -> Result<String, Box<dyn std::error::Error>> {
        // Try to get cached CSS first
        if let Some(css) = ThemeCssCache::get_current_css(pool).await? {
            if !css.is_empty() && css != "/* CSS will be generated from database themes */" {
                return Ok(css);
            }
        }

        // If no valid cached CSS, generate new one
        Self::generate_and_cache_css(pool).await
    }
}
