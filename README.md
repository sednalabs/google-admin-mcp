# google-admin-mcp

`google-admin-mcp` is a small Rust stdio MCP for reducing Google auth friction
across downstream MCPs. It does not bypass Google authentication. It centralizes
the boring inspect/plan/verify steps so agents do not repeatedly ask an operator
to hand-feed tokens.

## Documentation

- [Getting started](docs/GETTING_STARTED.md)
- [Security model](docs/SECURITY_MODEL.md)
- [Tool guide](docs/TOOL_GUIDE.md)

## V1 scope

- Inspect local `gcloud` and Application Default Credentials state.
- Build safe `gcloud auth application-default login` commands for a dedicated
  OAuth client JSON.
- Validate OAuth client JSON files without returning secrets.
- List visible Google Cloud projects through `gcloud`.
- Smoke-test GA4 account-summary access using the current ADC access token.
- Search tool metadata for OpenAI/GPT-5.5 `tool_search` and deferred-loading
  clients with `find_tools`.
- Keep all tool responses in Contract V1 form: `ok/data/meta` or `ok/error/meta`.

## Safety posture

- Default profile: `read_only`.
- `operator` profile exists for future mutating tools, but v1 does not install or
  copy credentials to remote hosts.
- Secrets are never returned in tool responses. Tokens are used only as in-memory
  bearer credentials for upstream Google API calls.

## Build

```bash
cargo fmt --all --check
cargo test
cargo run -- --print-tools
```

## Run

```bash
cargo run --release --bin google-admin-mcp
```

Optional:

```bash
export GOOGLE_ADMIN_MCP_PROFILE=read_only
export GOOGLE_ADMIN_MCP_DEFAULT_QUOTA_PROJECT=<YOUR_GCP_PROJECT_ID>
```

## Useful flow

1. `google_auth_status`
2. `find_tools` when a client needs to discover the relevant auth/GA4 tools.
3. `google_oauth_client_file_validate` with a downloaded Desktop OAuth client JSON.
4. `google_adc_login_command` to get the exact command to run.
5. Run the command once outside the MCP and complete Google consent.
6. `google_ga4_account_summaries_smoke` to prove GA4 access works.

## Getting a usable GA4 token

The Google Cloud SDK default OAuth app can be blocked for `analytics.readonly`.
When that happens, use a dedicated Desktop OAuth client JSON:

1. Create or choose a Google Cloud project you control.
2. Enable the Google Analytics Admin API and Google Analytics Data API.
3. Configure the OAuth consent screen. If the app is External/testing, add your
   Google account as a test user.
4. Create **Credentials -> OAuth client ID -> Desktop app**.
5. Download the client JSON locally.
6. Run:

```bash
gcloud auth application-default login \
  --no-launch-browser \
  --client-id-file /path/to/client.json \
  --scopes=https://www.googleapis.com/auth/analytics.readonly,https://www.googleapis.com/auth/cloud-platform
```

The MCP can generate this command via `google_adc_login_command` and then verify
the result via `google_ga4_account_summaries_smoke`.

The next step after v1 is adding an operator-gated planner for downstream GA4
MCP credential installation, with apply still requiring an explicit profile
gate.
