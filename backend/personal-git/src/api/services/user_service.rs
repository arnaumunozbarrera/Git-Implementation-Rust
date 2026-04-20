use crate::api::clients::supabase::SupabaseClient;
use crate::api::models::User;

pub async fn get_all_users(
    client: &SupabaseClient,
) -> Result<Vec<User>, sqlx::Error> {
    let users = sqlx::query_as::<_, User>(
        "SELECT * FROM users"
    )
    .fetch_all(&client.pool)
    .await?;

    Ok(users)
}