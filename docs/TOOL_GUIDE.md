# Tool Guide

All tools return Contract V1 envelopes: `ok/data/meta` on success and
`ok/error/meta` on failure.

## Discovery

| Tool | Purpose |
|---|---|
| `find_tools` | Search tool metadata by keyword, group, read-only flag, and optional schema inclusion. |

`find_tools` is intended for OpenAI `tool_search` and deferred-loading clients.
Use `include_schema=true` only when the caller needs matching MCP schemas.

## Auth Inspection and Planning

| Tool | Purpose |
|---|---|
| `google_auth_status` | Inspect local `gcloud` and ADC state without exposing tokens. |
| `google_adc_login_command` | Build a safe `gcloud auth application-default login` command. |
| `google_oauth_client_file_validate` | Validate OAuth client JSON metadata without returning secrets. |

`google_auth_status` can include `gcloud auth list` output when
`include_gcloud_accounts=true`. It summarizes ADC state without returning the
raw credential file.

`google_adc_login_command` defaults to headless-friendly browser behavior and
returns both an argv array and shell string. Run the command outside the MCP and
complete Google consent there.

## Cloud and GA4 Verification

| Tool | Purpose |
|---|---|
| `google_cloud_projects_list` | List Google Cloud projects visible to the current `gcloud` identity. |
| `google_ga4_account_summaries_smoke` | Verify GA4 read-only access with the current ADC token. |

`google_ga4_account_summaries_smoke` accepts an optional `quota_project` and
`page_size`. It is a live Google API call and should be run only when validating
real ADC access.

## Common Flow

1. Inspect with `google_auth_status`.
2. Validate the OAuth client JSON with `google_oauth_client_file_validate`.
3. Generate the ADC login command with `google_adc_login_command`.
4. Run the command outside the MCP.
5. Verify with `google_ga4_account_summaries_smoke`.

If the smoke test fails with a Google `403`, the ADC identity may lack GA4
account/property access or the quota project may be missing/misconfigured.
