use std::path::Path;
use std::time::Duration;

use tokio::process::Command;

use crate::core::error::AppError;

pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

pub struct ShellRunner;

impl ShellRunner {
    pub async fn execute(program: &Path, args: &[&str]) -> Result<CommandOutput, AppError> {
        Self::execute_with_timeout(program, args, Duration::from_secs(30)).await
    }

    pub async fn execute_with_timeout(
        program: &Path,
        args: &[&str],
        timeout: Duration,
    ) -> Result<CommandOutput, AppError> {
        let output = tokio::time::timeout(
            timeout,
            Command::new(program)
                .args(args)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .output(),
        )
        .await
        .map_err(|_| AppError::internal_error("命令执行超时"))?
        .map_err(|e| {
            AppError::internal_error(&format!("命令执行失败: {e}"))
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(CommandOutput {
            success: output.status.success(),
            stdout,
            stderr,
        })
    }
}
