use mcp_toolkit_core::tool_inventory::{
    ToolCapability, ToolDiscoveryMetadata, ToolInventory, ToolInventoryError,
};

pub(crate) fn build_tool_inventory() -> Result<ToolInventory, ToolInventoryError> {
    ToolInventory::from_capabilities(tool_capabilities())
}

fn tool_capabilities() -> Vec<ToolCapability> {
    vec![
        cap(
            "find_tools",
            "discovery",
            true,
            "Search Google admin MCP tools for OpenAI tool_search and deferred-loading clients.",
            [
                "tool_search",
                "deferred",
                "discover",
                "tools",
                "gpt-5.5",
                "openai",
            ],
        ),
        cap(
            "google_auth_status",
            "auth",
            true,
            "Inspect local gcloud and Application Default Credentials state without exposing tokens.",
            [
                "google",
                "auth",
                "adc",
                "gcloud",
                "status",
                "credentials",
                "token",
            ],
        ),
        cap(
            "google_adc_login_command",
            "auth",
            true,
            "Build the exact gcloud ADC login command for a dedicated OAuth client JSON.",
            [
                "google",
                "auth",
                "adc",
                "login",
                "oauth",
                "client",
                "scopes",
                "analytics",
                "ga4",
                "token",
                "credential",
            ],
        ),
        cap(
            "google_oauth_client_file_validate",
            "auth",
            true,
            "Validate a Google OAuth client JSON file while hiding client secrets.",
            [
                "google",
                "oauth",
                "client",
                "json",
                "validate",
                "secret",
                "desktop",
                "token",
                "credential",
            ],
        ),
        cap(
            "google_cloud_projects_list",
            "cloud",
            true,
            "List Google Cloud projects visible to the current gcloud identity.",
            ["google", "cloud", "gcp", "projects", "list", "iam"],
        ),
        cap(
            "google_ga4_account_summaries_smoke",
            "analytics",
            true,
            "Verify GA4 readonly access by calling Analytics Admin accountSummaries with ADC.",
            [
                "google",
                "analytics",
                "ga4",
                "account",
                "summaries",
                "smoke",
                "readonly",
                "quota",
                "token",
                "credential",
                "oauth",
            ],
        ),
    ]
}

fn cap<const N: usize>(
    name: &'static str,
    group: &'static str,
    read_only: bool,
    description: &'static str,
    keywords: [&'static str; N],
) -> ToolCapability {
    ToolCapability::new(name)
        .with_group(group)
        .with_read_only(read_only)
        .with_discovery(ToolDiscoveryMetadata::new(description, keywords))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mcp_toolkit_core::tool_inventory::{ToolInventoryPolicy, ToolOperation, ToolSearchFilter};

    #[test]
    fn inventory_search_finds_ga4_smoke_tool() {
        let inventory = build_tool_inventory().expect("inventory");
        let results = inventory.search(
            &ToolSearchFilter {
                query: Some("ga4 account".to_string()),
                group: None,
                read_only: Some(true),
                limit: Some(10),
            },
            ToolOperation::List,
            &ToolInventoryPolicy::strict(),
        );
        assert!(
            results
                .iter()
                .any(|result| result.name == "google_ga4_account_summaries_smoke")
        );
    }
}
