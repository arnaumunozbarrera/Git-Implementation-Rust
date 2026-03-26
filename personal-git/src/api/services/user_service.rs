use crate::api::clients::supabase::SupabaseClient;
use crate::api::models::User;

pub async fn get_users(
    client: &SupabaseClient,
) -> Result<Vec<User>, reqwest::Error> {

    let url = format!("{}/rest/v1/users", client.base_url);

    let res = client.client
        .get(url)
        .send()
        .await?
        .json::<Vec<User>>()
        .await?;

    Ok(res)
}