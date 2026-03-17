use std::collections::HashSet;
use std::io::Write as _;
use std::path::Path;
use std::sync::Arc;

use crate::app_state::AppState;

/// Progress reporting trait — decouples export/import from BackupJob/RestoreJob.
pub trait ProgressReporter: Send + Sync {
    fn set_total(&self, total: u64);
    fn inc_progress(&self);
    fn set_status(&self, status: &str);
}

/// Options for export.
pub struct ExportOptions {
    pub include_images: bool,
}

/// Options for import.
pub struct ImportOptions {
    pub mode: String, // "replace" or "merge"
    pub image_storage_path: Option<String>,
}

/// Archive manifest (v2).
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub format_version: u32,
    pub schema_version: i64,
    pub app_version: String,
    pub created_at: String,
    pub includes_images: bool,
}

/// Export all data to a .storeit file (zstd-compressed tar).
pub async fn export_to_file(
    state: &Arc<AppState>,
    output_path: &Path,
    options: &ExportOptions,
    progress: &dyn ProgressReporter,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    progress.set_status("running");

    // Collect all data
    let users = state.user_repo.list_all().await?;
    let groups = state.group_repo.list_all().await?;
    let memberships = state.user_group_repo.list_all().await?;
    let settings = state.settings_repo.list_all().await?;
    let locations = state.location_repo.list_all_unscoped().await?;
    let containers = state.container_repo.list_all_unscoped().await?;
    let items = state.item_repo.list_all_unscoped().await?;
    let photos = state.photo_repo.list_all().await?;
    let nfc_tags = state.nfc_tag_repo.list_all_unscoped().await?;

    let image_count = if options.include_images {
        photos.len() as u64
    } else {
        0
    };
    progress.set_total(9 + image_count);

    // Create temp dir for building the archive
    let temp_dir = tempfile::TempDir::new()?;
    let data_dir = temp_dir.path().join("data");
    std::fs::create_dir_all(&data_dir)?;

    // Write manifest (v2)
    let manifest = Manifest {
        format_version: 2,
        schema_version: storeit_db_sqlite::SqliteDb::EXPECTED_SCHEMA_VERSION,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        includes_images: options.include_images,
    };
    std::fs::write(
        temp_dir.path().join("manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;

    // Collect password hashes for users
    let mut user_data: Vec<serde_json::Value> = Vec::new();
    for u in &users {
        let hash = state.user_repo.get_password_hash(&u.external_id).await?;
        let mut val = serde_json::to_value(u)?;
        if let Some(h) = hash {
            val["_password_hash"] = serde_json::Value::String(h);
        }
        user_data.push(val);
    }

    let data_files: Vec<(&str, serde_json::Value)> = vec![
        ("users.json", serde_json::to_value(&user_data)?),
        ("groups.json", serde_json::to_value(&groups)?),
        ("memberships.json", serde_json::to_value(&memberships)?),
        ("settings.json", serde_json::to_value(&settings)?),
        ("locations.json", serde_json::to_value(&locations)?),
        ("containers.json", serde_json::to_value(&containers)?),
        ("items.json", serde_json::to_value(&items)?),
        ("photos.json", serde_json::to_value(&photos)?),
        ("nfc_tags.json", serde_json::to_value(&nfc_tags)?),
    ];

    for (name, value) in &data_files {
        std::fs::write(data_dir.join(name), serde_json::to_string_pretty(value)?)?;
        progress.inc_progress();
    }

    // Write images if requested (dedup by storage key)
    if options.include_images {
        let images_dir = temp_dir.path().join("images");
        std::fs::create_dir_all(&images_dir)?;

        let storage = state.image_storage();
        let mut seen_keys = HashSet::new();
        for photo in &photos {
            if !seen_keys.insert(photo.storage_key.clone()) {
                progress.inc_progress();
                continue;
            }
            match storage.retrieve(&photo.storage_key).await {
                Ok((data, _)) => {
                    let dest = images_dir.join(&photo.storage_key);
                    if let Some(parent) = dest.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(dest, data)?;
                }
                Err(e) => {
                    tracing::warn!("skipping photo {}: {e}", photo.id);
                }
            }
            progress.inc_progress();
        }
    }

    // Build zstd-compressed tar archive
    let archive_file = std::fs::File::create(output_path)?;
    let zstd_encoder = zstd::stream::write::Encoder::new(archive_file, 3)?.auto_finish();
    let mut tar = tar::Builder::new(zstd_encoder);

    // Add manifest
    tar.append_path_with_name(
        temp_dir.path().join("manifest.json"),
        "backup/manifest.json",
    )?;

    // Add data files
    for (name, _) in &data_files {
        tar.append_path_with_name(data_dir.join(name), format!("backup/data/{name}"))?;
    }

    // Add images
    if options.include_images {
        let images_dir = temp_dir.path().join("images");
        if images_dir.exists() {
            walk_dir_tar(&images_dir, &images_dir, &mut tar)?;
        }
    }

    tar.into_inner()?.flush()?;

    Ok(())
}

/// Import data from archive bytes (.storeit zstd format).
pub async fn import_from_bytes(
    state: &Arc<AppState>,
    data: &[u8],
    options: &ImportOptions,
    progress: &dyn ProgressReporter,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    progress.set_status("running");

    let temp_dir = tempfile::TempDir::new()?;

    // Try zstd first (format v2)
    let unpack_result = unpack_zstd(data, temp_dir.path());
    if unpack_result.is_err() {
        // Try gzip (format v1 backward compat for HTTP restore)
        unpack_gzip(data, temp_dir.path())?;
    }

    let backup_dir = temp_dir.path().join("backup");
    if !backup_dir.exists() {
        return Err("invalid archive: missing 'backup/' directory".into());
    }

    // Read manifest
    let manifest_path = backup_dir.join("manifest.json");
    if !manifest_path.exists() {
        return Err("invalid archive: missing manifest.json".into());
    }
    let manifest_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&manifest_path)?)?;

    let _format_version = manifest_value
        .get("format_version")
        .or_else(|| manifest_value.get("version"))
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as u32;

    let archive_schema_version = manifest_value
        .get("schema_version")
        .and_then(|v| v.as_i64())
        .unwrap_or(1);

    let includes_images = manifest_value
        .get("includes_images")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let data_dir = backup_dir.join("data");

    // Parse all data files
    let users_json: Vec<serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(data_dir.join("users.json"))?)?;
    let groups: Vec<storeit_domain::entities::Group> =
        serde_json::from_str(&std::fs::read_to_string(data_dir.join("groups.json"))?)?;
    let memberships: Vec<storeit_domain::entities::UserGroup> =
        serde_json::from_str(&std::fs::read_to_string(data_dir.join("memberships.json"))?)?;
    let settings_list: Vec<(String, String)> =
        serde_json::from_str(&std::fs::read_to_string(data_dir.join("settings.json"))?)?;
    let locations: Vec<storeit_domain::entities::Location> =
        serde_json::from_str(&std::fs::read_to_string(data_dir.join("locations.json"))?)?;
    let containers: Vec<storeit_domain::entities::Container> =
        serde_json::from_str(&std::fs::read_to_string(data_dir.join("containers.json"))?)?;
    let items: Vec<storeit_domain::entities::Item> =
        serde_json::from_str(&std::fs::read_to_string(data_dir.join("items.json"))?)?;
    let photos: Vec<storeit_domain::entities::Photo> =
        serde_json::from_str(&std::fs::read_to_string(data_dir.join("photos.json"))?)?;
    let nfc_tags: Vec<storeit_domain::entities::NfcTag> =
        serde_json::from_str(&std::fs::read_to_string(data_dir.join("nfc_tags.json"))?)?;

    let image_count = if includes_images {
        photos.len() as u64
    } else {
        0
    };
    let total = users_json.len() as u64
        + groups.len() as u64
        + memberships.len() as u64
        + settings_list.len() as u64
        + locations.len() as u64
        + containers.len() as u64
        + items.len() as u64
        + photos.len() as u64
        + nfc_tags.len() as u64
        + image_count;
    progress.set_total(total);

    // Apply version transforms if needed
    let mut archive_data = ArchiveData {
        users_json,
        groups,
        memberships,
        settings_list,
        locations,
        containers,
        items,
        photos,
        nfc_tags,
    };
    transform_data(
        &mut archive_data,
        archive_schema_version,
        storeit_db_sqlite::SqliteDb::EXPECTED_SCHEMA_VERSION,
    )?;

    // Optionally update image storage path before restore
    if let Some(new_path) = &options.image_storage_path {
        let trimmed = new_path.trim();
        if !trimmed.is_empty() {
            std::fs::create_dir_all(trimmed)?;
            let new_storage = Arc::new(storeit_storage_fs::FsImageStorage::new(trimmed));
            state.swap_image_storage(trimmed.to_string(), new_storage);
            state
                .settings_repo
                .set("image_storage_path", trimmed)
                .await?;
        }
    }

    // Clone photos for image copying later
    let photos_for_images = archive_data.photos.clone();

    if options.mode == "replace" {
        restore_replace(state, progress, archive_data).await?;
    } else {
        restore_merge(state, progress, archive_data).await?;
    }

    // Copy images
    if includes_images {
        let images_dir = backup_dir.join("images");
        if images_dir.exists() {
            let storage = state.image_storage();
            let mut seen_keys = HashSet::new();
            for photo in &photos_for_images {
                if !seen_keys.insert(photo.storage_key.clone()) {
                    progress.inc_progress();
                    continue;
                }
                let src = images_dir.join(&photo.storage_key);
                if src.exists() {
                    let data = std::fs::read(&src)?;
                    storage
                        .store_at(&photo.storage_key, &data, &photo.mime_type)
                        .await?;
                }
                progress.inc_progress();
            }
        }
    }

    // Full reindex of search
    let locations_for_idx = state.location_repo.list_all_unscoped().await?;
    let containers_for_idx = state.container_repo.list_all_unscoped().await?;
    let items_for_idx = state.item_repo.list_all_unscoped().await?;
    state
        .search_repo
        .full_reindex(&locations_for_idx, &containers_for_idx, &items_for_idx)
        .await?;

    Ok(())
}

/// Parsed archive data, ready for transforms and insertion.
pub struct ArchiveData {
    pub users_json: Vec<serde_json::Value>,
    pub groups: Vec<storeit_domain::entities::Group>,
    pub memberships: Vec<storeit_domain::entities::UserGroup>,
    pub settings_list: Vec<(String, String)>,
    pub locations: Vec<storeit_domain::entities::Location>,
    pub containers: Vec<storeit_domain::entities::Container>,
    pub items: Vec<storeit_domain::entities::Item>,
    pub photos: Vec<storeit_domain::entities::Photo>,
    pub nfc_tags: Vec<storeit_domain::entities::NfcTag>,
}

/// Apply version transforms to in-memory data when schema versions differ.
/// For v1→v1 this is a no-op. Future versions add match arms here.
pub fn transform_data(
    data: &mut ArchiveData,
    from_version: i64,
    to_version: i64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if from_version == to_version {
        return Ok(());
    }

    // v1→v2: Photo now has thumbnail_key and large_key fields (both nullable).
    // Derive thumbnail_key from storage_key for backward compat.
    if from_version == 1 && to_version == 2 {
        for photo in &mut data.photos {
            if photo.thumbnail_key.is_none() {
                let stem = photo
                    .storage_key
                    .rsplit_once('.')
                    .map_or(photo.storage_key.as_str(), |(s, _)| s);
                photo.thumbnail_key = Some(format!("{stem}_thumb.webp"));
            }
            // large_key stays None — will be generated on first access or by migration
        }
        return Ok(());
    }

    Err(format!("unsupported schema version transform: {from_version} -> {to_version}").into())
}

async fn restore_replace(
    state: &Arc<AppState>,
    progress: &dyn ProgressReporter,
    data: ArchiveData,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Delete all in FK-safe order
    state.nfc_tag_repo.delete_all().await?;
    state.photo_repo.delete_all().await?;
    state.item_repo.delete_all().await?;
    state.container_repo.delete_all().await?;
    state.location_repo.delete_all().await?;
    state.session_repo.delete_all().await?;
    state.user_group_repo.delete_all().await?;
    state.group_repo.delete_all().await?;
    state.user_repo.delete_all().await?;
    state.settings_repo.delete_all().await?;

    // Insert in dependency order
    for val in &data.users_json {
        let user: storeit_domain::entities::User = serde_json::from_value(val.clone())?;
        let hash = val
            .get("_password_hash")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        state.user_repo.insert_raw(&user, hash.as_deref()).await?;
        progress.inc_progress();
    }

    for g in &data.groups {
        state.group_repo.insert_raw(g).await?;
        progress.inc_progress();
    }

    for m in &data.memberships {
        state
            .user_group_repo
            .insert_raw(m.user_id, m.group_id, m.role)
            .await?;
        progress.inc_progress();
    }

    for (k, v) in &data.settings_list {
        state.settings_repo.set(k, v).await?;
        progress.inc_progress();
    }

    for l in &data.locations {
        state.location_repo.insert_raw(l).await?;
        progress.inc_progress();
    }

    for c in &data.containers {
        state.container_repo.insert_raw(c).await?;
        progress.inc_progress();
    }

    for i in &data.items {
        state.item_repo.insert_raw(i).await?;
        progress.inc_progress();
    }

    for p in &data.photos {
        state.photo_repo.insert_raw(p).await?;
        progress.inc_progress();
    }

    for t in &data.nfc_tags {
        state.nfc_tag_repo.insert_raw(t).await?;
        progress.inc_progress();
    }

    Ok(())
}

async fn restore_merge(
    state: &Arc<AppState>,
    progress: &dyn ProgressReporter,
    data: ArchiveData,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::collections::HashMap;
    use uuid::Uuid;

    let mut remap: HashMap<Uuid, Uuid> = HashMap::new();

    fn get_or_create(remap: &mut HashMap<Uuid, Uuid>, old: Uuid) -> Uuid {
        *remap.entry(old).or_insert_with(Uuid::new_v4)
    }

    fn remap_id(remap: &HashMap<Uuid, Uuid>, old: Uuid) -> Uuid {
        remap.get(&old).copied().unwrap_or(old)
    }

    fn remap_opt(remap: &HashMap<Uuid, Uuid>, old: Option<Uuid>) -> Option<Uuid> {
        old.map(|id| remap.get(&id).copied().unwrap_or(id))
    }

    // Pre-generate all new UUIDs
    for val in &data.users_json {
        if let Some(id) = val
            .get("id")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<Uuid>().ok())
        {
            get_or_create(&mut remap, id);
        }
    }
    for g in &data.groups {
        get_or_create(&mut remap, g.id);
    }
    for l in &data.locations {
        get_or_create(&mut remap, l.id);
    }
    for c in &data.containers {
        get_or_create(&mut remap, c.id);
    }
    for i in &data.items {
        get_or_create(&mut remap, i.id);
    }
    for p in &data.photos {
        get_or_create(&mut remap, p.id);
    }
    for t in &data.nfc_tags {
        get_or_create(&mut remap, t.id);
    }

    // Remap and insert users
    for val in &data.users_json {
        let mut user: storeit_domain::entities::User = serde_json::from_value(val.clone())?;
        user.id = remap_id(&remap, user.id);
        user.external_id = format!("merged:{}", user.external_id);
        let hash = val
            .get("_password_hash")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        state.user_repo.insert_raw(&user, hash.as_deref()).await?;
        progress.inc_progress();
    }

    // Remap and insert groups
    let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
    for mut g in data.groups {
        g.id = remap_id(&remap, g.id);
        g.name = format!("{} (merged-{})", g.name, &timestamp);
        state.group_repo.insert_raw(&g).await?;
        progress.inc_progress();
    }

    // Remap and insert memberships
    for m in &data.memberships {
        let new_uid = remap_id(&remap, m.user_id);
        let new_gid = remap_id(&remap, m.group_id);
        state
            .user_group_repo
            .insert_raw(new_uid, new_gid, m.role)
            .await?;
        progress.inc_progress();
    }

    // Settings: merge (overwrite keys)
    for (k, v) in &data.settings_list {
        state.settings_repo.set(k, v).await?;
        progress.inc_progress();
    }

    // Remap and insert locations
    for mut l in data.locations {
        l.id = remap_id(&remap, l.id);
        l.group_id = remap_id(&remap, l.group_id);
        l.parent_id = remap_opt(&remap, l.parent_id);
        state.location_repo.insert_raw(&l).await?;
        progress.inc_progress();
    }

    // Remap and insert containers
    for mut c in data.containers {
        c.id = remap_id(&remap, c.id);
        c.group_id = remap_id(&remap, c.group_id);
        c.parent_location_id = remap_opt(&remap, c.parent_location_id);
        c.parent_container_id = remap_opt(&remap, c.parent_container_id);
        state.container_repo.insert_raw(&c).await?;
        progress.inc_progress();
    }

    // Remap and insert items
    for mut i in data.items {
        i.id = remap_id(&remap, i.id);
        i.group_id = remap_id(&remap, i.group_id);
        i.container_id = remap_opt(&remap, i.container_id);
        i.location_id = remap_opt(&remap, i.location_id);
        state.item_repo.insert_raw(&i).await?;
        progress.inc_progress();
    }

    // Remap and insert photos
    let mut photos = data.photos;
    for p in &mut photos {
        p.id = remap_id(&remap, p.id);
        p.entity_id = remap_id(&remap, p.entity_id);
        state.photo_repo.insert_raw(p).await?;
        progress.inc_progress();
    }

    // Remap and insert NFC tags
    for mut t in data.nfc_tags {
        t.id = remap_id(&remap, t.id);
        t.group_id = remap_id(&remap, t.group_id);
        t.entity_id = remap_opt(&remap, t.entity_id);
        state.nfc_tag_repo.insert_raw(&t).await?;
        progress.inc_progress();
    }

    Ok(())
}

// -- Helpers --

fn unpack_zstd(data: &[u8], dest: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cursor = std::io::Cursor::new(data);
    let decoder = zstd::stream::read::Decoder::new(cursor)?;
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(dest)?;
    Ok(())
}

fn unpack_gzip(data: &[u8], dest: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cursor = std::io::Cursor::new(data);
    let gz = flate2::read::GzDecoder::new(cursor);
    let mut archive = tar::Archive::new(gz);
    archive.unpack(dest)?;
    Ok(())
}

fn walk_dir_tar<W: std::io::Write>(
    dir: &Path,
    base: &Path,
    tar: &mut tar::Builder<W>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_dir_tar(&path, base, tar)?;
        } else {
            let rel = path.strip_prefix(base)?;
            tar.append_path_with_name(&path, format!("backup/images/{}", rel.to_string_lossy()))?;
        }
    }
    Ok(())
}
