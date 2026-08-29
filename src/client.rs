//! Google Search HTTP client powered by Google Search Grounding.

use crate::config::{BackendMode, ResolvedConfig};
use crate::types::{
    GeminiAiStudioRequestBody, GeminiAiStudioTool, GeminiContent, GeminiGenerateContentResponse,
    GeminiPart, GoogleApiErrorPayload, GoogleApiErrorResponse, SearchArgs, SearchResultItem,
    SearchResultOutput, VertexAiRequestBody, VertexAiTool,
};
use crate::url_encoder::url_encode;

pub const GOOGLE_AI_STUDIO_API_HOST: &str = "https://generativelanguage.googleapis.com";

/// Safely truncate a UTF-8 string to at most `max_bytes` bytes at a valid character boundary.
pub fn truncate_str(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        s
    } else {
        let mut boundary = max_bytes;
        while boundary > 0 && !s.is_char_boundary(boundary) {
            boundary -= 1;
        }
        &s[..boundary]
    }
}

/// Extract domain from link URL.
pub fn extract_domain(url_str: &str) -> Option<String> {
    let after_scheme = if let Some(pos) = url_str.find("://") {
        &url_str[pos + 3..]
    } else {
        url_str
    };

    let host_end = after_scheme.find('/').unwrap_or(after_scheme.len());
    let host = &after_scheme[..host_end];
    let host_clean = host.split(':').next().unwrap_or(host);

    if !host_clean.is_empty() {
        Some(host_clean.to_string())
    } else {
        None
    }
}

/// Formats a Google API error payload into a readable error message.
pub fn format_api_error(err: &GoogleApiErrorPayload, status: u16) -> String {
    let mut err_msg = format!(
        "Google Search API error (HTTP {}): {}",
        err.code.unwrap_or(status),
        err.message.as_deref().unwrap_or("Unknown error")
    );
    if let Some(ref st) = err.status {
        err_msg.push_str(&format!(" [{}]", st));
    }
    if let Some(ref errors) = err.errors {
        for e in errors {
            if let Some(ref reason) = e.reason {
                err_msg.push_str(&format!(" (reason: {})", reason));
            }
        }
    }
    err_msg
}

/// Builds the search endpoint URL for the resolved backend.
pub fn build_search_endpoint(config: &ResolvedConfig) -> String {
    match &config.backend {
        BackendMode::GoogleAiStudio { api_key } => {
            format!(
                "{}/v1beta/models/{}:generateContent?key={}",
                GOOGLE_AI_STUDIO_API_HOST,
                config.model,
                url_encode(api_key)
            )
        }
        BackendMode::VertexAi {
            project_id,
            location,
            api_key,
            auth_token,
        } => {
            let host = if location == "global" {
                "https://aiplatform.googleapis.com".to_string()
            } else {
                format!("https://{}-aiplatform.googleapis.com", location)
            };

            let base = format!(
                "{}/v1/projects/{}/locations/{}/publishers/google/models/{}:generateContent",
                host, project_id, location, config.model
            );

            if let Some(ref key) = api_key {
                if auth_token.is_none() {
                    return format!("{}?key={}", base, url_encode(key));
                }
            }

            base
        }
    }
}

/// Builds the HTTP headers for the request.
pub fn build_request_headers(config: &ResolvedConfig) -> Vec<(String, String)> {
    let mut headers = vec![
        ("Content-Type".to_string(), "application/json".to_string()),
        ("Accept".to_string(), "application/json".to_string()),
        (
            "User-Agent".to_string(),
            "Carapace-Google-Search-Plugin/0.2.0".to_string(),
        ),
    ];

    match &config.backend {
        BackendMode::GoogleAiStudio { api_key } => {
            headers.push(("x-goog-api-key".to_string(), api_key.trim().to_string()));
        }
        BackendMode::VertexAi {
            auth_token,
            api_key,
            ..
        } => {
            if let Some(ref token) = auth_token {
                headers.push((
                    "Authorization".to_string(),
                    format!("Bearer {}", token.trim()),
                ));
            }
            if let Some(ref key) = api_key {
                headers.push(("x-goog-api-key".to_string(), key.trim().to_string()));
            }
        }
    }

    headers
}

/// Formats the user search prompt including any query restrictions.
pub fn format_search_prompt(args: &SearchArgs) -> String {
    let mut prompt = format!(
        "Search the entire public web and provide comprehensive, factual, and up-to-date results for: {}",
        args.query.trim()
    );

    if let Some(ref site) = args.site {
        let trimmed = site.trim();
        if !trimmed.is_empty() {
            prompt.push_str(&format!(" (site: {})", trimmed));
        }
    }

    if let Some(ref exact) = args.exact_terms {
        let trimmed = exact.trim();
        if !trimmed.is_empty() {
            prompt.push_str(&format!(" (must include exact phrase: \"{}\")", trimmed));
        }
    }

    if let Some(ref exclude) = args.exclude_terms {
        let trimmed = exclude.trim();
        if !trimmed.is_empty() {
            prompt.push_str(&format!(" (exclude: {})", trimmed));
        }
    }

    prompt
}

/// Serializes the JSON request body for the search grounding request.
pub fn build_request_body_json(config: &ResolvedConfig, args: &SearchArgs) -> Result<Vec<u8>, String> {
    let prompt_text = format_search_prompt(args);

    match &config.backend {
        BackendMode::GoogleAiStudio { .. } => {
            let body = GeminiAiStudioRequestBody {
                contents: vec![GeminiContent {
                    role: None,
                    parts: vec![GeminiPart {
                        text: Some(prompt_text),
                    }],
                }],
                tools: vec![GeminiAiStudioTool {
                    google_search: serde_json::json!({}),
                }],
            };
            serde_json::to_vec(&body)
                .map_err(|e| format!("Failed to serialize Google AI Studio request body: {}", e))
        }
        BackendMode::VertexAi { .. } => {
            let body = VertexAiRequestBody {
                contents: vec![GeminiContent {
                    role: Some("user".to_string()),
                    parts: vec![GeminiPart {
                        text: Some(prompt_text),
                    }],
                }],
                tools: vec![VertexAiTool {
                    google_search: serde_json::json!({}),
                }],
            };
            serde_json::to_vec(&body)
                .map_err(|e| format!("Failed to serialize Vertex AI request body: {}", e))
        }
    }
}

/// Parses the HTTP response from Gemini with Google Search Grounding into structured `SearchResultOutput`.
pub fn parse_search_response(
    status: u16,
    body: &[u8],
    query: &str,
    max_results: Option<u32>,
) -> Result<SearchResultOutput, String> {
    if status == 200 {
        let api_resp: GeminiGenerateContentResponse =
            serde_json::from_slice(body).map_err(|e| {
                let body_str = String::from_utf8_lossy(body);
                let preview = truncate_str(&body_str, 500);
                format!(
                    "Failed to parse Google Search API response JSON: {}. Raw response: {}",
                    e, preview
                )
            })?;

        if let Some(ref err) = api_resp.error {
            return Err(format_api_error(err, status));
        }

        let candidate = api_resp
            .candidates
            .as_ref()
            .and_then(|cands| cands.first());

        let summary = candidate
            .and_then(|c| c.content.as_ref())
            .and_then(|cnt| {
                let parts: Vec<&str> = cnt
                    .parts
                    .iter()
                    .filter_map(|p| p.text.as_deref())
                    .collect();
                if parts.is_empty() {
                    None
                } else {
                    Some(parts.join("\n\n"))
                }
            });

        let grounding_metadata = candidate.and_then(|c| c.grounding_metadata.as_ref());

        let search_queries = grounding_metadata
            .and_then(|gm| gm.web_search_queries.clone())
            .filter(|q| !q.is_empty());

        let limit = max_results.unwrap_or(5).clamp(1, 20) as usize;
        let mut results = Vec::new();

        if let Some(gm) = grounding_metadata {
            if let Some(ref chunks) = gm.grounding_chunks {
                for (idx, chunk) in chunks.iter().enumerate() {
                    if results.len() >= limit {
                        break;
                    }

                    if let Some(ref web) = chunk.web {
                        let link = web.uri.clone().unwrap_or_default();
                        let display_link = extract_domain(&link);
                        let title = web
                            .title
                            .clone()
                            .filter(|t| !t.trim().is_empty())
                            .or_else(|| display_link.clone())
                            .unwrap_or_else(|| format!("Source {}", idx + 1));

                        // Find snippet from grounding_supports matching this chunk index
                        let mut snippet = String::new();
                        if let Some(ref supports) = gm.grounding_supports {
                            for sup in supports {
                                if let Some(ref indices) = sup.grounding_chunk_indices {
                                    if indices.contains(&idx) {
                                        if let Some(ref seg) = sup.segment {
                                            if let Some(ref txt) = seg.text {
                                                let trimmed = txt.trim();
                                                if !trimmed.is_empty() {
                                                    snippet = trimmed.to_string();
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if snippet.is_empty() {
                            if let Some(ref sum) = summary {
                                snippet = truncate_str(sum, 200).to_string();
                            }
                        }

                        if !link.is_empty() || !title.is_empty() {
                            results.push(SearchResultItem {
                                title,
                                link,
                                snippet,
                                display_link,
                            });
                        }
                    }
                }
            }
        }

        let count = results.len();

        Ok(SearchResultOutput {
            query: query.to_string(),
            summary,
            search_queries,
            count,
            results,
        })
    } else {
        if let Ok(err_resp) = serde_json::from_slice::<GoogleApiErrorResponse>(body) {
            if let Some(ref err) = err_resp.error {
                return Err(format_api_error(err, status));
            }
        }

        let body_str = String::from_utf8_lossy(body);
        let preview = truncate_str(&body_str, 500);

        Err(format!(
            "Google Search API request failed with HTTP status {}: {}",
            status, preview
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            extract_domain("https://news.ycombinator.com/item?id=123"),
            Some("news.ycombinator.com".to_string())
        );
        assert_eq!(
            extract_domain("http://example.org:8080/docs"),
            Some("example.org".to_string())
        );
    }

    #[test]
    fn test_build_search_endpoint_google_ai_studio() {
        let config = ResolvedConfig {
            model: "gemini-2.5-flash".to_string(),
            backend: BackendMode::GoogleAiStudio {
                api_key: "AIzaSyTestKey".to_string(),
            },
        };

        let url = build_search_endpoint(&config);
        assert_eq!(
            url,
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key=AIzaSyTestKey"
        );
    }

    #[test]
    fn test_build_search_endpoint_vertex_ai() {
        let config = ResolvedConfig {
            model: "gemini-2.5-flash".to_string(),
            backend: BackendMode::VertexAi {
                project_id: "my-gcp-project".to_string(),
                location: "us-central1".to_string(),
                auth_token: Some("token123".to_string()),
                api_key: None,
            },
        };

        let url = build_search_endpoint(&config);
        assert_eq!(
            url,
            "https://us-central1-aiplatform.googleapis.com/v1/projects/my-gcp-project/locations/us-central1/publishers/google/models/gemini-2.5-flash:generateContent"
        );
    }

    #[test]
    fn test_build_request_headers_ai_studio() {
        let config = ResolvedConfig {
            model: "gemini-2.5-flash".to_string(),
            backend: BackendMode::GoogleAiStudio {
                api_key: "key123".to_string(),
            },
        };

        let headers = build_request_headers(&config);
        assert!(headers.contains(&("x-goog-api-key".to_string(), "key123".to_string())));
        assert!(headers.contains(&(
            "Content-Type".to_string(),
            "application/json".to_string()
        )));
    }

    #[test]
    fn test_parse_search_response_success() {
        let json = r#"{
            "candidates": [
                {
                    "content": {
                        "parts": [
                            { "text": "Rust WebAssembly enables fast, sandboxed components." }
                        ],
                        "role": "model"
                    },
                    "groundingMetadata": {
                        "webSearchQueries": [
                            "rust webassembly component model 2026"
                        ],
                        "groundingChunks": [
                            {
                                "web": {
                                    "uri": "https://rustwasm.github.io/book/",
                                    "title": "Rust and WebAssembly Book"
                                }
                            }
                        ],
                        "groundingSupports": [
                            {
                                "groundingChunkIndices": [0],
                                "confidenceScores": [0.95],
                                "segment": {
                                    "text": "Rust WebAssembly provides reliable sandboxing."
                                }
                            }
                        ]
                    }
                }
            ]
        }"#;

        let res = parse_search_response(200, json.as_bytes(), "rust wasm", Some(5)).unwrap();
        assert_eq!(res.query, "rust wasm");
        assert_eq!(res.count, 1);
        assert_eq!(
            res.summary.as_deref(),
            Some("Rust WebAssembly enables fast, sandboxed components.")
        );
        assert_eq!(
            res.search_queries.as_deref(),
            Some(&vec!["rust webassembly component model 2026".to_string()][..])
        );
        assert_eq!(res.results[0].title, "Rust and WebAssembly Book");
        assert_eq!(res.results[0].link, "https://rustwasm.github.io/book/");
        assert_eq!(
            res.results[0].snippet,
            "Rust WebAssembly provides reliable sandboxing."
        );
        assert_eq!(
            res.results[0].display_link.as_deref(),
            Some("rustwasm.github.io")
        );
    }

    #[test]
    fn test_parse_search_response_error_payload() {
        let json = r#"{
            "error": {
                "code": 403,
                "message": "API key not valid",
                "status": "INVALID_ARGUMENT"
            }
        }"#;

        let res = parse_search_response(403, json.as_bytes(), "test", Some(5));
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains("403"));
        assert!(err.contains("API key not valid"));
    }
}
