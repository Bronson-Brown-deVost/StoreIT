use std::sync::Arc;

use clap::Parser;
use tracing_subscriber::EnvFilter;

use storeit_server::app_state;
use storeit_server::cli::{Cli, Command};
use storeit_server::config;
use storeit_server::interchange;
use storeit_server::router;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Version) => {
            cmd_version().await?;
        }
        Some(Command::Import { archive, mode }) => {
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::from_default_env())
                .init();
            cmd_import(&archive, &mode).await?;
        }
        Some(Command::AutoUpgrade) => {
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::from_default_env())
                .init();
            cmd_auto_upgrade().await?;
        }
        Some(Command::Serve) | None => {
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::from_default_env())
                .init();
            cmd_serve().await?;
        }
    }

    Ok(())
}

async fn cmd_serve() -> anyhow::Result<()> {
    let config = config::Config::from_env()?;
    tracing::info!("database: {}", config.database_url);
    tracing::info!("images:   {}", config.image_storage_path);
    let state = app_state::AppState::new(&config).await?;

    // Startup version check (uses the same pool AppState created)
    let db = storeit_db_sqlite::SqliteDb::new(state.db_pool().clone());
    let current = db.schema_version().await;
    let expected = storeit_db_sqlite::SqliteDb::EXPECTED_SCHEMA_VERSION;
    if current != expected {
        anyhow::bail!(
            "Schema version mismatch: database has version {current}, \
             but this binary expects version {expected}. \
             Use storeit-ctl to upgrade/downgrade."
        );
    }

    let app = router::build_router(Arc::new(state));

    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    tracing::info!("listening on {}", config.bind_addr);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn cmd_version() -> anyhow::Result<()> {
    // Try to read actual schema version from DB if possible
    let schema_version = if let Ok(config) = config::Config::from_env() {
        if let Ok(pool) = sqlx::SqlitePool::connect(&config.database_url).await {
            let db = storeit_db_sqlite::SqliteDb::new(pool);
            db.schema_version().await
        } else {
            0
        }
    } else {
        0
    };

    let info = serde_json::json!({
        "app_version": env!("CARGO_PKG_VERSION"),
        "schema_version": schema_version,
        "expected_schema_version": storeit_db_sqlite::SqliteDb::EXPECTED_SCHEMA_VERSION,
    });
    println!("{}", serde_json::to_string_pretty(&info)?);
    Ok(())
}

async fn cmd_import(archive_path: &str, mode: &str) -> anyhow::Result<()> {
    let config = config::Config::from_env()?;

    // Extract DB file path from the database URL
    let db_file = config
        .database_url
        .strip_prefix("sqlite:")
        .and_then(|s| s.split('?').next())
        .map(|s| s.to_string());

    // Step 1: If existing DB file found, rename to .pre-import
    let pre_import_path = db_file.as_ref().map(|f| format!("{f}.pre-import"));
    if let Some(ref db_path) = db_file {
        let path = std::path::Path::new(db_path);
        if path.exists() {
            let backup = format!("{db_path}.pre-import");
            tracing::info!("backing up existing database to {backup}");
            std::fs::rename(path, &backup)?;
        }
    }

    // Step 2: Create fresh DB with migrations
    let result = run_import(&config, archive_path, mode).await;

    match result {
        Ok(()) => {
            tracing::info!("import completed successfully");
            if let Some(ref backup) = pre_import_path.filter(|b| std::path::Path::new(b).exists()) {
                tracing::info!(
                    "previous database saved at {backup}. \
                     Delete it after verifying the import, or restore it to rollback."
                );
            }
            Ok(())
        }
        Err(e) => {
            tracing::error!("import failed: {e}");
            // Rollback: restore the old DB
            if let (Some(db_path), Some(backup)) = (
                &db_file,
                pre_import_path
                    .as_ref()
                    .filter(|b| std::path::Path::new(b.as_str()).exists()),
            ) {
                tracing::info!("rolling back: restoring {backup} to {db_path}");
                let _ = std::fs::remove_file(db_path);
                std::fs::rename(backup, db_path)?;
                tracing::info!("rollback complete");
            }
            Err(anyhow::anyhow!("{e}"))
        }
    }
}

async fn run_import(
    config: &config::Config,
    archive_path: &str,
    mode: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create the AppState (this creates the fresh DB and runs migrations)
    let state = app_state::AppState::new(config).await?;
    let state = Arc::new(state);

    // Read archive
    let data = std::fs::read(archive_path)?;

    let progress = CliProgress::new();
    let options = interchange::ImportOptions {
        mode: mode.to_string(),
        image_storage_path: None,
    };

    interchange::import_from_bytes(&state, &data, &options, &progress).await?;

    // Set schema version after successful import
    let pool = sqlx::SqlitePool::connect(&config.database_url).await?;
    let db = storeit_db_sqlite::SqliteDb::new(pool);
    db.set_schema_version(storeit_db_sqlite::SqliteDb::EXPECTED_SCHEMA_VERSION)
        .await;

    Ok(())
}

async fn cmd_auto_upgrade() -> anyhow::Result<()> {
    let config = config::Config::from_env()?;
    let expected = storeit_db_sqlite::SqliteDb::EXPECTED_SCHEMA_VERSION;

    // Extract DB file path
    let db_file = config
        .database_url
        .strip_prefix("sqlite:")
        .and_then(|s| s.split('?').next())
        .map(|s| s.to_string());

    let db_path = match &db_file {
        Some(p) if std::path::Path::new(p).exists() => p,
        _ => {
            // No existing DB — fresh install, nothing to migrate
            tracing::info!("no existing database found, skipping auto-upgrade");
            return Ok(());
        }
    };

    // Read current schema version from existing DB
    let pool = sqlx::SqlitePool::connect(&config.database_url).await?;
    let db = storeit_db_sqlite::SqliteDb::new(pool.clone());
    // Run pending sqlx migrations first so _meta table exists
    db.migrate().await?;
    let current = db.schema_version().await;
    // Close the pool before we rename the file
    pool.close().await;

    if current == expected {
        tracing::info!("schema version {current} matches, no migration needed");
        return Ok(());
    }

    tracing::info!(
        "schema version mismatch: database has {current}, binary expects {expected} — auto-upgrading"
    );

    // Step 1: Export all data from the old DB
    let temp_archive = std::env::temp_dir().join("storeit-auto-upgrade.storeit");
    {
        let state = app_state::AppState::new(&config).await?;
        let state = Arc::new(state);
        let progress = CliProgress::new();
        let options = interchange::ExportOptions {
            include_images: true,
        };
        interchange::export_to_file(&state, &temp_archive, &options, &progress)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        tracing::info!("exported data to temporary archive");
    }
    // AppState and pool are dropped here

    // Step 2: Rename old DB to .pre-upgrade
    let pre_upgrade = format!("{db_path}.pre-upgrade");
    tracing::info!("backing up database to {pre_upgrade}");
    std::fs::rename(db_path, &pre_upgrade)?;

    // Step 3: Import into fresh DB using current binary
    let import_result = run_import(&config, temp_archive.to_str().unwrap(), "replace").await;

    match import_result {
        Ok(()) => {
            tracing::info!("auto-upgrade complete: schema version {current} -> {expected}");
            // Clean up temp archive
            let _ = std::fs::remove_file(&temp_archive);
            // Keep .pre-upgrade for safety — Docker users can delete it
            tracing::info!(
                "previous database saved at {pre_upgrade}. \
                 Delete it after verifying the upgrade."
            );
            Ok(())
        }
        Err(e) => {
            tracing::error!("auto-upgrade import failed: {e}");
            // Rollback
            let _ = std::fs::remove_file(db_path);
            std::fs::rename(&pre_upgrade, db_path)?;
            let _ = std::fs::remove_file(&temp_archive);
            tracing::info!("rolled back to previous database");
            Err(anyhow::anyhow!(
                "auto-upgrade failed: {e}. Database has been restored to the previous version."
            ))
        }
    }
}

/// Simple CLI progress reporter that logs to tracing.
struct CliProgress {
    total: std::sync::atomic::AtomicU64,
    progress: std::sync::atomic::AtomicU64,
}

impl CliProgress {
    fn new() -> Self {
        Self {
            total: std::sync::atomic::AtomicU64::new(0),
            progress: std::sync::atomic::AtomicU64::new(0),
        }
    }
}

impl interchange::ProgressReporter for CliProgress {
    fn set_total(&self, total: u64) {
        self.total
            .store(total, std::sync::atomic::Ordering::Relaxed);
        tracing::info!("import: {total} items to process");
    }

    fn inc_progress(&self) {
        let p = self
            .progress
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;
        let t = self.total.load(std::sync::atomic::Ordering::Relaxed);
        if t > 0 && (p.is_multiple_of(100) || p == t) {
            tracing::info!("import: {p}/{t}");
        }
    }

    fn set_status(&self, status: &str) {
        tracing::info!("import status: {status}");
    }
}
