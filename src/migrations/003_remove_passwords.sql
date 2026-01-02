-- Remove password-related columns from users table
ALTER TABLE sunday_user 
DROP COLUMN IF EXISTS password_hash,
DROP COLUMN IF EXISTS salt,
DROP COLUMN IF EXISTS reset_token,
DROP COLUMN IF EXISTS reset_token_expires_at;