use thiserror::Error;

#[derive(Debug, Error)]
pub enum GoogleAdminError {
    #[error("invalid {field}: {message}")]
    InvalidArgument {
        field: &'static str,
        message: String,
    },

    #[error("required command not found: {0}")]
    MissingCommand(String),

    #[error("command failed: {command}: {message}")]
    CommandFailed { command: String, message: String },

    #[error("authentication is not configured: {0}")]
    AuthNotConfigured(String),

    #[error("upstream API request failed with status {status}: {message}")]
    UpstreamApi { status: u16, message: String },

    #[error("failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("http transport error: {0}")]
    Transport(#[from] reqwest::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("tool '{tool}' is blocked by capability profile '{profile}'")]
    PolicyDenied { profile: String, tool: String },
}

impl GoogleAdminError {
    pub fn invalid(field: &'static str, message: impl Into<String>) -> Self {
        Self::InvalidArgument {
            field,
            message: message.into(),
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidArgument { .. } => "INVALID_PARAMS",
            Self::MissingCommand(_) => "LOCAL_DEPENDENCY_MISSING",
            Self::CommandFailed { .. } => "LOCAL_COMMAND_FAILED",
            Self::AuthNotConfigured(_) => "AUTHENTICATION_REQUIRED",
            Self::UpstreamApi { status, .. } if *status >= 500 => "UPSTREAM_UNAVAILABLE",
            Self::UpstreamApi { .. } => "UPSTREAM_REJECTED",
            Self::Json(_) => "JSON_PARSE_ERROR",
            Self::Transport(_) => "UPSTREAM_TRANSPORT_ERROR",
            Self::Io(_) => "IO_ERROR",
            Self::PolicyDenied { .. } => "POLICY_DENIED",
        }
    }

    pub fn reason(&self) -> &'static str {
        match self {
            Self::InvalidArgument { .. } => "invalid_params",
            Self::MissingCommand(_) => "missing_command",
            Self::CommandFailed { .. } => "command_failed",
            Self::AuthNotConfigured(_) => "auth_not_configured",
            Self::UpstreamApi { status, .. } if *status >= 500 => "upstream_unavailable",
            Self::UpstreamApi { .. } => "upstream_rejected",
            Self::Json(_) => "json_parse",
            Self::Transport(_) => "transport",
            Self::Io(_) => "io",
            Self::PolicyDenied { .. } => "policy_denied",
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            Self::InvalidArgument { .. } => "validation",
            Self::MissingCommand(_) | Self::CommandFailed { .. } => "local_command",
            Self::AuthNotConfigured(_) => "auth",
            Self::UpstreamApi { .. } => "upstream_api",
            Self::Json(_) => "parse",
            Self::Transport(_) => "transport",
            Self::Io(_) => "io",
            Self::PolicyDenied { .. } => "policy",
        }
    }
}
