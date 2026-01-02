-- Theme management schema
-- This migration adds support for dynamic theme creation and management

-- Add theme preference to users
ALTER TABLE sunday_user ADD COLUMN theme_preference VARCHAR DEFAULT 'swiss';
ALTER TABLE sunday_user ADD COLUMN custom_theme_id UUID;

-- Create themes table
CREATE TABLE IF NOT EXISTS themes (
    id UUID PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    display_name VARCHAR NOT NULL,
    description VARCHAR,
    is_default BOOLEAN DEFAULT false,
    is_custom BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    created_by UUID REFERENCES sunday_user(id),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Create theme_colors table for storing color definitions
CREATE TABLE IF NOT EXISTS theme_colors (
    id UUID PRIMARY KEY,
    theme_id UUID NOT NULL REFERENCES themes(id) ON DELETE CASCADE,
    color_key VARCHAR NOT NULL, -- 'base-100', 'primary', 'secondary', etc.
    color_value VARCHAR NOT NULL, -- OKLCH color value
    color_category VARCHAR NOT NULL DEFAULT 'base', -- 'base', 'primary', 'secondary', etc.
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT unique_theme_color UNIQUE(theme_id, color_key)
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_themes_name ON themes(name);
CREATE INDEX IF NOT EXISTS idx_themes_is_active ON themes(is_active);
CREATE INDEX IF NOT EXISTS idx_themes_is_default ON themes(is_default);
CREATE INDEX IF NOT EXISTS idx_theme_colors_theme_id ON theme_colors(theme_id);
CREATE INDEX IF NOT EXISTS idx_theme_colors_category ON theme_colors(color_category);
CREATE INDEX IF NOT EXISTS idx_sunday_user_theme_preference ON sunday_user(theme_preference);

-- Add foreign key constraint for custom_theme_id
ALTER TABLE sunday_user ADD CONSTRAINT fk_sunday_user_custom_theme_id 
    FOREIGN KEY (custom_theme_id) REFERENCES themes(id) ON DELETE SET NULL;

-- Insert default themes (Swiss light and dark)
INSERT INTO themes (id, name, display_name, description, is_default, is_custom, is_active) VALUES 
(gen_random_uuid(), 'swiss', 'Swiss Light', 'Clean, minimal light theme inspired by Swiss design principles', true, false, true),
(gen_random_uuid(), 'swiss-dark', 'Swiss Dark', 'Clean, minimal dark theme inspired by Swiss design principles', false, false, true);

-- Insert default theme colors for Swiss Light
WITH swiss_theme AS (SELECT id FROM themes WHERE name = 'swiss')
INSERT INTO theme_colors (id, theme_id, color_key, color_value, color_category) 
SELECT 
    gen_random_uuid(),
    swiss_theme.id,
    color_def.color_key,
    color_def.color_value,
    color_def.color_category
FROM swiss_theme,
(VALUES 
    ('base-100', 'oklch(97% 0.014 343.198)', 'base'),
    ('base-200', 'oklch(89% 0.026 342.55)', 'base'),
    ('base-300', 'oklch(80% 0.037 342.176)', 'base'),
    ('base-content', 'oklch(25% 0.027 342.334)', 'base'),
    ('primary', 'oklch(65% 0.241 354.308)', 'primary'),
    ('primary-content', 'oklch(89% 0.026 342.55)', 'primary'),
    ('secondary', 'oklch(62% 0.265 303.9)', 'secondary'),
    ('secondary-content', 'oklch(89% 0.026 342.55)', 'secondary'),
    ('accent', 'oklch(76% 0.184 183.61)', 'accent'),
    ('accent-content', 'oklch(15% 0.042 183.61)', 'accent'),
    ('neutral', 'oklch(25% 0.027 342.334)', 'neutral'),
    ('neutral-content', 'oklch(89% 0.026 342.55)', 'neutral'),
    ('info', 'oklch(63% 0.181 231.729)', 'info'),
    ('info-content', 'oklch(15% 0.042 231.729)', 'info'),
    ('success', 'oklch(64% 0.155 160.463)', 'success'),
    ('success-content', 'oklch(15% 0.042 160.463)', 'success'),
    ('warning', 'oklch(75% 0.166 83.618)', 'warning'),
    ('warning-content', 'oklch(15% 0.042 83.618)', 'warning'),
    ('error', 'oklch(61% 0.204 22.207)', 'error'),
    ('error-content', 'oklch(89% 0.026 342.55)', 'error')
) AS color_def(color_key, color_value, color_category);

-- Insert default theme colors for Swiss Dark
WITH swiss_dark_theme AS (SELECT id FROM themes WHERE name = 'swiss-dark')
INSERT INTO theme_colors (id, theme_id, color_key, color_value, color_category) 
SELECT 
    gen_random_uuid(),
    swiss_dark_theme.id,
    color_def.color_key,
    color_def.color_value,
    color_def.color_category
FROM swiss_dark_theme,
(VALUES 
    ('base-100', 'oklch(8% 0.061 343.231)', 'base'),
    ('base-200', 'oklch(15% 0.042 342.176)', 'base'),
    ('base-300', 'oklch(25% 0.027 342.334)', 'base'),
    ('base-content', 'oklch(89% 0.026 342.55)', 'base'),
    ('primary', 'oklch(65% 0.241 354.308)', 'primary'),
    ('primary-content', 'oklch(15% 0.042 342.176)', 'primary'),
    ('secondary', 'oklch(62% 0.265 303.9)', 'secondary'),
    ('secondary-content', 'oklch(15% 0.042 342.176)', 'secondary'),
    ('accent', 'oklch(76% 0.184 183.61)', 'accent'),
    ('accent-content', 'oklch(15% 0.042 183.61)', 'accent'),
    ('neutral', 'oklch(89% 0.026 342.55)', 'neutral'),
    ('neutral-content', 'oklch(15% 0.042 342.176)', 'neutral'),
    ('info', 'oklch(63% 0.181 231.729)', 'info'),
    ('info-content', 'oklch(15% 0.042 231.729)', 'info'),
    ('success', 'oklch(64% 0.155 160.463)', 'success'),
    ('success-content', 'oklch(15% 0.042 160.463)', 'success'),
    ('warning', 'oklch(75% 0.166 83.618)', 'warning'),
    ('warning-content', 'oklch(15% 0.042 83.618)', 'warning'),
    ('error', 'oklch(61% 0.204 22.207)', 'error'),
    ('error-content', 'oklch(15% 0.042 342.176)', 'error')
) AS color_def(color_key, color_value, color_category);

-- Create table for storing generated CSS
CREATE TABLE IF NOT EXISTS theme_css_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    css_content TEXT NOT NULL,
    generated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    is_current BOOLEAN DEFAULT false
);

-- Create table for system theme settings
CREATE TABLE IF NOT EXISTS system_theme_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    setting_key VARCHAR UNIQUE NOT NULL,
    theme_name VARCHAR NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for quick lookups
CREATE INDEX IF NOT EXISTS idx_theme_css_cache_is_current ON theme_css_cache(is_current);
CREATE INDEX IF NOT EXISTS idx_system_theme_settings_key ON system_theme_settings(setting_key);

-- Insert initial generated CSS (will be updated when themes are modified)
INSERT INTO theme_css_cache (css_content, is_current) VALUES ('/* CSS will be generated from database themes */', true);

-- Insert default system theme settings
INSERT INTO system_theme_settings (setting_key, theme_name) VALUES 
('default_light_theme', 'swiss'),
('default_dark_theme', 'swiss-dark')
ON CONFLICT (setting_key) DO NOTHING;