use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::{Deserialize, Serialize};
use storeit_domain::entities::IdentificationResult;
use storeit_domain::errors::{DomainError, Result};
use storeit_domain::storage::ItemIdentifier;

use crate::parse::parse_identification_json;
use crate::prompt;

pub struct AnthropicApiIdentifier {
    api_key: String,
    model: String,
    client: reqwest::Client,
    base_url: String,
}

impl AnthropicApiIdentifier {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: reqwest::Client::new(),
            base_url: "https://api.anthropic.com".into(),
        }
    }

    #[cfg(test)]
    fn with_base_url(api_key: String, model: String, base_url: String) -> Self {
        Self {
            api_key,
            model,
            client: reqwest::Client::new(),
            base_url,
        }
    }

    async fn call_api(
        &self,
        image_data: &[u8],
        mime_type: &str,
        prompt_text: &str,
    ) -> Result<String> {
        let b64 = BASE64.encode(image_data);

        let body = ApiRequest {
            model: &self.model,
            max_tokens: 1024,
            messages: vec![ApiMessage {
                role: "user",
                content: vec![
                    ContentBlock::Image {
                        r#type: "image",
                        source: ImageSource {
                            r#type: "base64",
                            media_type: mime_type,
                            data: &b64,
                        },
                    },
                    ContentBlock::Text {
                        r#type: "text",
                        text: prompt_text,
                    },
                ],
            }],
        };

        let url = format!("{}/v1/messages", self.base_url);
        let resp = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .map_err(|e| DomainError::Internal(format!("Anthropic API request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(DomainError::Internal(format!(
                "Anthropic API returned {status}: {text}"
            )));
        }

        let api_resp: ApiResponse = resp.json().await.map_err(|e| {
            DomainError::Internal(format!("failed to parse Anthropic response: {e}"))
        })?;

        // Extract text from the first text content block
        api_resp
            .content
            .into_iter()
            .find_map(|block| {
                if block.r#type == "text" {
                    Some(block.text)
                } else {
                    None
                }
            })
            .ok_or_else(|| DomainError::Internal("no text block in Anthropic response".into()))
    }
}

#[async_trait]
impl ItemIdentifier for AnthropicApiIdentifier {
    async fn identify(&self, image_data: &[u8], mime_type: &str) -> Result<IdentificationResult> {
        let prompt_text = prompt::build_identify_prompt();
        let raw = self.call_api(image_data, mime_type, &prompt_text).await?;
        tracing::debug!(raw_response = %raw, "anthropic api identify response");
        parse_identification_json(&raw)
    }

    async fn identify_with_correction(
        &self,
        image_data: &[u8],
        mime_type: &str,
        correction: &str,
    ) -> Result<IdentificationResult> {
        let prompt_text = prompt::build_correction_prompt(correction);
        let raw = self.call_api(image_data, mime_type, &prompt_text).await?;
        tracing::debug!(raw_response = %raw, "anthropic api identify_with_correction response");
        parse_identification_json(&raw)
    }
}

// -- API types (private) --

#[derive(Serialize)]
struct ApiRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    messages: Vec<ApiMessage<'a>>,
}

#[derive(Serialize)]
struct ApiMessage<'a> {
    role: &'a str,
    content: Vec<ContentBlock<'a>>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum ContentBlock<'a> {
    Image {
        r#type: &'a str,
        source: ImageSource<'a>,
    },
    Text {
        r#type: &'a str,
        text: &'a str,
    },
}

#[derive(Serialize)]
struct ImageSource<'a> {
    r#type: &'a str,
    media_type: &'a str,
    data: &'a str,
}

#[derive(Deserialize)]
struct ApiResponse {
    content: Vec<ResponseContentBlock>,
}

#[derive(Deserialize)]
struct ResponseContentBlock {
    r#type: String,
    text: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    const VALID_ID_JSON: &str = r#"{"name":"Blue Mug","category":"kitchen","description":"A ceramic mug","aliases":["mug","cup"],"keywords":["kitchen","drink"],"color":"blue","material":"ceramic","condition_notes":"good"}"#;

    /// Spin up a one-shot HTTP mock that returns the given status and body.
    async fn mock_server(status: u16, body: &str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let base_url = format!("http://{addr}");

        let body = body.to_string();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();

            // Read request (we don't care about parsing it fully)
            let mut buf = vec![0u8; 8192];
            let mut total = 0;
            loop {
                let n = stream.read(&mut buf[total..]).await.unwrap();
                total += n;
                // Check if we've read the full HTTP headers + body
                // Look for end of headers, then check Content-Length
                let data = &buf[..total];
                if let Some(header_end) = find_header_end(data) {
                    let headers = std::str::from_utf8(&data[..header_end]).unwrap_or("");
                    let content_len = parse_content_length(headers);
                    let body_received = total - header_end;
                    if body_received >= content_len {
                        break;
                    }
                }
                if n == 0 {
                    break;
                }
            }

            let response = format!(
                "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).await.unwrap();
            stream.shutdown().await.unwrap();
        });

        base_url
    }

    fn find_header_end(data: &[u8]) -> Option<usize> {
        data.windows(4)
            .position(|w| w == b"\r\n\r\n")
            .map(|pos| pos + 4)
    }

    fn parse_content_length(headers: &str) -> usize {
        for line in headers.lines() {
            if let Some(val) = line.strip_prefix("content-length:") {
                return val.trim().parse().unwrap_or(0);
            }
            if let Some(val) = line.strip_prefix("Content-Length:") {
                return val.trim().parse().unwrap_or(0);
            }
        }
        0
    }

    fn api_success_body(identification_json: &str) -> String {
        format!(
            r#"{{"content":[{{"type":"text","text":"{}"}}]}}"#,
            identification_json.replace('"', r#"\""#)
        )
    }

    #[tokio::test]
    async fn identify_success() {
        let body = api_success_body(VALID_ID_JSON);
        let base_url = mock_server(200, &body).await;

        let identifier =
            AnthropicApiIdentifier::with_base_url("test-key".into(), "test-model".into(), base_url);
        let result = identifier
            .identify(b"fake-image", "image/jpeg")
            .await
            .unwrap();
        assert_eq!(result.name, "Blue Mug");
        assert_eq!(result.category.as_deref(), Some("kitchen"));
    }

    #[tokio::test]
    async fn identify_with_correction_success() {
        let body = api_success_body(VALID_ID_JSON);
        let base_url = mock_server(200, &body).await;

        let identifier =
            AnthropicApiIdentifier::with_base_url("test-key".into(), "test-model".into(), base_url);
        let result = identifier
            .identify_with_correction(b"fake-image", "image/jpeg", "it's a mug")
            .await
            .unwrap();
        assert_eq!(result.name, "Blue Mug");
    }

    #[tokio::test]
    async fn identify_api_error_returns_error() {
        let base_url = mock_server(401, r#"{"error":"unauthorized"}"#).await;

        let identifier =
            AnthropicApiIdentifier::with_base_url("bad-key".into(), "test-model".into(), base_url);
        let err = identifier
            .identify(b"fake-image", "image/jpeg")
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("Anthropic API returned"),
            "got: {}",
            err
        );
    }

    #[tokio::test]
    async fn identify_no_text_block_returns_error() {
        // Response with no "text" type content block
        let body = r#"{"content":[{"type":"image","text":"ignored"}]}"#;
        let base_url = mock_server(200, body).await;

        let identifier =
            AnthropicApiIdentifier::with_base_url("test-key".into(), "test-model".into(), base_url);
        let err = identifier
            .identify(b"fake-image", "image/jpeg")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("no text block"), "got: {}", err);
    }

    #[tokio::test]
    async fn identify_connection_refused_returns_error() {
        // Use a port that nothing is listening on
        let identifier = AnthropicApiIdentifier::with_base_url(
            "test-key".into(),
            "test-model".into(),
            "http://127.0.0.1:1".into(),
        );
        let err = identifier
            .identify(b"fake-image", "image/jpeg")
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("Anthropic API request failed"),
            "got: {}",
            err
        );
    }

    #[test]
    fn new_sets_production_base_url() {
        let identifier = AnthropicApiIdentifier::new("key".into(), "model".into());
        assert_eq!(identifier.base_url, "https://api.anthropic.com");
    }
}
