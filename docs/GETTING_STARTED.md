# Getting Started

`google-admin-mcp` is a local Google auth helper MCP. It inspects local
Google Cloud SDK and ADC state, builds safe commands, validates OAuth client
JSON shape without returning secrets, and performs read-only smoke checks.

It does not bypass Google authentication and does not copy credentials to remote
hosts in v1.

## Prerequisites

- Rust toolchain compatible with edition 2024.
- Google Cloud SDK (`gcloud`) installed for local auth inspection and command
  execution.
- A Google Cloud project suitable for OAuth/quota when validating GA4 access.
- A downloaded Desktop OAuth client JSON if the default `gcloud` OAuth client is
  blocked for the required scopes.

## Run

```bash
cargo run --release --bin google-admin-mcp
```

Optional configuration:

```bash
export GOOGLE_ADMIN_MCP_PROFILE=read_only
export GOOGLE_ADMIN_MCP_DEFAULT_QUOTA_PROJECT=<YOUR_GCP_PROJECT_ID>
```

The default `read_only` profile is the intended v1 posture. The `operator`
profile exists as an explicit future gate for mutating tools, but v1 tool
handlers remain inspect/plan/verify oriented.

## Typical Flow

1. `google_auth_status` to inspect local `gcloud` and ADC state.
2. `find_tools` when a client needs deferred discovery metadata.
3. `google_oauth_client_file_validate` for a downloaded OAuth client JSON.
4. `google_adc_login_command` to generate the exact `gcloud` command.
5. Run the generated command outside the MCP and complete Google consent.
6. `google_ga4_account_summaries_smoke` to verify GA4 read-only access.

## ADC Login Command

The helper can produce a command like:

```bash
gcloud auth application-default login \
  --no-launch-browser \
  --client-id-file /path/to/client.json \
  --scopes=https://www.googleapis.com/auth/analytics.readonly,https://www.googleapis.com/auth/cloud-platform
```

The command writes ADC on the machine where it is run. The MCP returns the
command and notes; it does not return OAuth tokens.

## Local Verification

```bash
cargo fmt --all --check
cargo test
cargo run -- --print-tools
```

For docs-only changes, `git diff --check` is usually enough.
