use crate::api::clients::supabase::SupabaseClient;
use crate::api::models::{InitRepoRequest, InitRepoResponse, Repository};

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

pub async fn init_repo(
    client: &SupabaseClient,
    payload: InitRepoRequest,
) -> Result<InitRepoResponse, String> {
    let repo_id = payload.repo_id.trim();
    let name = payload.name.trim();
    let owner_id = payload.owner_id.trim();
    let default_branch = payload.default_branch.trim();
    let description = payload
        .description
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let readme_path = payload
        .readme_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    if repo_id.is_empty() {
        return Err("[ERROR] Missing repo_id".to_string());
    }

    if name.is_empty() {
        return Err("[ERROR] Missing name".to_string());
    }

    if owner_id.is_empty() {
        return Err("[ERROR] Missing owner_id".to_string());
    }

    if default_branch.is_empty() {
        return Err("[ERROR] Missing default_branch".to_string());
    }

    let owner_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
        .bind(owner_id)
        .fetch_one(&client.pool)
        .await
        .map_err(|error| format!("[ERROR] Failed to verify owner '{}': {}", owner_id, error))?;

    if !owner_exists {
        return Err(format!("[ERROR] Owner '{}' not found", owner_id));
    }

    let repo_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM repositories WHERE id = $1)")
            .bind(repo_id)
            .fetch_one(&client.pool)
            .await
            .map_err(|error| format!("[ERROR] Failed to verify repository '{}': {}", repo_id, error))?;

    if repo_exists {
        return Err(format!("[ERROR] Repository '{}' already exists", repo_id));
    }

    sqlx::query(
        "INSERT INTO repositories (id, name, owner_id, is_private, description, tags, default_branch, readme_path, theme) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(repo_id)
    .bind(name)
    .bind(owner_id)
    .bind(payload.is_private)
    .bind(description)
    .bind(payload.tags)
    .bind(default_branch)
    .bind(readme_path)
    .bind(payload.theme)
    .execute(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to initialize repository '{}': {}", repo_id, error))?;

    sqlx::query(
        "INSERT INTO branches (repo_id, name, last_commit_hash) VALUES ($1, $2, $3)",
    )
    .bind(repo_id)
    .bind(default_branch)
    .bind(Option::<String>::None)
    .execute(&client.pool)
    .await
    .map_err(|error| format!(
        "[ERROR] Failed to create default branch '{}' for repository '{}': {}",
        default_branch, repo_id, error
    ))?;

    Ok(InitRepoResponse {
        message: format!("Initialized remote repository '{}'", repo_id),
        repo_id: repo_id.to_string(),
        database_action: Some(format!(
            "Created repository '{}' with default branch '{}'",
            repo_id, default_branch
        )),
    })
}
