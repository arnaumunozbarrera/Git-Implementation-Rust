use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;

#[derive(Clone)]
pub struct SupabaseClient {
    pub pool: PgPool,
}

impl SupabaseClient {
    pub async fn new() -> Self {
        let database_url = env::var("SUPABASE_URL")
            .expect("SUPABASE_URL must be set");

        // println!("[DEBUG] SUPABASE_URL = {}", database_url);

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .unwrap_or_else(|e| panic!("Failed to connect to Supabase Postgres: {e}"));

        Self { pool }
    }

    pub async fn healthcheck(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}