//! Google Custom Search JSON API HTTP client.

use crate::types::{
    GoogleApiErrorPayload, GoogleApiErrorResponse, GoogleSearchApiResponse, SearchArgs,
    SearchResultItem, SearchResultOutput,
};
use crate::url_encoder::QueryBuilder;

pub const GOOGLE_CUSTOM_SEARCH_BASE_URL: &str =
    "https://customsearch.googleapis.com/customsearch/v1";

/// Maximum number of search results per Google Custom Search query (Google API limit is 10).
pub const MAX_RESULTS_PER_PAGE: u32 = 10;
/// Default number of results if unspecified.
pub const DEFAULT_NUM_RESULTS: u32 = 5;
/// Maximum start index supported by Google Custom Search API.
pub const MAX_START_INDEX: u32 = 91;

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

/// Builds the full Google Custom Search API request URL with all query parameters.
pub fn build_search_url(api_key: &str, cx: &str, args: &SearchArgs) -> String {
    let mut qb = QueryBuilder::new();

    qb.add_param("key", api_key.trim());
    qb.add_param("cx", cx.trim());
    qb.add_param("q", args.query.trim());

    // Number of results: clamp between 1 and 10, defaulting to 5
    let num = args
        .num
        .unwrap_or(DEFAULT_NUM_RESULTS)
        .clamp(1, MAX_RESULTS_PER_PAGE);
    qb.add_param("num", &num.to_string());

    // Start index: clamp between 1 and 91, defaulting to 1
    let start = args.start.unwrap_or(1).clamp(1, MAX_START_INDEX);
    if start > 1 {
        qb.add_param("start", &start.to_string());
    }

    // SafeSearch setting: default "active", accept "off"
    let safe = match args.safe.as_deref().map(str::trim) {
        Some(s) if s.eq_ignore_ascii_case("off") => "off",
        _ => "active",
    };
    qb.add_param("safe", safe);

    // Optional filters and constraints
    qb.add_optional_param("siteSearch", args.site.as_deref());
    qb.add_optional_param("fileType", args.file_type.as_deref());
    qb.add_optional_param("dateRestrict", args.date_restrict.as_deref());
    qb.add_optional_param("exactTerms", args.exact_terms.as_deref());
    qb.add_optional_param("excludeTerms", args.exclude_terms.as_deref());
    qb.add_optional_param("sort", args.sort.as_deref());
    qb.add_optional_param("lr", args.language.as_deref());
    qb.add_optional_param("gl", args.country.as_deref());

    if let Some(ref st) = args.search_type {
        if st.trim().eq_ignore_ascii_case("image") {
            qb.add_param("searchType", "image");
        }
    }

    format!("{}{}", GOOGLE_CUSTOM_SEARCH_BASE_URL, qb.build_query_string())
}

/// Parses the HTTP response from the Google Custom Search API into a structured `SearchResultOutput`.
pub fn parse_search_response(
    status: u16,
    body: &[u8],
    query: &str,
) -> Result<SearchResultOutput, String> {
    if status == 200 {
        let api_resp: GoogleSearchApiResponse = serde_json::from_slice(body).map_err(|e| {
            let body_str = String::from_utf8_lossy(body);
            let preview = truncate_str(&body_str, 500);
            format!(
                "Failed to parse Google Search API response JSON: {}. Raw response: {}",
                e, preview
            )
        })?;

        // Check if a 200 response contains an error envelope
        if let Some(ref err) = api_resp.error {
            return Err(format_api_error(err, status));
        }

        let mut results = Vec::new();
        if let Some(items) = api_resp.items {
            for item in items {
                let title = item.title.unwrap_or_default();
                let link = item.link.unwrap_or_default();
                let snippet = item.snippet.unwrap_or_default();
                let display_link = item.display_link;

                // Skip items that have neither title nor link
                if !title.is_empty() || !link.is_empty() {
                    results.push(SearchResultItem {
                        title,
                        link,
                        snippet,
                        display_link,
                    });
                }
            }
        }

        let total_results = api_resp
            .search_information
            .as_ref()
            .and_then(|info| info.formatted_total_results.clone().or_else(|| info.total_results.clone()));

        let search_time = api_resp
            .search_information
            .as_ref()
            .and_then(|info| info.search_time);

        let spelling_suggestion = api_resp
            .spelling
            .as_ref()
            .and_then(|s| s.corrected_query.clone());

        let count = results.len();

        Ok(SearchResultOutput {
            query: query.to_string(),
            total_results,
            search_time,
            spelling_suggestion,
            count,
            results,
        })
    } else {
        // Attempt to parse Google's structured error payload
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
    fn test_truncate_str_ascii() {
        let s = "abcdefghij";
        assert_eq!(truncate_str(s, 5), "abcde");
        assert_eq!(truncate_str(s, 20), "abcdefghij");
        assert_eq!(truncate_str(s, 0), "");
    }

    #[test]
    fn test_truncate_str_unicode() {
        // '🦀' is 4 bytes (F0 9F A6 80)
        let s = "hello 🦀 world";
        // byte indices: 'hello ' = 6 bytes, '🦀' = 4 bytes (6..10), ' world' = 6 bytes
        // Truncating at 7 (mid-crab) should yield "hello " (length 6)
        assert_eq!(truncate_str(s, 7), "hello ");
        assert_eq!(truncate_str(s, 10), "hello 🦀");
        assert_eq!(truncate_str(s, 11), "hello 🦀 ");
    }

    #[test]
    fn test_build_search_url_defaults() {
        let args = SearchArgs {
            query: "rust components".to_string(),
            num: None,
            start: None,
            site: None,
            file_type: None,
            date_restrict: None,
            exact_terms: None,
            exclude_terms: None,
            sort: None,
            safe: None,
            language: None,
            country: None,
            search_type: None,
            api_key: None,
            cx: None,
        };

        let url = build_search_url("test_key", "test_cx", &args);
        assert!(url.starts_with(GOOGLE_CUSTOM_SEARCH_BASE_URL));
        assert!(url.contains("key=test_key"));
        assert!(url.contains("cx=test_cx"));
        assert!(url.contains("q=rust%20components"));
        assert!(url.contains("num=5"));
        assert!(url.contains("safe=active"));
        assert!(!url.contains("start="));
    }

    #[test]
    fn test_build_search_url_options() {
        let args = SearchArgs {
            query: "  wasm tutorial  ".to_string(),
            num: Some(10),
            start: Some(11),
            site: Some("rust-lang.org".to_string()),
            file_type: Some("pdf".to_string()),
            date_restrict: Some("m1".to_string()),
            exact_terms: Some("component model".to_string()),
            exclude_terms: Some("legacy".to_string()),
            sort: Some("date".to_string()),
            safe: Some("OFF".to_string()),
            language: Some("lang_en".to_string()),
            country: Some("us".to_string()),
            search_type: Some("IMAGE".to_string()),
            api_key: None,
            cx: None,
        };

        let url = build_search_url("my_key", "my_cx", &args);
        assert!(url.contains("num=10"));
        assert!(url.contains("start=11"));
        assert!(url.contains("q=wasm%20tutorial"));
        assert!(url.contains("siteSearch=rust-lang.org"));
        assert!(url.contains("fileType=pdf"));
        assert!(url.contains("dateRestrict=m1"));
        assert!(url.contains("exactTerms=component%20model"));
        assert!(url.contains("excludeTerms=legacy"));
        assert!(url.contains("sort=date"));
        assert!(url.contains("safe=off"));
        assert!(url.contains("lr=lang_en"));
        assert!(url.contains("gl=us"));
        assert!(url.contains("searchType=image"));
    }

    #[test]
    fn test_parse_search_response_success() {
        let json = r#"{
            "searchInformation": {
                "totalResults": "45200",
                "formattedTotalResults": "45,200",
                "searchTime": 0.28
            },
            "spelling": {
                "correctedQuery": "rust webassembly"
            },
            "items": [
                {
                    "title": "Rust and WebAssembly",
                    "link": "https://rustwasm.github.io/docs/book/",
                    "snippet": "An introduction to Rust and WebAssembly.",
                    "displayLink": "rustwasm.github.io"
                }
            ]
        }"#;

        let res = parse_search_response(200, json.as_bytes(), "rust wasm").unwrap();
        assert_eq!(res.query, "rust wasm");
        assert_eq!(res.count, 1);
        assert_eq!(res.total_results.as_deref(), Some("45,200"));
        assert_eq!(res.search_time, Some(0.28));
        assert_eq!(res.spelling_suggestion.as_deref(), Some("rust webassembly"));
        assert_eq!(res.results[0].title, "Rust and WebAssembly");
        assert_eq!(res.results[0].link, "https://rustwasm.github.io/docs/book/");
        assert_eq!(res.results[0].snippet, "An introduction to Rust and WebAssembly.");
        assert_eq!(res.results[0].display_link.as_deref(), Some("rustwasm.github.io"));
    }

    #[test]
    fn test_parse_search_response_empty() {
        let json = r#"{
            "searchInformation": {
                "totalResults": "0",
                "formattedTotalResults": "0",
                "searchTime": 0.12
            }
        }"#;

        let res = parse_search_response(200, json.as_bytes(), "nonexistent term").unwrap();
        assert_eq!(res.count, 0);
        assert!(res.results.is_empty());
    }

    #[test]
    fn test_parse_search_response_error_payload_200() {
        let json = r#"{
            "error": {
                "code": 400,
                "message": "Invalid Query",
                "status": "INVALID_ARGUMENT"
            }
        }"#;

        let res = parse_search_response(200, json.as_bytes(), "bad query");
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains("400"));
        assert!(err.contains("Invalid Query"));
        assert!(err.contains("INVALID_ARGUMENT"));
    }

    #[test]
    fn test_parse_search_response_error_payload_403() {
        let json = r#"{
            "error": {
                "code": 403,
                "message": "The request is missing a valid API key.",
                "status": "PERMISSION_DENIED",
                "errors": [
                    {
                        "message": "The request is missing a valid API key.",
                        "domain": "global",
                        "reason": "forbidden"
                    }
                ]
            }
        }"#;

        let res = parse_search_response(403, json.as_bytes(), "test");
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains("403"));
        assert!(err.contains("missing a valid API key"));
        assert!(err.contains("PERMISSION_DENIED"));
        assert!(err.contains("forbidden"));
    }

    #[test]
    fn test_parse_search_response_raw_html_error() {
        let raw_html = b"<html><head><title>502 Bad Gateway</title></head><body>Bad Gateway</body></html>";
        let res = parse_search_response(502, raw_html, "test");
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains("502"));
        assert!(err.contains("Bad Gateway"));
    }
}
