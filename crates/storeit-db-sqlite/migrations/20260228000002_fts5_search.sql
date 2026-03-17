-- FTS5 virtual table for full-text search.
-- entity_type, entity_id, group_id are unindexed (stored for filtering).
CREATE VIRTUAL TABLE IF NOT EXISTS search_index USING fts5(
    entity_type UNINDEXED,
    entity_id UNINDEXED,
    group_id UNINDEXED,
    searchable_text,
    tokenize = 'porter unicode61'
);
