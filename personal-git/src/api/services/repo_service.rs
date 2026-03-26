use crate::api::clients::supabase::SupabaseClient;
use crate::api::models::Repository;

pub async fn get_all_repos(
    client: &SupabaseClient,
) -> Result<Vec<Repository>, sqlx::Error> {
    let repos = sqlx::query_as::<_, Repository>(
        "SELECT * FROM repositories"
    )
    .fetch_all(&client.pool)
    .await?;

    Ok(repos)
}