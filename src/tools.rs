use std::path::PathBuf;
use std::time::Instant;

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::tool;
use rmcp::tool_router;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::process::Command;

use crate::contract;
use crate::error::GoogleAdminError;
use crate::google_cli::{GoogleCli, adc_summary};
use crate::server::GoogleAdminMcp;
use mcp_toolkit_core::tool_inventory::{ToolOperation, ToolSearchFilter, ToolSearchResponse};

const GA4_READONLY_SCOPE: &str = "https://www.googleapis.com/auth/analytics.readonly";
const CLOUD_PLATFORM_SCOPE: &str = "https://www.googleapis.com/auth/cloud-platform";

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct AuthStatusArgs {
    /// Include active `gcloud auth list` account information.
    #[serde(default)]
    pub include_gcloud_accounts: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindToolsArgs {
    /// Keyword query matched against tool names, descriptions, and keywords.
    #[serde(default)]
    pub query: Option<String>,
    /// Optional group filter such as auth, cloud, analytics, or discovery.
    #[serde(default)]
    pub group: Option<String>,
    /// Optional read-only filter.
    #[serde(default)]
    pub read_only: Option<bool>,
    /// Maximum result count, 1..100.
    #[serde(default)]
    pub limit: Option<usize>,
    /// Include matching MCP tool schemas in the response.
    #[serde(default)]
    pub include_schema: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AdcLoginCommandArgs {
    /// Optional downloaded OAuth client JSON path.
    pub client_id_file: Option<String>,
    /// Optional scopes. Defaults to GA4 readonly plus cloud-platform.
    pub scopes: Option<Vec<String>>,
    /// Include `--no-launch-browser`. Defaults to true for headless usage.
    pub no_launch_browser: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct OAuthClientFileArgs {
    /// Path to a downloaded Google OAuth client JSON file.
    pub path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProjectListArgs {
    /// Maximum projects to return, 1..200.
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Ga4SmokeArgs {
    /// Quota project for x-goog-user-project. Defaults to MCP config or ADC quota project.
    pub quota_project: Option<String>,
    /// Maximum account summaries to request, 1..200.
    pub page_size: Option<u32>,
}

#[tool_router(router = tool_router_google_admin, vis = "pub")]
impl GoogleAdminMcp {
    /// Search tools for OpenAI tool_search and deferred-loading clients.
    #[tool(
        name = "find_tools",
        description = "Search Google admin MCP tools by keyword, group, and read-only status for OpenAI tool_search/deferred-loading clients."
    )]
    async fn find_tools(
        &self,
        Parameters(args): Parameters<FindToolsArgs>,
    ) -> Result<CallToolResult, crate::McpError> {
        let started = Instant::now();
        let limit = args.limit.unwrap_or(20).clamp(1, 100);
        let filter = ToolSearchFilter {
            query: args.query.clone(),
            group: args.group.clone(),
            read_only: args.read_only,
            limit: Some(limit),
        };
        let results =
            self.tool_inventory
                .search(&filter, ToolOperation::List, &self.tool_inventory_policy);
        let schemas = if args.include_schema {
            let tools = Self::tool_router_google_admin().list_all();
            let mut schema_map = serde_json::Map::new();
            for result in &results {
                if let Some(tool) = tools.iter().find(|tool| tool.name.as_ref() == result.name) {
                    schema_map.insert(result.name.clone(), json!(tool));
                }
            }
            Some(Value::Object(schema_map))
        } else {
            None
        };
        let response =
            ToolSearchResponse::find_tools(args.query, args.group, args.read_only, results)
                .with_schemas(schemas)
                .with_metadata_label("gpt-5.5-compatible tool_search metadata contract");

        Ok(contract::success(response.to_value(), started))
    }

    /// Inspect local Google auth state without returning secrets.
    #[tool(
        name = "google_auth_status",
        description = "Inspect local gcloud and Application Default Credentials state without returning secrets."
    )]
    async fn google_auth_status(
        &self,
        Parameters(args): Parameters<AuthStatusArgs>,
    ) -> Result<CallToolResult, crate::McpError> {
        let started = Instant::now();
        let adc_path = GoogleCli::adc_path();
        let adc = match adc_path.as_deref() {
            Some(path) => match GoogleCli::read_adc_file(path) {
                Ok(value) => value,
                Err(err) => return Ok(contract::error(err, started)),
            },
            None => None,
        };

        let accounts = if args.include_gcloud_accounts {
            match self.cli.auth_list().await {
                Ok(value) => Some(value),
                Err(err) => return Ok(contract::error(err, started)),
            }
        } else {
            None
        };

        Ok(contract::success(
            json!({
                "profile": self.profile.as_str(),
                "gcloud_available": command_exists("gcloud").await,
                "adc": adc_summary(adc_path.as_deref(), adc.as_ref()),
                "gcloud_accounts": accounts,
                "default_quota_project": self.default_quota_project,
            }),
            started,
        ))
    }

    /// Build the exact ADC login command to run with a dedicated OAuth client file.
    #[tool(
        name = "google_adc_login_command",
        description = "Build a gcloud ADC login command for a dedicated Google OAuth client JSON without running it."
    )]
    async fn google_adc_login_command(
        &self,
        Parameters(args): Parameters<AdcLoginCommandArgs>,
    ) -> Result<CallToolResult, crate::McpError> {
        let started = Instant::now();
        let scopes = normalize_scopes(args.scopes);
        let no_launch_browser = args.no_launch_browser.unwrap_or(true);
        let command = GoogleCli::adc_login_command(
            args.client_id_file.as_deref(),
            &scopes,
            no_launch_browser,
        );
        let shell_command = shell_join(&command);
        Ok(contract::success(
            json!({
                "command": command,
                "shell_command": shell_command,
                "scopes": scopes,
                "requires_operator_browser_consent": true,
                "notes": [
                    "Use a downloaded Desktop OAuth client JSON when Google's default gcloud OAuth app is blocked for analytics.readonly.",
                    "The command writes Application Default Credentials on the machine where it is run.",
                    "No token is returned by this tool."
                ],
            }),
            started,
        ))
    }

    /// Validate a Google OAuth client JSON file without returning its secret.
    #[tool(
        name = "google_oauth_client_file_validate",
        description = "Validate a Google OAuth client JSON file and report non-secret metadata."
    )]
    async fn google_oauth_client_file_validate(
        &self,
        Parameters(args): Parameters<OAuthClientFileArgs>,
    ) -> Result<CallToolResult, crate::McpError> {
        let started = Instant::now();
        match validate_oauth_client_file(PathBuf::from(args.path)) {
            Ok(value) => Ok(contract::success(value, started)),
            Err(err) => Ok(contract::error(err, started)),
        }
    }

    /// List visible Google Cloud projects through gcloud.
    #[tool(
        name = "google_cloud_projects_list",
        description = "List Google Cloud projects visible to the current gcloud account."
    )]
    async fn google_cloud_projects_list(
        &self,
        Parameters(args): Parameters<ProjectListArgs>,
    ) -> Result<CallToolResult, crate::McpError> {
        let started = Instant::now();
        match self.cli.projects_list(args.limit).await {
            Ok(projects) => Ok(contract::success(json!({ "projects": projects }), started)),
            Err(err) => Ok(contract::error(err, started)),
        }
    }

    /// Smoke-test GA4 account summaries using the current ADC access token.
    #[tool(
        name = "google_ga4_account_summaries_smoke",
        description = "Verify GA4 readonly access by calling accountSummaries with the current ADC access token."
    )]
    async fn google_ga4_account_summaries_smoke(
        &self,
        Parameters(args): Parameters<Ga4SmokeArgs>,
    ) -> Result<CallToolResult, crate::McpError> {
        let started = Instant::now();
        let adc_path = GoogleCli::adc_path();
        let adc = match adc_path.as_deref() {
            Some(path) => match GoogleCli::read_adc_file(path) {
                Ok(value) => value,
                Err(err) => return Ok(contract::error(err, started)),
            },
            None => None,
        };
        let quota_project = args
            .quota_project
            .or_else(|| self.default_quota_project.clone())
            .or_else(|| adc.as_ref().and_then(|file| file.quota_project_id.clone()));
        let token = match self.cli.adc_access_token().await {
            Ok(token) => token,
            Err(err) => return Ok(contract::error(err, started)),
        };
        match self
            .api
            .ga4_account_summaries(&token, quota_project.as_deref(), args.page_size)
            .await
        {
            Ok(value) => Ok(contract::success_with_meta(
                value,
                json!({ "quota_project_used": quota_project }),
                started,
            )),
            Err(err) => Ok(contract::error(err, started)),
        }
    }
}

fn normalize_scopes(scopes: Option<Vec<String>>) -> Vec<String> {
    let mut scopes = scopes.unwrap_or_else(|| {
        vec![
            GA4_READONLY_SCOPE.to_string(),
            CLOUD_PLATFORM_SCOPE.to_string(),
        ]
    });
    scopes.retain(|scope| !scope.trim().is_empty());
    scopes.sort();
    scopes.dedup();
    scopes
}

fn validate_oauth_client_file(path: PathBuf) -> Result<Value, GoogleAdminError> {
    let raw = std::fs::read_to_string(&path)?;
    let parsed: Value = serde_json::from_str(&raw)?;
    let client = parsed
        .get("installed")
        .or_else(|| parsed.get("web"))
        .ok_or_else(|| GoogleAdminError::invalid("path", "missing installed or web object"))?;
    let client_id_present = client
        .get("client_id")
        .and_then(Value::as_str)
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    let client_secret_present = client
        .get("client_secret")
        .and_then(Value::as_str)
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    let token_uri = client
        .get("token_uri")
        .and_then(Value::as_str)
        .unwrap_or("https://oauth2.googleapis.com/token");
    if !client_id_present {
        return Err(GoogleAdminError::invalid("path", "missing client_id"));
    }
    Ok(json!({
        "path": path.display().to_string(),
        "valid": true,
        "client_type": if parsed.get("installed").is_some() { "installed" } else { "web" },
        "client_id_present": client_id_present,
        "client_secret_present": client_secret_present,
        "token_uri": token_uri,
    }))
}

async fn command_exists(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await
        .map(|status| status.success())
        .unwrap_or(false)
}

fn shell_join(command: &[String]) -> String {
    command
        .iter()
        .map(|part| {
            if part
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || "-_./:=,@".contains(ch))
            {
                part.clone()
            } else {
                format!("'{}'", part.replace('\'', "'\\''"))
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_scopes_defaults_to_ga4_and_cloud() {
        let scopes = normalize_scopes(None);
        assert!(scopes.contains(&GA4_READONLY_SCOPE.to_string()));
        assert!(scopes.contains(&CLOUD_PLATFORM_SCOPE.to_string()));
    }

    #[test]
    fn validate_oauth_client_file_hides_secret() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("client.json");
        std::fs::write(
            &path,
            r#"{"installed":{"client_id":"abc","client_secret":"super-secret","token_uri":"https://oauth2.googleapis.com/token"}}"#,
        )
        .unwrap();
        let value = validate_oauth_client_file(path).unwrap();
        assert_eq!(value["valid"], json!(true));
        assert_eq!(value["client_secret_present"], json!(true));
        assert!(!value.to_string().contains("super-secret"));
    }

    #[test]
    fn shell_join_quotes_spaces() {
        assert_eq!(
            shell_join(&["gcloud".to_string(), "a b".to_string()]),
            "gcloud 'a b'"
        );
    }
}
