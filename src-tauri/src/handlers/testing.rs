use crate::agents::atom_executor::{AtomExecutor, AtomInput};
use crate::maker_core::SpawnFlags;
use crate::{AtomType, LlmConfig};
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestFrameworkInfo {
    pub framework: String,
    pub test_command: String,
    pub test_pattern: String,
}

#[tauri::command]
pub async fn detect_test_framework(workspace_path: String) -> Result<TestFrameworkInfo, String> {
    use std::path::Path;
    let ws = Path::new(&workspace_path);

    if ws.join("Cargo.toml").exists() {
        return Ok(TestFrameworkInfo {
            framework: "rust-cargo".to_string(),
            test_command: "cargo test".to_string(),
            test_pattern: "#[test]".to_string(),
        });
    }

    // ... (simplified detection for brevity, assuming standard stacks)
    if ws.join("package.json").exists() {
        return Ok(TestFrameworkInfo {
            framework: "node".to_string(),
            test_command: "npm test".to_string(),
            test_pattern: "*.test.js".to_string(),
        });
    }

    Ok(TestFrameworkInfo {
        framework: "unknown".to_string(),
        test_command: "echo 'No test framework detected'".to_string(),
        test_pattern: "".to_string(),
    })
}

#[tauri::command]
pub async fn generate_tests(
    workspace_path: String,
    source_file: String,
    test_type: Option<String>,
) -> Result<serde_json::Value, String> {
    let test_type = test_type.unwrap_or_else(|| "unit".to_string());

    let source_path = std::path::Path::new(&workspace_path).join(&source_file);
    let source_content = std::fs::read_to_string(&source_path)
        .map_err(|e| format!("Failed to read source: {}", e))?;

    // Use rust-embed to load prompt template
    let prompt_template =
        get_prompt("generate_tests.md").ok_or("Failed to load generate_tests.md prompt asset")?;

    // Basic template replacement (could use a crate like tinytemplate, but distinct handling is fine for now)
    let language = if source_file.ends_with(".rs") {
        "rust"
    } else {
        "unknown"
    };
    let prompt = prompt_template
        .replace("{{test_type}}", &test_type)
        .replace("{{language}}", language)
        .replace("{{source_file}}", &source_file)
        .replace("{{source_content}}", &source_content);

    let input = AtomInput::new(AtomType::Tester, &prompt).with_flags(SpawnFlags {
        require_json: false,
        temperature: 0.3,
        max_tokens: Some(4000),
        red_flag_check: false, // Don't check red flags on test generation itself
    });

    let config = LlmConfig::cerebras(); // Default to Cerebras for speed
    let executor = AtomExecutor::new(config);
    let result = executor
        .execute(input)
        .await
        .map_err(|e| format!("Test gen failed: {}", e))?;

    Ok(serde_json::json!({
        "test_code": result.output,
        "language": language
    }))
}
