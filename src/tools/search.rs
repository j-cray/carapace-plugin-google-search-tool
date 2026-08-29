//! Google Vertex AI Search tool implementation.

use crate::client::{
    build_request_headers, build_search_endpoint, build_search_request, parse_search_response,
};
use crate::config::resolve_full_config_with;
use crate::types::SearchArgs;

pub const TOOL_NAME: &str = "google_search";
pub const TOOL_DESCRIPTION: &str =
    "Search web or enterprise data using Google Vertex AI Search / Google Enterprise Web Search (Discovery Engine API). Returns relevant search results with titles, snippets, URLs, summaries, and metadata.";

/// Returns the JSON Schema string describing the tool's expected input parameters.
pub fn tool_input_schema() -> String {
    serde_json::json!({
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "The search query string to execute with Vertex AI Search."
            },
            "num": {
                "type": "integer",
                "description": "Number of search results to return (1-100, default 5).",
                "minimum": 1,
                "maximum": 100
            },
            "page_size": {
                "type": "integer",
                "description": "Alias for num. Number of search results to return (1-100, default 5).",
                "minimum": 1,
                "maximum": 100
            },
            "start": {
                "type": "integer",
                "description": "1-based index of the first result to return, for pagination.",
                "minimum": 1
            },
            "offset": {
                "type": "integer",
                "description": "0-based offset of the results to return, for pagination.",
                "minimum": 0
            },
            "site": {
                "type": "string",
                "description": "Restrict search results to a specific domain (e.g. 'docs.rs' or 'github.com')."
            },
            "file_type": {
                "type": "string",
                "description": "Restrict results to a specific file extension (e.g., 'pdf', 'html', 'json')."
            },
            "exact_terms": {
                "type": "string",
                "description": "Identifies a phrase that all documents in the search results must contain."
            },
            "exclude_terms": {
                "type": "string",
                "description": "Identifies a word or phrase that should not appear in any documents in the search results."
            },
            "filter": {
                "type": "string",
                "description": "Filter expression applied to structured metadata attributes."
            },
            "order_by": {
                "type": "string",
                "description": "The ordering expression to apply to the results (e.g. 'publish_time desc')."
            },
            "sort": {
                "type": "string",
                "description": "Alias for order_by."
            },
            "project_id": {
                "type": "string",
                "description": "Optional Google Cloud Project ID override."
            },
            "data_store_id": {
                "type": "string",
                "description": "Optional Vertex AI Search Data Store ID override."
            },
            "engine_id": {
                "type": "string",
                "description": "Optional Vertex AI Search App / Engine ID override."
            },
            "location": {
                "type": "string",
                "description": "Google Cloud location / region override (default 'global')."
            },
            "collection_id": {
                "type": "string",
                "description": "Vertex AI Search collection ID override (default 'default_collection')."
            },
            "serving_config_id": {
                "type": "string",
                "description": "Vertex AI Search serving config ID override (default 'default_search')."
            },
            "api_key": {
                "type": "string",
                "description": "Optional Google Cloud API Key override."
            },
            "auth_token": {
                "type": "string",
                "description": "Optional OAuth2 Bearer / access token override."
            },
            "cx": {
                "type": "string",
                "description": "Legacy Search Engine ID override (mapped to data_store_id)."
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

    let req_body = build_search_request(&args);
    let body_bytes = serde_json::to_vec(&req_body)
        .map_err(|e| format!("Failed to serialize search request body: {}", e))?;

    let (status, body) = http_fetch(&url, &headers, &body_bytes)?;

    let output = parse_search_response(status, &body, &args.query)?;

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
        assert!(val["properties"]["project_id"].is_object());
        assert!(val["properties"]["data_store_id"].is_object());
        assert!(val["properties"]["engine_id"].is_object());
        assert!(val["properties"]["api_key"].is_object());
        assert!(val["properties"]["auth_token"].is_object());
        assert_eq!(val["required"][0], "query");
    }

    #[test]
    fn test_parse_and_validate_args_valid() {
        let json = r#"{"query": "  rust wasm  ", "num": 3, "project_id": "p1", "data_store_id": "ds1"}"#;
        let args = parse_and_validate_args(json).unwrap();
        assert_eq!(args.query, "rust wasm");
        assert_eq!(args.num, Some(3));
        assert_eq!(args.project_id.as_deref(), Some("p1"));
        assert_eq!(args.data_store_id.as_deref(), Some("ds1"));
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
            "totalSize": 100,
            "results": [
                {
                    "document": {
                        "derivedStructData": {
                            "title": "Rust Programming",
                            "link": "https://www.rust-lang.org",
                            "snippets": [
                                {
                                    "snippet": "Empowering everyone to build reliable software."
                                }
                            ]
                        }
                    }
                }
            ]
        }"#;

        let res = execute_search_with(
            r#"{"query": "rust", "project_id": "p1", "data_store_id": "ds1", "api_key": "k1"}"#,
            |_| None,
            |_| None,
            |_url, _headers, _body| Ok((200, mock_body.as_bytes().to_vec())),
        )
        .unwrap();

        assert!(res.contains("Rust Programming"));
        assert!(res.contains("https://www.rust-lang.org"));
        assert!(res.contains("Empowering everyone"));
    }

    #[test]
    fn test_execute_search_with_api_error_response() {
        let error_body = r#"{
            "error": {
                "code": 403,
                "message": "Quota exceeded",
                "status": "RESOURCE_EXHAUSTED"
            }
        }"#;

        let res = execute_search_with(
            r#"{"query": "rust", "project_id": "p1", "data_store_id": "ds1", "api_key": "k1"}"#,
            |_| None,
            |_| None,
            |_url, _headers, _body| Ok((403, error_body.as_bytes().to_vec())),
        );

        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains("Quota exceeded"));
        assert!(err.contains("RESOURCE_EXHAUSTED"));
    }
}
