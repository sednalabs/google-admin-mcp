use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde_json::Value;

use crate::error::GoogleAdminError;

#[derive(Debug, Clone)]
pub struct GoogleApi {
    http: reqwest::Client,
}

impl Default for GoogleApi {
    fn default() -> Self {
        Self {
            http: reqwest::Client::new(),
        }
    }
}

impl GoogleApi {
    pub async fn ga4_account_summaries(
        &self,
        access_token: &str,
        quota_project: Option<&str>,
        page_size: Option<u32>,
    ) -> Result<Value, GoogleAdminError> {
        let page_size = page_size.unwrap_or(20).clamp(1, 200).to_string();
        let mut headers = HeaderMap::new();
        let bearer = format!("Bearer {access_token}");
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&bearer).map_err(|err| {
                GoogleAdminError::invalid("access_token", format!("invalid bearer token: {err}"))
            })?,
        );
        if let Some(project) = quota_project.filter(|value| !value.trim().is_empty()) {
            headers.insert(
                "x-goog-user-project",
                HeaderValue::from_str(project.trim()).map_err(|err| {
                    GoogleAdminError::invalid(
                        "quota_project",
                        format!("invalid header value: {err}"),
                    )
                })?,
            );
        }
        let response = self
            .http
            .get("https://analyticsadmin.googleapis.com/v1beta/accountSummaries")
            .headers(headers)
            .query(&[("pageSize", page_size.as_str())])
            .send()
            .await?;
        let status = response.status();
        let body = response.text().await?;
        let parsed: Value = serde_json::from_str(&body)?;
        if !status.is_success() {
            let message = parsed
                .pointer("/error/message")
                .and_then(Value::as_str)
                .unwrap_or("upstream request failed")
                .to_string();
            return Err(GoogleAdminError::UpstreamApi {
                status: status.as_u16(),
                message,
            });
        }
        Ok(parsed)
    }
}
