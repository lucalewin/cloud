-- Enable UUID generation extension (for PostgreSQL)
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Users table (you can expand this as needed)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT UNIQUE NOT NULL,
    name TEXT,
    created_at TIMESTAMP DEFAULT now()
);

-- Create folders table
CREATE TABLE folders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    parent_id UUID REFERENCES folders(id) ON DELETE CASCADE,
    created_at TIMESTAMP DEFAULT now(),

    UNIQUE (user_id, parent_id, name)  -- Prevent duplicate folder names in same parent
);

-- Files table
CREATE TABLE files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    folder_id UUID REFERENCES folders(id) ON DELETE SET NULL,
    filename TEXT NOT NULL,
    size BIGINT NOT NULL,
    last_modified TIMESTAMP DEFAULT now(),

    UNIQUE (user_id, folder_id, filename)  -- Prevent duplicate filenames in same folder
);
