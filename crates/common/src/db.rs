use crate::error::Error;
use schemars::JsonSchema;
use serde::Deserialize;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::time::Duration;
use uuid::Uuid;

pub const DEFAULT_MAX_CONNECTIONS: u32 = 100;
pub const PAGING_MAX_PER_PAGE: usize = 100;
pub const PAGING_DEFAULT_PER_PAGE: usize = 10;

pub type DB = Pool<Postgres>;

pub async fn connect(app_name: Option<&str>) -> anyhow::Result<DB> {
    use anyhow::Context;

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| Error::BadConfig("DATABASE_URL env var is missing".to_string()))?;

    let max_connections = match std::env::var("DATABASE_CONNECTIONS") {
        Ok(n) => n.parse::<u32>().context("invalid DATABASE_CONNECTIONS")?,
        Err(_) => DEFAULT_MAX_CONNECTIONS,
    };

    Ok(pool_db(&database_url, max_connections, app_name).await?)
}

async fn pool_db(
    database_url: &str,
    max_connections: u32,
    app_name: Option<&str>,
) -> Result<DB, Error> {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(5))
        .max_connections(max_connections)
        .max_lifetime(Duration::from_secs(5 * 60))
        .idle_timeout(Duration::from_secs(60))
        .connect(
            format!(
                "{database_url}&application_name={}",
                app_name.unwrap_or(env!("CARGO_PKG_NAME"))
            )
            .as_str(),
        )
        .await
        .map_err(|err| Error::ConnectingToDatabase(err.to_string()))
}

#[derive(Deserialize, JsonSchema)]
pub struct Pagination {
    pub anchor: Option<Uuid>,
    pub per_page: Option<usize>,
}

pub fn paginate(pagination: Pagination) -> (usize, Option<Uuid>) {
    let per_page = pagination
        .per_page
        .unwrap_or(PAGING_DEFAULT_PER_PAGE)
        .clamp(1, PAGING_MAX_PER_PAGE);
    (per_page, pagination.anchor)
}
