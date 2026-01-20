use crate::{get_llm_provider, llm, RUNTIME};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "prompts/"]
struct Asset;

fn get_prompt(name: &str) -> Option<String> {
    Asset::get(name).map(|file| {
        std::str::from_utf8(file.data.as_ref())
            .unwrap_or_default()
            .to_string()
    })
}

#[tauri::command]
pub async fn analyze_prd(content: String, filename: String) -> Result<serde_json::Value, String> {
    let provider = get_llm_provider()?;

    // Load prompt from external file
    let system_prompt =
        get_prompt("analyze_prd.md").ok_or("Failed to load analyze_prd.md prompt asset")?;

    let user_prompt = format!(
        "I've uploaded a PRD file named '{}'. Here is the content:\n\n{}",
        filename, content
    );

    let messages = vec![
        llm::Message::system(&system_prompt),
        llm::Message::user(&user_prompt),
    ];

    let response = provider
        .complete(messages)
        .await
        .map_err(|e| format!("LLM request failed: {}", e))?;

    Ok(serde_json::json!({
        "status": "analyzed",
        "filename": filename,
        "initial_message": response.content,
        // detected_features removed as we return natural language now per prompt
    }))
}

#[tauri::command]
pub fn execute_script(script: String) -> Result<serde_json::Value, String> {
    if script.trim().is_empty() {
        return Err("Script cannot be empty".to_string());
    }

    let runtime = RUNTIME
        .lock()
        .map_err(|_| "Failed to acquire runtime lock")?;

    let runtime = runtime.as_ref().ok_or("Runtime not initialized")?;

    let result = runtime
        .execute_script(&script)
        .map_err(|e| format!("Execution failed: {}", e))?;

    Ok(serde_json::json!(result.to_string()))
}
