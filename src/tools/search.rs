//! Google Search tool implementation with Google Search Grounding across the whole web.

use crate::client::{
    build_request_body_json, build_request_headers, build_search_endpoint, parse_search_response,
};
use crate::config::resolve_full_config_with;
use crate::types::SearchArgs;

pub const TOOL_NAME: &str = "google_search";
pub const TOOL_DESCRIPTION: &str =
    "Search the entire public web using Google Search. Returns real-time search results, grounded summaries, search queries, and source citations.";

/// Returns the JSON Schema string describing the tool's expected input parameters.
pub fn tool_input_schema() -> String {
    serde_json::json!({
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "The search query string to search across the entire public web."
            },
            "num": {
                "type": "integer",
                "description": "Maximum number of web search result citations to return (1-20, default 5).",
                "minimum": 1,
                "maximum": 20
            },
            "page_size": {
                "type": "integer",
                "description": "Alias for num. Maximum number of web search result citations to return (1-20, default 5).",
                "minimum": 1,
                "maximum": 20
            },
            "model": {
                "type": "string",
                "description": "Gemini model to use for grounded web search (default 'gemini-2.5-flash')."
            },
            "site": {
                "type": "string",
                "description": "Restrict search to a specific domain (e.g. 'docs.rs' or 'github.com')."
            },
            "exact_terms": {
                "type": "string",
                "description": "Identifies a phrase that all search results must contain."
            },
            "exclude_terms": {
                "type": "string",
                "description": "Identifies a word or phrase that should not appear in search results."
            },
            "api_key": {
                "type": "string",
                "description": "Optional Gemini / Google Cloud API Key override."
            },
            "auth_token": {
                "type": "string",
                "description": "Optional OAuth2 Bearer / access token override for Vertex AI."
            },
            "project_id": {
                "type": "string",
                "description": "Optional Google Cloud Project ID override for Vertex AI."
            },
            "location": {
                "type": "string",
                "description": "Google Cloud location / region for Vertex AI (default 'us-central1')."
            }
        },
        "required": ["query"],
        "additionalProperties": false
    })
    .to_string()
}

/// Parse and validate search arguments from JSON input.
pub fn parse_and_validate_args(params_json: &str) -> Result<SearchArgs, String> {
    let mut args: SearchArgs = serde_json::from_str(params_json)
        .map_err(|e| format!("Invalid arguments for google_search tool: {}", e))?;

    let trimmed_query = args.query.trim();
    if trimmed_query.is_empty() {
        return Err("Argument 'query' must not be empty.".to_string());
    }
    args.query = trimmed_query.to_string();

    Ok(args)
}

/// Execute a search with abstract credential getters and HTTP client function.
pub fn execute_search_with<FC, FK, FH>(
    params_json: &str,
    credential_get: FC,
    config_get: FK,
    http_fetch: FH,
) -> Result<String, String>
where
    FC: Fn(&str) -> Option<String>,
    FK: Fn(&str) -> Option<String>,
    FH: Fn(&str, &[(String, String)], &[u8]) -> Result<(u16, Vec<u8>), String>,
{
    let args = parse_and_validate_args(params_json)?;

    let config = resolve_full_config_with(&args, &credential_get, &config_get)?;
    let url = build_search_endpoint(&config);
    let headers = build_request_headers(&config);

    let body_bytes = build_request_body_json(&config, &args)?;

    let (status, body) = http_fetch(&url, &headers, &body_bytes)?;

    let output = parse_search_response(status, &body, &args.query, args.num)?;

    serde_json::to_string(&output)
        .map_err(|e| format!("Failed to serialize search output to JSON: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_input_schema_is_valid_json() {
        let schema_str = tool_input_schema();
        let val: serde_json::Value = serde_json::from_str(&schema_str).unwrap();
        assert_eq!(val["type"], "object");
        assert!(val["properties"]["query"].is_object());
        assert!(val["properties"]["num"].is_object());
        assert!(val["properties"]["model"].is_object());
        assert!(val["properties"]["api_key"].is_object());
        assert_eq!(val["required"][0], "query");
    }

    #[test]
    fn test_parse_and_validate_args_valid() {
        let json = r#"{"query": "  rust wasm  ", "num": 3, "api_key": "k1"}"#;
        let args = parse_and_validate_args(json).unwrap();
        assert_eq!(args.query, "rust wasm");
        assert_eq!(args.num, Some(3));
        assert_eq!(args.api_key.as_deref(), Some("k1"));
    }

    #[test]
    fn test_parse_and_validate_args_empty_query() {
        let json = r#"{"query": "   "}"#;
        let err = parse_and_validate_args(json).unwrap_err();
        assert!(err.contains("must not be empty"));
    }

    #[test]
    fn test_parse_and_validate_args_missing_query() {
        let json = r#"{"num": 5}"#;
        let err = parse_and_validate_args(json).unwrap_err();
        assert!(err.contains("missing field `query`"));
    }

    #[test]
    fn test_execute_search_with_mock() {
        let mock_body = r#"{
            "candidates": [
                {
                    "content": {
                        "parts": [
                            { "text": "Rust is a fast, memory-efficient programming language." }
                        ]
                    },
                    "groundingMetadata": {
                        "webSearchQueries": ["rust programming language overview"],
                        "groundingChunks": [
                            {
                                "web": {
                                    "uri": "https://www.rust-lang.org",
                                    "title": "Rust Programming Language"
                                }
                            }
                        ],
                        "groundingSupports": [
                            {
                                "groundingChunkIndices": [0],
                                "segment": {
                                    "text": "A language empowering everyone to build reliable and efficient software."
                                }
                            }
                        ]
                    }
                }
            ]
        }"#;

        let res = execute_search_with(
            r#"{"query": "rust", "api_key": "k1"}"#,
            |_| None,
            |_| None,
            |_url, _headers, _body| Ok((200, mock_body.as_bytes().to_vec())),
        )
        .unwrap();

        assert!(res.contains("Rust Programming Language"));
        assert!(res.contains("https://www.rust-lang.org"));
        assert!(res.contains("Rust is a fast, memory-efficient programming language."));
    }

    #[test]
    fn test_execute_search_with_api_error_response() {
        let error_body = r#"{
            "error": {
                "code": 400,
                "message": "Invalid API key",
                "status": "INVALID_ARGUMENT"
            }
        }"#;

        let res = execute_search_with(
            r#"{"query": "rust", "api_key": "invalid_key"}"#,
            |_| None,
            |_| None,
            |_url, _headers, _body| Ok((400, error_body.as_bytes().to_vec())),
        );

        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains("Invalid API key"));
        assert!(err.contains("400"));
    }
}
