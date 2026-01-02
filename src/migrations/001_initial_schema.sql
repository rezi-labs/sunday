-- Initial database schema for Sunday application
-- This migration creates all tables with their final structure

-- Create sunday_user table
CREATE TABLE IF NOT EXISTS sunday_user (
    id UUID PRIMARY KEY,
    username VARCHAR UNIQUE NOT NULL,
    email VARCHAR UNIQUE NOT NULL,
    password_hash VARCHAR NOT NULL,
    salt VARCHAR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ,
    last_login_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_admin BOOLEAN NOT NULL DEFAULT false,
    reset_token VARCHAR,
    reset_token_expires_at TIMESTAMPTZ
);

-- Create session table
CREATE TABLE IF NOT EXISTS session (
    id VARCHAR PRIMARY KEY,
    user_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL,
    last_accessed_at TIMESTAMPTZ,
    user_agent VARCHAR,
    ip_address VARCHAR,
    CONSTRAINT fk_session_user_id FOREIGN KEY (user_id) REFERENCES sunday_user(id) ON DELETE CASCADE ON UPDATE CASCADE
);

-- Create indexes for session table
CREATE INDEX IF NOT EXISTS idx_session_user_id ON session(user_id);
CREATE INDEX IF NOT EXISTS idx_session_expires_at ON session(expires_at);
