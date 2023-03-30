// use std::env;

use diesel::sqlite::SqliteConnection;
use diesel::Connection;
// use diesel::prelude::*;

use tauri::App;

pub fn establish_connection(app: &mut App) -> SqliteConnection {
    let resource_path = app.path_resolver()
        .resolve_resource("../databases/adam.db")
        .expect("failed to resolve resource");
    let database_url = format!("sqlite://{}", resource_path.display());
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}


use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
use std::error::Error;

pub fn run_migrations(connection: &mut impl MigrationHarness<diesel::sqlite::Sqlite>) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {

    // This will run the necessary migrations.
    //
    // See the documentation for `MigrationHarness` for
    // all available methods.
    connection.run_pending_migrations(MIGRATIONS)?;

    Ok(())
}