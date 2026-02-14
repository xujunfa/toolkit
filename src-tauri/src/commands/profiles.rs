use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub alias: String,
    pub env_vars: String, // JSON text: [{"key":"...","value":"..."},...]
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileResponse {
    pub id: String,
    pub name: String,
    pub alias: String,
    pub env_vars: Vec<EnvVar>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProfileInput {
    pub name: String,
    pub alias: String,
    pub env_vars: Vec<EnvVar>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProfileInput {
    pub name: Option<String>,
    pub alias: Option<String>,
    pub env_vars: Option<Vec<EnvVar>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProfilesResult {
    pub message: String,
    pub target_path: String,
    pub is_real_zshrc: bool,
}

impl Profile {
    fn to_response(&self) -> Result<ProfileResponse, String> {
        let env_vars: Vec<EnvVar> =
            serde_json::from_str(&self.env_vars).map_err(|e| e.to_string())?;
        Ok(ProfileResponse {
            id: self.id.clone(),
            name: self.name.clone(),
            alias: self.alias.clone(),
            env_vars,
            created_at: self.created_at.clone(),
            updated_at: self.updated_at.clone(),
        })
    }
}

pub async fn list_profiles_by_pool(db: &SqlitePool) -> Result<Vec<ProfileResponse>, String> {
    let profiles = sqlx::query_as::<_, Profile>("SELECT * FROM profiles ORDER BY created_at DESC")
        .fetch_all(db)
        .await
        .map_err(|e| e.to_string())?;
    profiles.iter().map(|p| p.to_response()).collect()
}

pub async fn create_profile_by_pool(
    db: &SqlitePool,
    input: CreateProfileInput,
) -> Result<ProfileResponse, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let env_vars_json = serde_json::to_string(&input.env_vars).map_err(|e| e.to_string())?;
    sqlx::query("INSERT INTO profiles (id, name, alias, env_vars) VALUES (?, ?, ?, ?)")
        .bind(&id)
        .bind(&input.name)
        .bind(&input.alias)
        .bind(&env_vars_json)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;

    let profile = sqlx::query_as::<_, Profile>("SELECT * FROM profiles WHERE id = ?")
        .bind(&id)
        .fetch_one(db)
        .await
        .map_err(|e| e.to_string())?;
    profile.to_response()
}

#[tauri::command]
pub async fn list_profiles(db: State<'_, SqlitePool>) -> Result<Vec<ProfileResponse>, String> {
    list_profiles_by_pool(db.inner()).await
}

#[tauri::command]
pub async fn create_profile(
    db: State<'_, SqlitePool>,
    input: CreateProfileInput,
) -> Result<ProfileResponse, String> {
    create_profile_by_pool(db.inner(), input).await
}

pub async fn update_profile_by_pool(
    db: &SqlitePool,
    id: String,
    input: UpdateProfileInput,
) -> Result<ProfileResponse, String> {
    let current = sqlx::query_as::<_, Profile>("SELECT * FROM profiles WHERE id = ?")
        .bind(&id)
        .fetch_optional(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Profile not found: {}", id))?;

    let current_env_vars: Vec<EnvVar> =
        serde_json::from_str(&current.env_vars).map_err(|e| e.to_string())?;

    let name = input.name.unwrap_or(current.name);
    let alias = input.alias.unwrap_or(current.alias);
    let env_vars = input.env_vars.unwrap_or(current_env_vars);
    let env_vars_json = serde_json::to_string(&env_vars).map_err(|e| e.to_string())?;

    sqlx::query(
        "UPDATE profiles SET name = ?, alias = ?, env_vars = ?, updated_at = datetime('now') WHERE id = ?",
    )
    .bind(&name)
    .bind(&alias)
    .bind(&env_vars_json)
    .bind(&id)
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;

    let updated = sqlx::query_as::<_, Profile>("SELECT * FROM profiles WHERE id = ?")
        .bind(&id)
        .fetch_one(db)
        .await
        .map_err(|e| e.to_string())?;
    updated.to_response()
}

pub async fn delete_profile_by_pool(db: &SqlitePool, id: String) -> Result<(), String> {
    let result = sqlx::query("DELETE FROM profiles WHERE id = ?")
        .bind(&id)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;
    if result.rows_affected() == 0 {
        return Err(format!("Profile not found: {}", id));
    }
    Ok(())
}

#[tauri::command]
pub async fn update_profile(
    db: State<'_, SqlitePool>,
    id: String,
    input: UpdateProfileInput,
) -> Result<ProfileResponse, String> {
    update_profile_by_pool(db.inner(), id, input).await
}

#[tauri::command]
pub async fn delete_profile(db: State<'_, SqlitePool>, id: String) -> Result<(), String> {
    delete_profile_by_pool(db.inner(), id).await
}

pub fn generate_alias_block(profiles: &[ProfileResponse]) -> String {
    let start = "# === CLAUDE_CODE_ALIAS_START (DO NOT EDIT MANUALLY) ===";
    let end = "# === CLAUDE_CODE_ALIAS_END ===";
    let mut lines = vec![start.to_string()];
    for p in profiles {
        let env_part: Vec<String> = p
            .env_vars
            .iter()
            .map(|e| format!("{}={}", e.key, e.value))
            .collect();
        let alias_line = if env_part.is_empty() {
            format!("alias {}=\"claude\"", p.alias)
        } else {
            format!("alias {}=\"{} claude\"", p.alias, env_part.join(" "))
        };
        lines.push(alias_line);
    }
    lines.push(end.to_string());
    lines.join("\n")
}

pub fn replace_marker_section(content: &str, new_block: &str) -> String {
    let start_marker = "# === CLAUDE_CODE_ALIAS_START (DO NOT EDIT MANUALLY) ===";
    let end_marker = "# === CLAUDE_CODE_ALIAS_END ===";

    if let Some(start_pos) = content.find(start_marker) {
        if let Some(end_pos) = content.find(end_marker) {
            let before = &content[..start_pos];
            let after = &content[end_pos + end_marker.len()..];
            return format!("{}{}{}", before, new_block, after);
        }
    }
    // No markers found — append
    if content.ends_with('\n') {
        format!("{}\n{}\n", content, new_block)
    } else {
        format!("{}\n\n{}\n", content, new_block)
    }
}

pub fn resolve_zshrc_path(use_real_zshrc: bool, workspace_dir: &Path, home_dir: &Path) -> PathBuf {
    if use_real_zshrc {
        home_dir.join(".zshrc")
    } else {
        workspace_dir.join(".zshrc")
    }
}

pub fn write_alias_block_to_path(
    target_path: &Path,
    block: &str,
    create_backup: bool,
) -> Result<(), String> {
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let content = if target_path.exists() {
        std::fs::read_to_string(target_path).map_err(|e| e.to_string())?
    } else {
        String::new()
    };

    if create_backup && target_path.exists() {
        let file_name = target_path
            .file_name()
            .ok_or("Invalid zshrc target path")?
            .to_string_lossy();
        let backup_path = target_path.with_file_name(format!("{}.bak", file_name));
        std::fs::copy(target_path, backup_path).map_err(|e| e.to_string())?;
    }

    let new_content = replace_marker_section(&content, block);
    std::fs::write(target_path, new_content).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn sync_profiles_to_zshrc(
    db: State<'_, SqlitePool>,
    use_real_zshrc: Option<bool>,
) -> Result<SyncProfilesResult, String> {
    let profiles = list_profiles_by_pool(db.inner()).await?;
    let block = generate_alias_block(&profiles);

    let home = dirs::home_dir().ok_or("Could not determine home directory")?;
    let workspace_dir = std::env::current_dir().map_err(|e| e.to_string())?;
    let is_real_zshrc = use_real_zshrc.unwrap_or(false);
    let zshrc_path = resolve_zshrc_path(is_real_zshrc, &workspace_dir, &home);

    write_alias_block_to_path(&zshrc_path, &block, is_real_zshrc)?;

    let message = if is_real_zshrc {
        format!("Synced {} profile(s) to ~/.zshrc", profiles.len())
    } else {
        format!(
            "Synced {} profile(s) to workspace mock .zshrc",
            profiles.len()
        )
    };

    Ok(SyncProfilesResult {
        message,
        target_path: zshrc_path.display().to_string(),
        is_real_zshrc,
    })
}

#[cfg(test)]
mod tests {
    use sqlx::SqlitePool;

    use super::*;

    async fn setup_db() -> SqlitePool {
        let db = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect sqlite memory");
        sqlx::query(
            "CREATE TABLE profiles (id TEXT PRIMARY KEY, name TEXT NOT NULL, alias TEXT NOT NULL UNIQUE, env_vars TEXT NOT NULL DEFAULT '[]', created_at TEXT NOT NULL DEFAULT (datetime('now')), updated_at TEXT NOT NULL DEFAULT (datetime('now')))",
        )
        .execute(&db)
        .await
        .expect("create profiles table");
        db
    }

    #[tokio::test]
    async fn list_profiles_returns_empty_when_no_data() {
        let db = setup_db().await;
        let profiles = list_profiles_by_pool(&db).await.expect("list profiles");
        assert!(profiles.is_empty());
    }

    #[tokio::test]
    async fn create_and_list_profile() {
        let db = setup_db().await;
        let input = CreateProfileInput {
            name: "Leo".to_string(),
            alias: "ccleo".to_string(),
            env_vars: vec![EnvVar {
                key: "ANTHROPIC_BASE_URL".to_string(),
                value: "https://example.com".to_string(),
            }],
        };
        let created = create_profile_by_pool(&db, input)
            .await
            .expect("create profile");
        assert_eq!(created.name, "Leo");
        assert_eq!(created.alias, "ccleo");
        assert_eq!(created.env_vars.len(), 1);
        assert_eq!(created.env_vars[0].key, "ANTHROPIC_BASE_URL");

        let profiles = list_profiles_by_pool(&db).await.expect("list profiles");
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].alias, "ccleo");
    }

    #[tokio::test]
    async fn update_profile_partial_fields() {
        let db = setup_db().await;
        let created = create_profile_by_pool(
            &db,
            CreateProfileInput {
                name: "Leo".to_string(),
                alias: "ccleo".to_string(),
                env_vars: vec![EnvVar {
                    key: "KEY".to_string(),
                    value: "val".to_string(),
                }],
            },
        )
        .await
        .expect("create");

        let updated = update_profile_by_pool(
            &db,
            created.id,
            UpdateProfileInput {
                name: Some("Leo Updated".to_string()),
                alias: None,
                env_vars: None,
            },
        )
        .await
        .expect("update");

        assert_eq!(updated.name, "Leo Updated");
        assert_eq!(updated.alias, "ccleo"); // unchanged
        assert_eq!(updated.env_vars.len(), 1); // unchanged
    }

    #[tokio::test]
    async fn delete_profile_removes_it() {
        let db = setup_db().await;
        let created = create_profile_by_pool(
            &db,
            CreateProfileInput {
                name: "ToDelete".to_string(),
                alias: "ccdel".to_string(),
                env_vars: vec![],
            },
        )
        .await
        .expect("create");

        delete_profile_by_pool(&db, created.id)
            .await
            .expect("delete");

        let profiles = list_profiles_by_pool(&db).await.expect("list");
        assert!(profiles.is_empty());
    }

    #[tokio::test]
    async fn delete_nonexistent_profile_returns_error() {
        let db = setup_db().await;
        let result = delete_profile_by_pool(&db, "nonexistent".to_string()).await;
        assert!(result.is_err());
    }

    #[test]
    fn generate_alias_block_single_profile() {
        let profiles = vec![ProfileResponse {
            id: "1".to_string(),
            name: "Leo".to_string(),
            alias: "ccleo".to_string(),
            env_vars: vec![
                EnvVar {
                    key: "ANTHROPIC_BASE_URL".to_string(),
                    value: "https://example.com".to_string(),
                },
                EnvVar {
                    key: "ANTHROPIC_AUTH_TOKEN".to_string(),
                    value: "sk-xxx".to_string(),
                },
            ],
            created_at: "".to_string(),
            updated_at: "".to_string(),
        }];
        let block = generate_alias_block(&profiles);
        assert!(block.starts_with("# === CLAUDE_CODE_ALIAS_START"));
        assert!(block.contains(
            "alias ccleo=\"ANTHROPIC_BASE_URL=https://example.com ANTHROPIC_AUTH_TOKEN=sk-xxx claude\""
        ));
        assert!(block.ends_with("# === CLAUDE_CODE_ALIAS_END ==="));
    }

    #[test]
    fn generate_alias_block_empty() {
        let block = generate_alias_block(&[]);
        assert!(block.contains("CLAUDE_CODE_ALIAS_START"));
        assert!(block.contains("CLAUDE_CODE_ALIAS_END"));
        // Only start + end markers, no alias lines
        assert_eq!(block.lines().count(), 2);
    }

    #[test]
    fn replace_marker_section_existing() {
        let content = "some stuff\n# === CLAUDE_CODE_ALIAS_START (DO NOT EDIT MANUALLY) ===\nold alias\n# === CLAUDE_CODE_ALIAS_END ===\nmore stuff";
        let new_block = "# === CLAUDE_CODE_ALIAS_START (DO NOT EDIT MANUALLY) ===\nnew alias\n# === CLAUDE_CODE_ALIAS_END ===";
        let result = replace_marker_section(content, new_block);
        assert!(result.contains("some stuff\n"));
        assert!(result.contains("new alias"));
        assert!(!result.contains("old alias"));
        assert!(result.contains("\nmore stuff"));
    }

    #[test]
    fn replace_marker_section_no_markers() {
        let content = "existing content\n";
        let new_block = "# === CLAUDE_CODE_ALIAS_START (DO NOT EDIT MANUALLY) ===\nalias cc=\"claude\"\n# === CLAUDE_CODE_ALIAS_END ===";
        let result = replace_marker_section(content, new_block);
        assert!(result.starts_with("existing content\n"));
        assert!(result.contains("CLAUDE_CODE_ALIAS_START"));
        assert!(result.contains("alias cc="));
    }

    #[test]
    fn resolve_zshrc_path_uses_workspace_file_when_real_mode_is_off() {
        let workspace = std::path::PathBuf::from("/tmp/toolkit-workspace");
        let home = std::path::PathBuf::from("/Users/tester");
        let target = resolve_zshrc_path(false, &workspace, &home);
        assert_eq!(target, workspace.join(".zshrc"));
    }

    #[test]
    fn resolve_zshrc_path_uses_home_file_when_real_mode_is_on() {
        let workspace = std::path::PathBuf::from("/tmp/toolkit-workspace");
        let home = std::path::PathBuf::from("/Users/tester");
        let target = resolve_zshrc_path(true, &workspace, &home);
        assert_eq!(target, home.join(".zshrc"));
    }

    #[test]
    fn write_alias_block_creates_backup_only_in_real_mode() {
        let temp_root =
            std::env::temp_dir().join(format!("toolkit-sync-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_root).expect("create temp root");
        let target = temp_root.join(".zshrc");
        std::fs::write(&target, "existing\n").expect("write existing zshrc");
        let block =
            "# === CLAUDE_CODE_ALIAS_START (DO NOT EDIT MANUALLY) ===\nalias cc='claude'\n# === CLAUDE_CODE_ALIAS_END ===";

        write_alias_block_to_path(&target, block, true).expect("write real mode");
        assert!(temp_root.join(".zshrc.bak").exists());

        std::fs::write(&target, "existing-again\n").expect("reset zshrc");
        let _ = std::fs::remove_file(temp_root.join(".zshrc.bak"));
        write_alias_block_to_path(&target, block, false).expect("write mock mode");
        assert!(!temp_root.join(".zshrc.bak").exists());

        std::fs::remove_dir_all(&temp_root).expect("cleanup temp root");
    }
}
