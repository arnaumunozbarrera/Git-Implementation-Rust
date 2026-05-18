use crate::api::clients::supabase::SupabaseClient;
use crate::api::models::{DeleteActionResponse, User};

pub async fn get_all_users(client: &SupabaseClient) -> Result<Vec<User>, sqlx::Error> {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(&client.pool)
        .await?;

    Ok(users)
}

pub async fn update_user_profile(
    client: &SupabaseClient,
    user_id: &str,
    username: Option<&str>,
    email: Option<&str>,
) -> Result<User, String> {
    let user_id = user_id.trim();
    if user_id.is_empty() {
        return Err("[ERROR] Missing user_id".to_string());
    }

    let normalized_username = username
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let normalized_email = email
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    sqlx::query_as::<_, User>(
        "UPDATE users
         SET username = $2,
             email = COALESCE($3, email)
         WHERE id = $1
         RETURNING id, username, email, created_at::text AS created_at",
    )
    .bind(user_id)
    .bind(normalized_username)
    .bind(normalized_email)
    .fetch_optional(&client.pool)
    .await
    .map_err(|error| {
        format!(
            "[ERROR] Failed to update user profile '{}': {}",
            user_id, error
        )
    })?
    .ok_or_else(|| format!("[ERROR] User '{}' not found", user_id))
}

pub async fn delete_user_account(
    client: &SupabaseClient,
    user_id: &str,
) -> Result<DeleteActionResponse, String> {
    let user_id = user_id.trim();

    if user_id.is_empty() {
        return Err("[ERROR] Missing user_id".to_string());
    }

    let repo_ids =
        sqlx::query_scalar::<_, String>("SELECT id FROM repositories WHERE owner_id = $1")
            .bind(user_id)
            .fetch_all(&client.pool)
            .await
            .map_err(|error| {
                format!(
                    "[ERROR] Failed to list repositories for user '{}': {}",
                    user_id, error
                )
            })?;

    for repo_id in &repo_ids {
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
    }

    sqlx::query("DELETE FROM repositories WHERE owner_id = $1")
        .bind(user_id)
        .execute(&client.pool)
        .await
        .map_err(|error| {
            format!(
                "[ERROR] Failed to delete repositories for user '{}': {}",
                user_id, error
            )
        })?;

    sqlx::query("DELETE FROM stars WHERE user_id = $1")
        .bind(user_id)
        .execute(&client.pool)
        .await
        .map_err(|error| {
            format!(
                "[ERROR] Failed to delete user stars '{}': {}",
                user_id, error
            )
        })?;

    sqlx::query("DELETE FROM repo_access_logs WHERE user_id = $1")
        .bind(user_id)
        .execute(&client.pool)
        .await
        .map_err(|error| {
            format!(
                "[ERROR] Failed to delete user access logs '{}': {}",
                user_id, error
            )
        })?;

    sqlx::query("DELETE FROM commits_metadata WHERE author_id = $1")
        .bind(user_id)
        .execute(&client.pool)
        .await
        .map_err(|error| {
            format!(
                "[ERROR] Failed to delete user commit metadata '{}': {}",
                user_id, error
            )
        })?;

    sqlx::query("DELETE FROM commits WHERE author_id = $1")
        .bind(user_id)
        .execute(&client.pool)
        .await
        .map_err(|error| {
            format!(
                "[ERROR] Failed to delete user commits '{}': {}",
                user_id, error
            )
        })?;

    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&client.pool)
        .await
        .map_err(|error| format!("[ERROR] Failed to delete user '{}': {}", user_id, error))?;

    if result.rows_affected() == 0 {
        return Err(format!("[ERROR] User '{}' not found", user_id));
    }

    Ok(DeleteActionResponse {
        message: format!("Deleted account records for '{}'", user_id),
        database_action: Some(format!("Removed user '{}' and owned repositories", user_id)),
    })
}
