use crate::api::clients::supabase::SupabaseClient;
use crate::api::models::{
    Branch, DeleteActionResponse, InitRepoRequest, InitRepoResponse, Repository,
};

pub async fn get_all_repos(
    client: &SupabaseClient,
    owner_id: &str,
) -> Result<Vec<Repository>, sqlx::Error> {
    let repos = sqlx::query_as::<_, Repository>(
        "SELECT id, name, owner_id, is_private, description, tags, default_branch,
                stars_count, readme_path, theme, created_at::text AS created_at
         FROM repositories
         WHERE owner_id = $1
         ORDER BY created_at DESC",
    )
    .bind(owner_id)
    .fetch_all(&client.pool)
    .await?;

    Ok(repos)
}

pub async fn get_repo_branches(
    client: &SupabaseClient,
    owner_id: &str,
    repo_id: &str,
) -> Result<Vec<Branch>, String> {
    let owner_id = owner_id.trim();
    let repo_id = repo_id.trim();

    if repo_id.is_empty() {
        return Err("[ERROR] Missing repo_id".to_string());
    }

    let repository_owner: Option<String> =
        sqlx::query_scalar("SELECT owner_id FROM repositories WHERE id = $1")
            .bind(repo_id)
            .fetch_optional(&client.pool)
            .await
            .map_err(|error| {
                format!("[ERROR] Failed to load repository '{}': {}", repo_id, error)
            })?;

    let Some(repository_owner) = repository_owner else {
        return Err(format!("[ERROR] Repository '{}' not found", repo_id));
    };

    if repository_owner != owner_id {
        return Err(format!(
            "[ERROR] User '{}' cannot access repository '{}'",
            owner_id, repo_id
        ));
    }

    sqlx::query_as::<_, Branch>(
        "SELECT id::text AS id, repo_id, name, last_commit_hash, created_at::text AS created_at
         FROM branches
         WHERE repo_id = $1
         ORDER BY
            CASE WHEN name = (SELECT default_branch FROM repositories WHERE id = $1) THEN 0 ELSE 1 END,
            created_at DESC,
            name ASC",
    )
    .bind(repo_id)
    .fetch_all(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to list branches for '{}': {}", repo_id, error))
}

pub async fn init_repo(
    client: &SupabaseClient,
    owner_id: &str,
    payload: InitRepoRequest,
) -> Result<InitRepoResponse, String> {
    let repo_id = payload.repo_id.trim();
    let name = payload.name.trim();
    let owner_id = owner_id.trim();
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
            .map_err(|error| {
                format!(
                    "[ERROR] Failed to verify repository '{}': {}",
                    repo_id, error
                )
            })?;

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

    sqlx::query("INSERT INTO branches (repo_id, name, last_commit_hash) VALUES ($1, $2, $3)")
        .bind(repo_id)
        .bind(default_branch)
        .bind(Option::<String>::None)
        .execute(&client.pool)
        .await
        .map_err(|error| {
            format!(
                "[ERROR] Failed to create default branch '{}' for repository '{}': {}",
                default_branch, repo_id, error
            )
        })?;

    Ok(InitRepoResponse {
        message: format!("Initialized remote repository '{}'", repo_id),
        repo_id: repo_id.to_string(),
        database_action: Some(format!(
            "Created repository '{}' with default branch '{}'",
            repo_id, default_branch
        )),
    })
}

pub async fn delete_repo(
    client: &SupabaseClient,
    user_id: &str,
    repo_id: &str,
) -> Result<DeleteActionResponse, String> {
    let user_id = user_id.trim();
    let repo_id = repo_id.trim();

    if repo_id.is_empty() {
        return Err("[ERROR] Missing repo_id".to_string());
    }

    let owner_id: Option<String> =
        sqlx::query_scalar("SELECT owner_id FROM repositories WHERE id = $1")
            .bind(repo_id)
            .fetch_optional(&client.pool)
            .await
            .map_err(|error| {
                format!("[ERROR] Failed to load repository '{}': {}", repo_id, error)
            })?;

    let Some(owner_id) = owner_id else {
        return Err(format!("[ERROR] Repository '{}' not found", repo_id));
    };

    if owner_id != user_id {
        return Err(format!(
            "[ERROR] User '{}' cannot delete repository '{}'",
            user_id, repo_id
        ));
    }

    sqlx::query("DELETE FROM stars WHERE repo_id = $1")
        .bind(repo_id)
        .execute(&client.pool)
        .await
        .map_err(|error| {
            format!(
                "[ERROR] Failed to delete stars for '{}': {}",
                repo_id, error
            )
        })?;

    sqlx::query("DELETE FROM repo_access_logs WHERE repo_id = $1")
        .bind(repo_id)
        .execute(&client.pool)
        .await
        .map_err(|error| {
            format!(
                "[ERROR] Failed to delete access logs for '{}': {}",
                repo_id, error
            )
        })?;

    sqlx::query("DELETE FROM commits_metadata WHERE repo_id = $1")
        .bind(repo_id)
        .execute(&client.pool)
        .await
        .map_err(|error| {
            format!(
                "[ERROR] Failed to delete commit metadata for '{}': {}",
                repo_id, error
            )
        })?;

    sqlx::query("DELETE FROM branches WHERE repo_id = $1")
        .bind(repo_id)
        .execute(&client.pool)
        .await
        .map_err(|error| {
            format!(
                "[ERROR] Failed to delete branches for '{}': {}",
                repo_id, error
            )
        })?;

    let result = sqlx::query("DELETE FROM repositories WHERE id = $1 AND owner_id = $2")
        .bind(repo_id)
        .bind(user_id)
        .execute(&client.pool)
        .await
        .map_err(|error| {
            format!(
                "[ERROR] Failed to delete repository '{}': {}",
                repo_id, error
            )
        })?;

    if result.rows_affected() == 0 {
        return Err(format!("[ERROR] Repository '{}' not found", repo_id));
    }

    Ok(DeleteActionResponse {
        message: format!("Deleted repository '{}'", repo_id),
        database_action: Some(format!(
            "Removed repository '{}' and related records",
            repo_id
        )),
    })
}
