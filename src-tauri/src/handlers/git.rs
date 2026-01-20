use crate::SHADOW_GIT;
use std::process::Command;

#[tauri::command]
pub fn create_snapshot(message: String) -> Result<serde_json::Value, String> {
    if message.trim().is_empty() {
        return Err("Snapshot message cannot be empty".to_string());
    }

    let mut sg = SHADOW_GIT
        .lock()
        .map_err(|_| "Failed to acquire shadow git lock")?;

    let shadow_git = sg.as_mut().ok_or("Shadow Git not initialized")?;

    let snapshot = shadow_git
        .snapshot(&message)
        .map_err(|e| format!("Failed to create: {}", e))?;

    Ok(serde_json::json!({
        "id": snapshot.id,
        "message": snapshot.message,
        "timestamp_ms": snapshot.timestamp_ms,
        "commit_hash": snapshot.commit_hash
    }))
}

#[tauri::command]
pub fn rollback_snapshot() -> Result<String, String> {
    let mut sg = SHADOW_GIT
        .lock()
        .map_err(|_| "Failed to acquire shadow git lock")?;

    let shadow_git = sg.as_mut().ok_or("Shadow Git not initialized")?;

    shadow_git
        .rollback()
        .map_err(|e| format!("Rollback failed: {}", e))?;

    Ok("Rolled back to previous snapshot".to_string())
}

#[tauri::command]
pub fn get_git_history(workspace_path: String, limit: usize) -> Result<serde_json::Value, String> {
    let output = Command::new("git")
        .current_dir(&workspace_path)
        .args(["log", "--oneline", "-n", &limit.to_string()])
        .output()
        .map_err(|e| format!("Failed to get git history: {}", e))?;

    let entries: Vec<serde_json::Value> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            serde_json::json!({
                "hash": parts.first().unwrap_or(&""),
                "message": parts.get(1).unwrap_or(&"")
            })
        })
        .collect();

    Ok(serde_json::json!(entries))
}
