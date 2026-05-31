//! Rust MCP for Google credential and admin workflow assistance.

pub mod config;
pub mod contract;
pub mod error;
pub mod google_api;
pub mod google_cli;
pub mod server;
pub mod tool_surface;
pub mod tools;

pub type McpError = rmcp::ErrorData;
