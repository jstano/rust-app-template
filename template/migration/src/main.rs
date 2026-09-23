use sea_orm_migration::prelude::*;
use std::env;

#[tokio::main]
async fn main() {
    // SeaORM's migration CLI expects DATABASE_URL, but the app reads
    // {{ project_name | upper | replace('-', '_') }}_DATABASE_URL (see .env-example) —
    // map it over if set.
    if let Ok(database_url) = env::var("{{ project_name | upper | replace('-', '_') }}_DATABASE_URL") {
        // SAFETY: runs before any threads are spawned (at program startup), so there is
        // no risk of concurrent environment variable access.
        unsafe {
            env::set_var("DATABASE_URL", database_url);
        }
    }

    cli::run_cli(migration::Migrator).await;
}
