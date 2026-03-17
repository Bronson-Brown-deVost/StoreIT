-- Schema version tracking for migration system
CREATE TABLE IF NOT EXISTS _meta (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

INSERT INTO _meta (key, value) VALUES ('schema_version', '1');
