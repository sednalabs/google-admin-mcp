use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "snake_case")]
pub enum CapabilityProfile {
    ReadOnly,
    Operator,
}

impl CapabilityProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::Operator => "operator",
        }
    }

    pub fn allows_mutation(self) -> bool {
        matches!(self, Self::Operator)
    }
}

#[derive(Debug, Clone, Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// Capability profile. Default is read_only.
    #[arg(long, env = "GOOGLE_ADMIN_MCP_PROFILE", default_value = "read_only")]
    pub profile: CapabilityProfile,

    /// Default Google quota project for ADC-backed API smoke tests.
    #[arg(long, env = "GOOGLE_ADMIN_MCP_DEFAULT_QUOTA_PROJECT")]
    pub default_quota_project: Option<String>,

    /// Print tool names and exit.
    #[arg(long)]
    pub print_tools: bool,

    /// Print tool schema and exit.
    #[arg(long)]
    pub print_tool_schema: bool,
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub profile: CapabilityProfile,
    pub default_quota_project: Option<String>,
    pub print_tools: bool,
    pub print_tool_schema: bool,
}

impl From<Cli> for Settings {
    fn from(cli: Cli) -> Self {
        Self {
            profile: cli.profile,
            default_quota_project: cli.default_quota_project,
            print_tools: cli.print_tools,
            print_tool_schema: cli.print_tool_schema,
        }
    }
}
