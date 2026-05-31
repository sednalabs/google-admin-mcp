use std::future::Future;
use std::sync::Arc;

use mcp_toolkit_core::rmcp_models;
use mcp_toolkit_core::tool_inventory::{ToolInventory, ToolInventoryPolicy, ToolOperation};
use mcp_toolkit_core::tool_schema::tool_schema_snapshot_value;
use mcp_toolkit_observability::{EventContext, Level, emit_event, safe_text};
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::tool::ToolCallContext;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Implementation, ListToolsResult, PaginatedRequestParams,
    ProtocolVersion, ServerCapabilities, ServerInfo,
};
use rmcp::service::RequestContext;
use rmcp::{RoleServer, ServerHandler};
use serde_json::Value;

use crate::config::CapabilityProfile;
use crate::contract;
use crate::error::GoogleAdminError;
use crate::google_api::GoogleApi;
use crate::google_cli::GoogleCli;
use crate::tool_surface::build_tool_inventory;

#[derive(Clone)]
pub struct GoogleAdminMcp {
    pub cli: Arc<GoogleCli>,
    pub api: Arc<GoogleApi>,
    pub profile: CapabilityProfile,
    pub default_quota_project: Option<String>,
    pub(crate) tool_inventory: ToolInventory,
    pub(crate) tool_inventory_policy: ToolInventoryPolicy,
    tool_router: ToolRouter<GoogleAdminMcp>,
}

impl GoogleAdminMcp {
    pub fn new(profile: CapabilityProfile, default_quota_project: Option<String>) -> Self {
        let tool_inventory =
            build_tool_inventory().expect("google-admin-mcp tool inventory should build");
        Self {
            cli: Arc::new(GoogleCli::default()),
            api: Arc::new(GoogleApi::default()),
            profile,
            default_quota_project,
            tool_inventory,
            tool_inventory_policy: ToolInventoryPolicy::strict(),
            tool_router: Self::tool_router_google_admin(),
        }
    }

    pub fn tool_names(&self) -> Vec<String> {
        self.tool_router
            .list_all()
            .into_iter()
            .map(|tool| tool.name.to_string())
            .collect()
    }

    pub fn tool_schema_snapshot(&self) -> Value {
        tool_schema_snapshot_value(&self.tool_router.list_all())
            .expect("registered tool definitions should serialize")
    }

    fn is_tool_allowed(&self, tool_name: &str) -> bool {
        if tool_is_mutating(tool_name) {
            self.profile.allows_mutation()
        } else {
            true
        }
    }
}

impl ServerHandler for GoogleAdminMcp {
    fn get_info(&self) -> ServerInfo {
        rmcp_models::server_info(
            ProtocolVersion::V_2024_11_05,
            ServerCapabilities::builder().enable_tools().build(),
            Implementation::from_build_env(),
            Some(
                "Google admin/auth helper MCP. Centralizes local Google credential inspection, planning, and read-only smoke verification."
                    .to_string(),
            ),
        )
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, rmcp::ErrorData>> + Send + '_ {
        let tools = self.tool_inventory.filter_tools(
            self.tool_router.list_all(),
            ToolOperation::List,
            &self.tool_inventory_policy,
            |tool| tool.name.as_ref(),
        );
        std::future::ready(Ok(ListToolsResult {
            meta: None,
            tools,
            next_cursor: None,
        }))
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResult, rmcp::ErrorData>> + Send + '_ {
        let tool_name = request.name.to_string();
        let tool_context = ToolCallContext::new(self, request, context);
        async move {
            let registered_allowed = self.tool_inventory.is_allowed(
                &tool_name,
                ToolOperation::Call,
                &self.tool_inventory_policy,
            );
            if !registered_allowed || !self.is_tool_allowed(&tool_name) {
                let err = GoogleAdminError::PolicyDenied {
                    profile: self.profile.as_str().to_string(),
                    tool: tool_name.clone(),
                };
                return Ok(contract::error(err, std::time::Instant::now()));
            }

            emit_event(
                Level::INFO,
                "google_admin_mcp.tool.start",
                &EventContext::new().with_tool_name(&tool_name),
                &[
                    safe_text("tool", &tool_name),
                    safe_text("profile", self.profile.as_str()),
                ],
            );
            let result = self.tool_router.call(tool_context).await;
            emit_event(
                Level::INFO,
                "google_admin_mcp.tool.finish",
                &EventContext::new().with_tool_name(&tool_name),
                &[safe_text("tool", &tool_name)],
            );
            result
        }
    }
}

fn tool_is_mutating(tool_name: &str) -> bool {
    matches!(tool_name, "google_adc_revoke")
}
