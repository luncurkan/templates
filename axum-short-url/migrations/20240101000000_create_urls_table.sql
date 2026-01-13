-- Create urls table for short URL storage
CREATE TABLE IF NOT EXISTS urls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    short_code VARCHAR(10) NOT NULL UNIQUE,
    original_url TEXT NOT NULL,
    clicks BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for fast short_code lookups
CREATE INDEX idx_urls_short_code ON urls(short_code);

-- Index for listing URLs by creation date
CREATE INDEX idx_urls_created_at ON urls(created_at DESC);
