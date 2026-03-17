-- Locations table
CREATE TABLE IF NOT EXISTS locations (
    id TEXT PRIMARY KEY NOT NULL,
    group_id TEXT NOT NULL,
    parent_id TEXT REFERENCES locations(id),
    name TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f', 'now'))
);

CREATE INDEX idx_locations_group ON locations(group_id);
CREATE INDEX idx_locations_parent ON locations(parent_id);

-- Containers table
CREATE TABLE IF NOT EXISTS containers (
    id TEXT PRIMARY KEY NOT NULL,
    group_id TEXT NOT NULL,
    parent_location_id TEXT REFERENCES locations(id),
    parent_container_id TEXT REFERENCES containers(id),
    name TEXT NOT NULL,
    description TEXT,
    color TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f', 'now')),
    CHECK (
        (parent_location_id IS NOT NULL AND parent_container_id IS NULL) OR
        (parent_location_id IS NULL AND parent_container_id IS NOT NULL)
    )
);

CREATE INDEX idx_containers_group ON containers(group_id);
CREATE INDEX idx_containers_parent_location ON containers(parent_location_id);
CREATE INDEX idx_containers_parent_container ON containers(parent_container_id);

-- Items table
CREATE TABLE IF NOT EXISTS items (
    id TEXT PRIMARY KEY NOT NULL,
    group_id TEXT NOT NULL,
    container_id TEXT REFERENCES containers(id),
    location_id TEXT REFERENCES locations(id),
    name TEXT NOT NULL,
    description TEXT,
    aliases TEXT NOT NULL DEFAULT '[]',
    keywords TEXT NOT NULL DEFAULT '[]',
    category TEXT,
    barcode TEXT,
    material TEXT,
    color TEXT,
    condition_notes TEXT,
    quantity INTEGER NOT NULL DEFAULT 1,
    ai_raw TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f', 'now')),
    CHECK (
        (container_id IS NOT NULL AND location_id IS NULL) OR
        (container_id IS NULL AND location_id IS NOT NULL)
    )
);

CREATE INDEX idx_items_group ON items(group_id);
CREATE INDEX idx_items_container ON items(container_id);
CREATE INDEX idx_items_location ON items(location_id);

-- Photos table
CREATE TABLE IF NOT EXISTS photos (
    id TEXT PRIMARY KEY NOT NULL,
    entity_type TEXT NOT NULL CHECK (entity_type IN ('location', 'container', 'item')),
    entity_id TEXT NOT NULL,
    storage_key TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    is_primary INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f', 'now'))
);

CREATE INDEX idx_photos_entity ON photos(entity_type, entity_id);

-- NFC Tags table
CREATE TABLE IF NOT EXISTS nfc_tags (
    id TEXT PRIMARY KEY NOT NULL,
    group_id TEXT NOT NULL,
    tag_uri TEXT NOT NULL UNIQUE,
    entity_type TEXT CHECK (entity_type IN ('location', 'container')),
    entity_id TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f', 'now')),
    assigned_at TEXT
);

CREATE INDEX idx_nfc_tags_uri ON nfc_tags(tag_uri);
CREATE INDEX idx_nfc_tags_group ON nfc_tags(group_id);
