//! Google Search tool implementation.

use crate::client::{build_search_url, parse_search_response};
use crate::config::{resolve_api_key_with, resolve_cx_with};
use crate::types::SearchArgs;

pub const TOOL_NAME: &str = "google_search";
pub const TOOL_DESCRIPTION: &str =
    "Search the web using Google Search (Custom Search JSON API). Returns relevant search results with titles, snippets, URLs, and metadata.";

/// Returns the JSON Schema string describing the tool's expected input parameters.
pub fn tool_input_schema() -> String {
    serde_json::json!({
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "The search query string to execute with Google Search."
            },
            "num": {
                "type": "integer",
                "description": "Number of search results to return (1-10, default 5).",
                "minimum": 1,
                "maximum": 10
            },
            "start": {
                "type": "integer",
                "description": "1-based index of the first result to return, for pagination (1-91, default 1).",
                "minimum": 1,
                "maximum": 91
            },
            "site": {
                "type": "string",
                "description": "Restrict search results to a specific domain (e.g., 'docs.rs' or 'github.com')."
            },
            "file_type": {
                "type": "string",
                "description": "Restrict results to a specific file extension (e.g., 'pdf', 'html', 'json')."
            },
            "date_restrict": {
                "type": "string",
                "description": "Restrict results by date range (e.g. 'd1' for past 24h, 'w1' for past week, 'm1' for past month, 'y1' for past year)."
            },
            "safe": {
                "type": "string",
                "description": "SafeSearch filtering level ('active' or 'off'). Defaults to 'active'.",
                "enum": ["active", "off"]
            },
            "language": {
                "type": "string",
                "description": "Language code filter (e.g. 'lang_en', 'lang_de', 'lang_fr')."
            },
            "country": {
                "type": "string",
                "description": "Geolocation country code for boosting local results (e.g. 'us', 'uk', 'de')."
            },
            "search_type": {
                "type": "string",
                "description": "Search mode ('web' or 'image'). Defaults to 'web'.",
                "enum": ["web", "image"]
            },
            "api_key": {
                "type": "string",
                "description": "Optional Google Cloud API Key override."
            },
            "cx": {
                "type": "string",
                "description": "Optional Google Custom Search Engine ID (cx) override."
            }
        },
        "required": ["query"],
        "additionalProperties": false
    })
    .to_string()
}

/// Parse and validate search arguments from JSON input.
pub fn parse_and_validate_args(params_json: &str) -> Result<SearchArgs, String> {
    let args: SearchArgs = serde_json::from_str(params_json)
        .map_err(|e| format!("Invalid arguments for google_search tool: {}", e))?;

    if args.query.trim().is_empty() {
        return Err("Argument 'query' must not be empty.".to_string());
    }

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
    FH: Fn(&str) -> Result<(u16, Vec<u8>), String>,
{
    let args = parse_and_validate_args(params_json)?;

    let api_key = resolve_api_key_with(args.api_key.as_deref(), &credential_get, &config_get)?;
    let cx = resolve_cx_with(args.cx.as_deref(), &credential_get, &config_get)?;

    let url = build_search_url(&api_key, &cx, &args);

    let (status, body) = http_fetch(&url)?;

    let output = parse_search_response(status, &body, &args.query)?;

    serde_json::to_string(&output)
        .map_err(|e| format!("Failed to serialize search output to JSON: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_validate_args_valid() {
        let json = r#"{"query": "rust wasm", "num": 3}"#;
        let args = parse_and_validate_args(json).unwrap();
        assert_eq!(args.query, "rust wasm");
        assert_eq!(args.num, Some(3));
    }

    #[test]
    fn test_parse_and_validate_args_empty_query() {
        let json = r#"{"query": "   "}"#;
        let err = parse_and_validate_args(json).unwrap_err();
        assert!(err.contains("must not be empty"));
    }

    #[test]
    fn test_execute_search_with_mock() {
        let mock_body = r#"{
            "searchInformation": {
                "totalResults": "100",
                "formattedTotalResults": "100",
                "searchTime": 0.1
            },
            "items": [
                {
                    "title": "Rust Programming",
                    "link": "https://www.rust-lang.org",
                    "snippet": "Empowering everyone to build reliable software."
                }
            ]
        }"#;

        let res = execute_search_with(
            r#"{"query": "rust", "api_key": "k1", "cx": "c1"}"#,
            |_| None,
            |_| None,
            |_url| Ok((200, mock_body.as_bytes().to_vec())),
        )
        .unwrap();

        assert!(res.contains("Rust Programming"));
        assert!(res.contains("https://www.rust-lang.org"));
    }
}
