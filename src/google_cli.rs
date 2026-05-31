use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use serde::Deserialize;
use serde_json::{Value, json};
use tokio::process::Command;
use tokio::time::timeout;

use crate::error::GoogleAdminError;

const DEFAULT_COMMAND_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug, Clone, Deserialize)]
pub struct AdcFile {
    #[serde(rename = "type")]
    pub credential_type: Option<String>,
    pub client_id: Option<String>,
    pub quota_project_id: Option<String>,
    pub universe_domain: Option<String>,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GoogleCli {
    gcloud: String,
}

impl Default for GoogleCli {
    fn default() -> Self {
        Self {
            gcloud: "gcloud".to_string(),
        }
    }
}

impl GoogleCli {
    pub fn new(gcloud: impl Into<String>) -> Self {
        Self {
            gcloud: gcloud.into(),
        }
    }

    pub fn adc_path() -> Option<PathBuf> {
        let config_home = std::env::var_os("CLOUDSDK_CONFIG")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config/gcloud"))
            })?;
        Some(config_home.join("application_default_credentials.json"))
    }

    pub fn read_adc_file(path: &Path) -> Result<Option<AdcFile>, GoogleAdminError> {
        if !path.exists() {
            return Ok(None);
        }
        let raw = std::fs::read_to_string(path)?;
        let parsed = serde_json::from_str(&raw)?;
        Ok(Some(parsed))
    }

    pub async fn auth_list(&self) -> Result<Value, GoogleAdminError> {
        let output = self
            .run_json(["auth", "list", "--format=json"], DEFAULT_COMMAND_TIMEOUT)
            .await?;
        Ok(output)
    }

    pub async fn projects_list(&self, limit: Option<u32>) -> Result<Value, GoogleAdminError> {
        let limit = limit.unwrap_or(50).clamp(1, 200).to_string();
        let output = self
            .run_json(
                [
                    "projects",
                    "list",
                    "--format=json",
                    "--limit",
                    limit.as_str(),
                ],
                DEFAULT_COMMAND_TIMEOUT,
            )
            .await?;
        Ok(output)
    }

    pub async fn adc_access_token(&self) -> Result<String, GoogleAdminError> {
        let output = self
            .run_text(
                ["auth", "application-default", "print-access-token"],
                DEFAULT_COMMAND_TIMEOUT,
            )
            .await?;
        let token = output.trim();
        if token.is_empty() {
            return Err(GoogleAdminError::AuthNotConfigured(
                "gcloud returned an empty ADC access token".to_string(),
            ));
        }
        Ok(token.to_string())
    }

    pub fn adc_login_command(
        client_id_file: Option<&str>,
        scopes: &[String],
        no_launch_browser: bool,
    ) -> Vec<String> {
        let mut command = vec![
            "gcloud".to_string(),
            "auth".to_string(),
            "application-default".to_string(),
            "login".to_string(),
        ];
        if no_launch_browser {
            command.push("--no-launch-browser".to_string());
        }
        if let Some(path) = client_id_file.filter(|value| !value.trim().is_empty()) {
            command.push("--client-id-file".to_string());
            command.push(path.trim().to_string());
        }
        if !scopes.is_empty() {
            command.push("--scopes".to_string());
            command.push(scopes.join(","));
        }
        command
    }

    async fn run_json<I, S>(
        &self,
        args: I,
        timeout_duration: Duration,
    ) -> Result<Value, GoogleAdminError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let text = self.run_text(args, timeout_duration).await?;
        serde_json::from_str(&text).map_err(GoogleAdminError::Json)
    }

    async fn run_text<I, S>(
        &self,
        args: I,
        timeout_duration: Duration,
    ) -> Result<String, GoogleAdminError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut command = Command::new(&self.gcloud);
        for arg in args {
            command.arg(arg.as_ref());
        }
        command.stdin(Stdio::null());
        let command_label = format!("{} {}", self.gcloud, command_label_args(command.as_std()));
        let child = command.output();
        let output = timeout(timeout_duration, child)
            .await
            .map_err(|_| GoogleAdminError::CommandFailed {
                command: command_label.clone(),
                message: "timed out".to_string(),
            })?
            .map_err(|err| {
                if err.kind() == std::io::ErrorKind::NotFound {
                    GoogleAdminError::MissingCommand(self.gcloud.clone())
                } else {
                    GoogleAdminError::Io(err)
                }
            })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let message = if stderr.is_empty() { stdout } else { stderr };
            return Err(GoogleAdminError::CommandFailed {
                command: command_label,
                message,
            });
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

fn command_label_args(command: &std::process::Command) -> String {
    command
        .get_args()
        .map(|arg| arg.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn adc_summary(adc_path: Option<&Path>, adc: Option<&AdcFile>) -> Value {
    match (adc_path, adc) {
        (Some(path), Some(adc)) => json!({
            "path": path.display().to_string(),
            "present": true,
            "credential_type": adc.credential_type,
            "client_id_present": adc.client_id.as_ref().map(|value| !value.is_empty()).unwrap_or(false),
            "refresh_token_present": adc.refresh_token.as_ref().map(|value| !value.is_empty()).unwrap_or(false),
            "quota_project_id": adc.quota_project_id,
            "universe_domain": adc.universe_domain,
        }),
        (Some(path), None) => json!({
            "path": path.display().to_string(),
            "present": false,
        }),
        (None, None) => json!({
            "path": null,
            "present": false,
        }),
        (None, Some(adc)) => json!({
            "path": null,
            "present": true,
            "credential_type": adc.credential_type,
            "client_id_present": adc.client_id.as_ref().map(|value| !value.is_empty()).unwrap_or(false),
            "refresh_token_present": adc.refresh_token.as_ref().map(|value| !value.is_empty()).unwrap_or(false),
            "quota_project_id": adc.quota_project_id,
            "universe_domain": adc.universe_domain,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_command_includes_client_file_and_scopes() {
        let command = GoogleCli::adc_login_command(
            Some("/tmp/client.json"),
            &[
                "https://www.googleapis.com/auth/analytics.readonly".to_string(),
                "https://www.googleapis.com/auth/cloud-platform".to_string(),
            ],
            true,
        );
        assert_eq!(
            command,
            vec![
                "gcloud",
                "auth",
                "application-default",
                "login",
                "--no-launch-browser",
                "--client-id-file",
                "/tmp/client.json",
                "--scopes",
                "https://www.googleapis.com/auth/analytics.readonly,https://www.googleapis.com/auth/cloud-platform",
            ]
        );
    }
}
