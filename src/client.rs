//! Google Vertex AI Search / Discovery Engine HTTP client.

use crate::config::{ResolvedConfig, SearchTarget};
use crate::types::{
    GoogleApiErrorPayload, GoogleApiErrorResponse, SearchArgs, SearchResultItem,
    SearchResultOutput, VertexContentSearchSpec, VertexExtractiveContentSpec,
    VertexSearchApiResponse, VertexSearchRequestBody, VertexSnippetSpec, VertexSpellCorrectionSpec,
    VertexSummarySpec,
};
use crate::url_encoder::url_encode;

pub const VERTEX_SEARCH_API_HOST: &str = "https://discoveryengine.googleapis.com";

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

/// Strips HTML tags and unescapes basic HTML entities in snippet text.
pub fn clean_snippet_text(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut in_tag = false;

    for c in input.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    let unescaped = result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ");

    unescaped.trim().to_string()
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

/// Formats a Google Cloud Discovery Engine error payload into a readable error message.
pub fn format_api_error(err: &GoogleApiErrorPayload, status: u16) -> String {
    let mut err_msg = format!(
        "Vertex AI Search API error (HTTP {}): {}",
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

/// Builds the full Vertex AI Search endpoint URL.
pub fn build_search_endpoint(config: &ResolvedConfig) -> String {
    let target_segment = match &config.target {
        SearchTarget::DataStore(ds) => format!("dataStores/{}", ds),
        SearchTarget::Engine(eng) => format!("engines/{}", eng),
    };

    let base = format!(
        "{}/v1/projects/{}/locations/{}/collections/{}/{}/servingConfigs/{}:search",
        VERTEX_SEARCH_API_HOST,
        config.project_id,
        config.location,
        config.collection_id,
        target_segment,
        config.serving_config_id,
    );

    if let Some(ref key) = config.auth.api_key {
        if config.auth.auth_token.is_none() {
            return format!("{}?key={}", base, url_encode(key));
        }
    }

    base
}

/// Builds the HTTP headers for the Vertex AI Search request.
pub fn build_request_headers(config: &ResolvedConfig) -> Vec<(String, String)> {
    let mut headers = vec![
        ("Content-Type".to_string(), "application/json".to_string()),
        ("Accept".to_string(), "application/json".to_string()),
        (
            "User-Agent".to_string(),
            "Carapace-Vertex-Search-Plugin/0.2.0".to_string(),
        ),
    ];

    if let Some(ref token) = config.auth.auth_token {
        headers.push((
            "Authorization".to_string(),
            format!("Bearer {}", token.trim()),
        ));
    }

    if let Some(ref key) = config.auth.api_key {
        headers.push(("x-goog-api-key".to_string(), key.trim().to_string()));
    }

    headers
}

/// Constructs the `VertexSearchRequestBody` from tool `SearchArgs`.
pub fn build_search_request(args: &SearchArgs) -> VertexSearchRequestBody {
    let mut query = args.query.trim().to_string();

    if let Some(ref exact) = args.exact_terms {
        let trimmed = exact.trim();
        if !trimmed.is_empty() {
            query.push_str(&format!(" \"{}\"", trimmed));
        }
    }

    if let Some(ref exclude) = args.exclude_terms {
        let trimmed = exclude.trim();
        if !trimmed.is_empty() {
            query.push_str(&format!(" -{}", trimmed));
        }
    }

    if let Some(ref site) = args.site {
        let trimmed = site.trim();
        if !trimmed.is_empty() {
            query.push_str(&format!(" site:{}", trimmed));
        }
    }

    if let Some(ref ft) = args.file_type {
        let trimmed = ft.trim();
        if !trimmed.is_empty() {
            query.push_str(&format!(" filetype:{}", trimmed));
        }
    }

    let page_size = args.num.map(|n| n.clamp(1, 100)).or(Some(5));

    let offset = match (args.offset, args.start) {
        (Some(off), _) => Some(off),
        (None, Some(st)) if st > 1 => Some(st.saturating_sub(1)),
        _ => None,
    };

    let content_search_spec = Some(VertexContentSearchSpec {
        snippet_spec: Some(VertexSnippetSpec {
            return_snippet: true,
        }),
        extractive_content_spec: Some(VertexExtractiveContentSpec {
            max_extractive_answer_count: Some(1),
            max_extractive_segment_count: Some(1),
        }),
        summary_spec: Some(VertexSummarySpec {
            summary_result_count: Some(5),
            include_citations: Some(true),
        }),
    });

    let spell_correction_spec = Some(VertexSpellCorrectionSpec {
        mode: Some("SUGGESTION_ONLY".to_string()),
    });

    VertexSearchRequestBody {
        query,
        page_size,
        offset,
        filter: args.filter.clone(),
        order_by: args.order_by.clone(),
        content_search_spec,
        spell_correction_spec,
    }
}

/// Parses the HTTP response from Vertex AI Search into structured `SearchResultOutput`.
pub fn parse_search_response(
    status: u16,
    body: &[u8],
    query: &str,
) -> Result<SearchResultOutput, String> {
    if status == 200 {
        let api_resp: VertexSearchApiResponse = serde_json::from_slice(body).map_err(|e| {
            let body_str = String::from_utf8_lossy(body);
            let preview = truncate_str(&body_str, 500);
            format!(
                "Failed to parse Vertex AI Search API response JSON: {}. Raw response: {}",
                e, preview
            )
        })?;

        if let Some(ref err) = api_resp.error {
            return Err(format_api_error(err, status));
        }

        let mut results = Vec::new();
        if let Some(items) = api_resp.results {
            for item in items {
                if let Some(doc) = item.document {
                    let derived = doc.derived_struct_data;
                    let struct_data = doc.struct_data;

                    let title = derived
                        .as_ref()
                        .and_then(|d| d.title.clone())
                        .or_else(|| {
                            struct_data.as_ref().and_then(|s| {
                                s.get("title").and_then(|v| v.as_str().map(|t| t.to_string()))
                            })
                        })
                        .unwrap_or_else(|| doc.id.clone().unwrap_or_default());

                    let link = derived
                        .as_ref()
                        .and_then(|d| {
                            d.link
                                .clone()
                                .or_else(|| d.url.clone())
                                .or_else(|| d.uri.clone())
                                .or_else(|| d.formatted_url.clone())
                        })
                        .or_else(|| {
                            struct_data.as_ref().and_then(|s| {
                                s.get("link")
                                    .or_else(|| s.get("url"))
                                    .or_else(|| s.get("uri"))
                                    .and_then(|v| v.as_str().map(|t| t.to_string()))
                            })
                        })
                        .unwrap_or_default();

                    let mut snippet = String::new();
                    if let Some(ref d) = derived {
                        if let Some(ref snippets) = d.snippets {
                            for s in snippets {
                                if let Some(ref snip) = s.snippet {
                                    let cleaned = clean_snippet_text(snip);
                                    if !cleaned.is_empty()
                                        && !cleaned.eq_ignore_ascii_case(
                                            "No snippet is available for this page.",
                                        )
                                    {
                                        snippet = cleaned;
                                        break;
                                    }
                                }
                            }
                        }

                        if snippet.is_empty() {
                            if let Some(ref answers) = d.extractive_answers {
                                if let Some(first_ans) = answers.first() {
                                    if let Some(ref c) = first_ans.content {
                                        snippet = clean_snippet_text(c);
                                    }
                                }
                            }
                        }

                        if snippet.is_empty() {
                            if let Some(ref segments) = d.extractive_segments {
                                if let Some(first_seg) = segments.first() {
                                    if let Some(ref c) = first_seg.content {
                                        snippet = clean_snippet_text(c);
                                    }
                                }
                            }
                        }
                    }

                    if snippet.is_empty() {
                        if let Some(ref s) = struct_data {
                            if let Some(desc) = s
                                .get("snippet")
                                .or_else(|| s.get("description"))
                                .and_then(|v| v.as_str())
                            {
                                snippet = clean_snippet_text(desc);
                            }
                        }
                    }

                    let display_link = derived
                        .as_ref()
                        .and_then(|d| d.display_link.clone())
                        .or_else(|| {
                            if !link.is_empty() {
                                extract_domain(&link)
                            } else {
                                None
                            }
                        });

                    if !title.is_empty() || !link.is_empty() || !snippet.is_empty() {
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

        let total_results = api_resp.total_size.and_then(|v| match v {
            serde_json::Value::Number(n) => Some(n.to_string()),
            serde_json::Value::String(s) => Some(s),
            _ => None,
        });

        let summary = api_resp
            .summary
            .and_then(|s| s.summary_text)
            .filter(|st| !st.trim().is_empty());

        let spelling_suggestion = api_resp.corrected_query.filter(|cq| !cq.trim().is_empty());

        let count = results.len();

        Ok(SearchResultOutput {
            query: query.to_string(),
            total_results,
            summary,
            spelling_suggestion,
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
            "Vertex AI Search API request failed with HTTP status {}: {}",
            status, preview
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ResolvedAuth;

    #[test]
    fn test_clean_snippet_text() {
        let input = "Learn about <b>WebAssembly</b> and &amp; <b>Rust</b> components.";
        let cleaned = clean_snippet_text(input);
        assert_eq!(cleaned, "Learn about WebAssembly and & Rust components.");
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            extract_domain("https://cloud.google.com/vertex-ai/docs"),
            Some("cloud.google.com".to_string())
        );
        assert_eq!(
            extract_domain("http://example.org:8080/path"),
            Some("example.org".to_string())
        );
    }

    #[test]
    fn test_build_search_endpoint_datastore_api_key() {
        let config = ResolvedConfig {
            project_id: "my-project".to_string(),
            target: SearchTarget::DataStore("my-store".to_string()),
            location: "global".to_string(),
            collection_id: "default_collection".to_string(),
            serving_config_id: "default_search".to_string(),
            auth: ResolvedAuth {
                auth_token: None,
                api_key: Some("AIzaSyTestKey".to_string()),
            },
        };

        let url = build_search_endpoint(&config);
        assert_eq!(
            url,
            "https://discoveryengine.googleapis.com/v1/projects/my-project/locations/global/collections/default_collection/dataStores/my-store/servingConfigs/default_search:search?key=AIzaSyTestKey"
        );
    }

    #[test]
    fn test_build_search_endpoint_engine_bearer_token() {
        let config = ResolvedConfig {
            project_id: "my-project".to_string(),
            target: SearchTarget::Engine("my-engine".to_string()),
            location: "us".to_string(),
            collection_id: "default_collection".to_string(),
            serving_config_id: "default_serving_config".to_string(),
            auth: ResolvedAuth {
                auth_token: Some("oauth-token-xyz".to_string()),
                api_key: None,
            },
        };

        let url = build_search_endpoint(&config);
        assert_eq!(
            url,
            "https://discoveryengine.googleapis.com/v1/projects/my-project/locations/us/collections/default_collection/engines/my-engine/servingConfigs/default_serving_config:search"
        );
    }

    #[test]
    fn test_build_request_headers() {
        let config = ResolvedConfig {
            project_id: "my-project".to_string(),
            target: SearchTarget::DataStore("my-store".to_string()),
            location: "global".to_string(),
            collection_id: "default_collection".to_string(),
            serving_config_id: "default_search".to_string(),
            auth: ResolvedAuth {
                auth_token: Some("token123".to_string()),
                api_key: Some("key123".to_string()),
            },
        };

        let headers = build_request_headers(&config);
        assert!(headers.contains(&(
            "Authorization".to_string(),
            "Bearer token123".to_string()
        )));
        assert!(headers.contains(&("x-goog-api-key".to_string(), "key123".to_string())));
        assert!(headers.contains(&(
            "Content-Type".to_string(),
            "application/json".to_string()
        )));
    }

    #[test]
    fn test_build_search_request_options() {
        let args = SearchArgs {
            query: "rust webassembly".to_string(),
            num: Some(10),
            start: Some(11),
            exact_terms: Some("component model".to_string()),
            exclude_terms: Some("legacy".to_string()),
            site: Some("rustwasm.github.io".to_string()),
            file_type: Some("html".to_string()),
            filter: Some("category: ANY(\"docs\")".to_string()),
            order_by: Some("publish_time desc".to_string()),
            ..Default::default()
        };

        let req = build_search_request(&args);
        assert_eq!(
            req.query,
            "rust webassembly \"component model\" -legacy site:rustwasm.github.io filetype:html"
        );
        assert_eq!(req.page_size, Some(10));
        assert_eq!(req.offset, Some(10)); // start 11 -> offset 10
        assert_eq!(req.filter, Some("category: ANY(\"docs\")".to_string()));
        assert_eq!(req.order_by, Some("publish_time desc".to_string()));
        assert!(req.content_search_spec.is_some());
    }

    #[test]
    fn test_parse_search_response_success() {
        let json = r#"{
            "totalSize": 42,
            "correctedQuery": "rust webassembly",
            "summary": {
                "summaryText": "WebAssembly allows Rust code to run in browser and server sandboxes."
            },
            "results": [
                {
                    "id": "doc1",
                    "document": {
                        "name": "projects/123/locations/global/collections/default_collection/dataStores/ds/branches/0/documents/doc1",
                        "id": "doc1",
                        "derivedStructData": {
                            "title": "Rust and WebAssembly",
                            "link": "https://rustwasm.github.io/docs/book/",
                            "displayLink": "rustwasm.github.io",
                            "snippets": [
                                {
                                    "snippet": "Learn <b>Rust</b> and <b>WebAssembly</b>.",
                                    "snippet_status": "SUCCESS"
                                }
                            ]
                        }
                    }
                }
            ]
        }"#;

        let res = parse_search_response(200, json.as_bytes(), "rust wasm").unwrap();
        assert_eq!(res.query, "rust wasm");
        assert_eq!(res.count, 1);
        assert_eq!(res.total_results.as_deref(), Some("42"));
        assert_eq!(res.spelling_suggestion.as_deref(), Some("rust webassembly"));
        assert_eq!(
            res.summary.as_deref(),
            Some("WebAssembly allows Rust code to run in browser and server sandboxes.")
        );
        assert_eq!(res.results[0].title, "Rust and WebAssembly");
        assert_eq!(
            res.results[0].link,
            "https://rustwasm.github.io/docs/book/"
        );
        assert_eq!(
            res.results[0].snippet,
            "Learn Rust and WebAssembly."
        );
        assert_eq!(
            res.results[0].display_link.as_deref(),
            Some("rustwasm.github.io")
        );
    }

    #[test]
    fn test_parse_search_response_extractive_fallback() {
        let json = r#"{
            "results": [
                {
                    "id": "doc2",
                    "document": {
                        "id": "doc2",
                        "derivedStructData": {
                            "title": "Documentation",
                            "link": "https://example.com/docs",
                            "extractive_answers": [
                                {
                                    "content": "This is an extractive answer from the document."
                                }
                            ]
                        }
                    }
                }
            ]
        }"#;

        let res = parse_search_response(200, json.as_bytes(), "docs").unwrap();
        assert_eq!(res.count, 1);
        assert_eq!(
            res.results[0].snippet,
            "This is an extractive answer from the document."
        );
    }

    #[test]
    fn test_parse_search_response_empty() {
        let json = r#"{ "totalSize": 0, "results": [] }"#;
        let res = parse_search_response(200, json.as_bytes(), "query").unwrap();
        assert_eq!(res.count, 0);
        assert!(res.results.is_empty());
    }

    #[test]
    fn test_parse_search_response_error_payload_403() {
        let json = r#"{
            "error": {
                "code": 403,
                "message": "Permission 'discoveryengine.servingConfigs.search' denied.",
                "status": "PERMISSION_DENIED"
            }
        }"#;

        let res = parse_search_response(403, json.as_bytes(), "test");
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains("403"));
        assert!(err.contains("PERMISSION_DENIED"));
        assert!(err.contains("discoveryengine.servingConfigs.search"));
    }
}
