-- Add migration script here
CREATE TABLE assets (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);