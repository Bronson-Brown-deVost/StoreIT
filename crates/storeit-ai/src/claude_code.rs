use async_trait::async_trait;
use storeit_domain::entities::IdentificationResult;
use storeit_domain::errors::{DomainError, Result};
use storeit_domain::storage::ItemIdentifier;
use tokio::process::Command;

use crate::parse::parse_identification_json;
use crate::prompt;

pub struct ClaudeCodeIdentifier {
    claude_path: String,
}

impl ClaudeCodeIdentifier {
    pub fn new(claude_path: String) -> Self {
        Self { claude_path }
    }

    async fn run_claude(
        &self,
        image_data: &[u8],
        mime_type: &str,
        prompt_text: &str,
    ) -> Result<String> {
        // Use the correct file extension so Claude CLI recognizes the image format
        let suffix = match mime_type {
            "image/png" => ".png",
            "image/gif" => ".gif",
            "image/webp" => ".webp",
            "image/heic" => ".heic",
            _ => ".jpg",
        };

        let temp_file = tempfile::Builder::new()
            .suffix(suffix)
            .tempfile()
            .map_err(|e| DomainError::Internal(format!("failed to create temp file: {e}")))?;

        let temp_path = temp_file.path().to_path_buf();

        tokio::fs::write(&temp_path, image_data)
            .await
            .map_err(|e| DomainError::Internal(format!("failed to write temp image: {e}")))?;

        // Include the image path in the prompt so Claude Code uses its Read tool
        // to read the image (positional file args are ignored in -p mode)
        let full_prompt = format!(
            "{prompt_text}\n\nAnalyze this image: {}",
            temp_path.display()
        );

        let output = Command::new(&self.claude_path)
            .env_remove("CLAUDECODE")
            .arg("-p")
            .arg(&full_prompt)
            .arg("--output-format")
            .arg("text")
            .arg("--max-turns")
            .arg("3")
            .arg("--allowedTools")
            .arg("Read")
            .output()
            .await
            .map_err(|e| DomainError::Internal(format!("failed to execute claude CLI: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DomainError::Internal(format!(
                "claude CLI exited with status {}: {stderr}",
                output.status
            )));
        }

        String::from_utf8(output.stdout)
            .map_err(|e| DomainError::Internal(format!("invalid UTF-8 from claude CLI: {e}")))
    }
}

#[async_trait]
impl ItemIdentifier for ClaudeCodeIdentifier {
    async fn identify(&self, image_data: &[u8], mime_type: &str) -> Result<IdentificationResult> {
        let prompt_text = prompt::build_identify_prompt();
        let raw = self.run_claude(image_data, mime_type, &prompt_text).await?;
        tracing::debug!(raw_response = %raw, "claude code identify response");
        parse_identification_json(&raw)
    }

    async fn identify_with_correction(
        &self,
        image_data: &[u8],
        mime_type: &str,
        correction: &str,
    ) -> Result<IdentificationResult> {
        let prompt_text = prompt::build_correction_prompt(correction);
        let raw = self.run_claude(image_data, mime_type, &prompt_text).await?;
        tracing::debug!(raw_response = %raw, "claude code identify_with_correction response");
        parse_identification_json(&raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;

    const VALID_JSON: &str = r#"{"name":"Red Stapler","category":"office supplies","description":"A stapler","aliases":["stapler"],"keywords":["office"],"color":"red","material":"plastic","condition_notes":"good"}"#;

    /// Create a temp shell script that outputs given text to stdout and exits with given code.
    fn make_fake_claude(stdout: &str, exit_code: i32) -> tempfile::NamedTempFile {
        let mut script = tempfile::Builder::new()
            .prefix("fake-claude-")
            .suffix(".sh")
            .tempfile()
            .unwrap();
        writeln!(script, "#!/bin/sh").unwrap();
        // Echo the fixed output regardless of args
        writeln!(script, "cat << 'FAKE_EOF'").unwrap();
        writeln!(script, "{stdout}").unwrap();
        writeln!(script, "FAKE_EOF").unwrap();
        writeln!(script, "exit {exit_code}").unwrap();
        script.flush().unwrap();

        // Make executable
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(script.path(), perms).unwrap();

        script
    }

    /// Create a fake script that writes to stderr and exits with failure.
    fn make_failing_claude(stderr_msg: &str) -> tempfile::NamedTempFile {
        let mut script = tempfile::Builder::new()
            .prefix("fake-claude-fail-")
            .suffix(".sh")
            .tempfile()
            .unwrap();
        writeln!(script, "#!/bin/sh").unwrap();
        writeln!(script, "echo '{stderr_msg}' >&2").unwrap();
        writeln!(script, "exit 1").unwrap();
        script.flush().unwrap();

        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(script.path(), perms).unwrap();

        script
    }

    #[tokio::test]
    async fn identify_success() {
        let script = make_fake_claude(VALID_JSON, 0);
        let identifier = ClaudeCodeIdentifier::new(script.path().to_str().unwrap().to_string());

        let result = identifier
            .identify(b"fake-image", "image/jpeg")
            .await
            .unwrap();
        assert_eq!(result.name, "Red Stapler");
        assert_eq!(result.category.as_deref(), Some("office supplies"));
        assert_eq!(result.color.as_deref(), Some("red"));
    }

    #[tokio::test]
    async fn identify_with_correction_success() {
        let script = make_fake_claude(VALID_JSON, 0);
        let identifier = ClaudeCodeIdentifier::new(script.path().to_str().unwrap().to_string());

        let result = identifier
            .identify_with_correction(b"fake-image", "image/jpeg", "it's a stapler")
            .await
            .unwrap();
        assert_eq!(result.name, "Red Stapler");
    }

    #[tokio::test]
    async fn identify_nonzero_exit_returns_error() {
        let script = make_failing_claude("something went wrong");
        let identifier = ClaudeCodeIdentifier::new(script.path().to_str().unwrap().to_string());

        let err = identifier
            .identify(b"fake-image", "image/jpeg")
            .await
            .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("claude CLI exited with status"), "got: {msg}");
        assert!(msg.contains("something went wrong"), "got: {msg}");
    }

    #[tokio::test]
    async fn identify_invalid_json_returns_error() {
        let script = make_fake_claude("not valid json at all", 0);
        let identifier = ClaudeCodeIdentifier::new(script.path().to_str().unwrap().to_string());

        let err = identifier
            .identify(b"fake-image", "image/jpeg")
            .await
            .unwrap_err();
        assert!(
            err.to_string()
                .contains("failed to parse identification JSON"),
            "got: {}",
            err
        );
    }

    #[tokio::test]
    async fn identify_binary_not_found_returns_error() {
        let identifier = ClaudeCodeIdentifier::new("/nonexistent/path/to/claude".to_string());

        let err = identifier
            .identify(b"fake-image", "image/jpeg")
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("failed to execute claude CLI"),
            "got: {}",
            err
        );
    }

    #[tokio::test]
    async fn identify_fenced_json_output() {
        let fenced = format!("Here is the result:\n```json\n{VALID_JSON}\n```\n");
        let script = make_fake_claude(&fenced, 0);
        let identifier = ClaudeCodeIdentifier::new(script.path().to_str().unwrap().to_string());

        let result = identifier
            .identify(b"fake-image", "image/jpeg")
            .await
            .unwrap();
        assert_eq!(result.name, "Red Stapler");
    }
}
