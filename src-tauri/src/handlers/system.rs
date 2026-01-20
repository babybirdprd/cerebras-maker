use crate::{get_settings_path, AppSettings, ExecutionMetrics, EXECUTION_METRICS};
// Note: We are still referencing globals from lib.rs for now to minimize breakage
// during the initial split. Ideally these should be moving to Tauri State.

#[tauri::command]
pub fn get_cwd() -> Result<String, String> {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| format!("Failed to get cwd: {}", e))
}

#[tauri::command]
pub fn get_execution_metrics() -> Result<ExecutionMetrics, String> {
    let metrics = EXECUTION_METRICS
        .lock()
        .map_err(|_| "Failed to acquire metrics lock")?;
    Ok(metrics.clone())
}

#[tauri::command]
pub fn save_settings(settings: AppSettings) -> Result<(), String> {
    let path = get_settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    std::fs::write(&path, &json).map_err(|e| format!("Failed to write settings: {}", e))?;

    // Update global cache if we can access it (defined in lib.rs)
    // For now, simpler to just write to disk.
    Ok(())
}

#[tauri::command]
pub fn load_settings() -> Result<AppSettings, String> {
    let path = get_settings_path();
    if !path.exists() {
        return Err("Settings file not found".to_string());
    }
    let json =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read settings: {}", e))?;
    serde_json::from_str(&json).map_err(|e| format!("Failed to parse settings: {}", e))
}
