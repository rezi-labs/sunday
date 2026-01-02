-- Migration to make year field non-nullable with default value
-- Set NULL years to 1900 first, then make the column NOT NULL with default

-- Update any existing NULL year values to 1900
UPDATE song SET year = 1900 WHERE year IS NULL;

-- Alter the year column to be NOT NULL with default 1900
ALTER TABLE song ALTER COLUMN year SET DEFAULT 1900;
ALTER TABLE song ALTER COLUMN year SET NOT NULL;
