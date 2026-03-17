use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "storeit-server", version, about = "StoreIT inventory server")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Start the web server (default)
    Serve,

    /// Import a .storeit archive into a fresh database
    Import {
        /// Path to the .storeit archive file
        archive: String,

        /// Import mode: "replace" or "merge"
        #[arg(long, default_value = "replace")]
        mode: String,
    },

    /// Print schema version and app version as JSON
    Version,

    /// Check schema version and auto-migrate if needed (used by Docker entrypoint)
    AutoUpgrade,
}
