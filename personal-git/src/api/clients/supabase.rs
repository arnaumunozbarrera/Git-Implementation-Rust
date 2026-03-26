use reqwest::Client;
use std::env;

#[derive(Clone)]
pub struct SupabaseClient {
    pub client: Client,
    pub base_url: String,
}

impl SupabaseClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: env::var("SUPABASE_URL").unwrap(),
        }
    }
}