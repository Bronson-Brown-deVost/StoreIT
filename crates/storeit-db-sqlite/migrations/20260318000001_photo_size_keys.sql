-- Store explicit keys for thumbnail and large display versions.
-- Previously thumbnail_key was derived from storage_key; now all three are explicit.
ALTER TABLE photos ADD COLUMN thumbnail_key TEXT;
ALTER TABLE photos ADD COLUMN large_key TEXT;

UPDATE _meta SET value = '2' WHERE key = 'schema_version';
