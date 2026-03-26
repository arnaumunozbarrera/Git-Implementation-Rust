use crate::api::clients::supabase::SupabaseClient;
use crate::api::models::Repository;

pub async fn get_all_repos(
    client: &SupabaseClient,
) -> Result<Vec<Repository>, reqwest::Error> {

    let url = format!("{}/rest/v1/repositories", client.base_url);

    let res = client.client
        .get(url)
        .send()
        .await?
        .json::<Vec<Repository>>()
        .await?;

    Ok(res)
}