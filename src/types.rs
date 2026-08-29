//! Types and data structures for Google Search tool and API responses.

use serde::{Deserialize, Serialize};

/// Tool input arguments for the `google_search` tool.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SearchArgs {
    /// Search query string (required)
    pub query: String,

    /// Number of search results to return (1-10, default 5)
    #[serde(default)]
    pub num: Option<u32>,

    /// Index of first result to return (1-91, default 1, for pagination)
    #[serde(default)]
    pub start: Option<u32>,

    /// Restrict search to a specific site or domain (e.g. "docs.rs", "github.com")
    #[serde(default)]
    pub site: Option<String>,

    /// Restrict results to files with specified extension (e.g. "pdf", "html")
    #[serde(default)]
    pub file_type: Option<String>,

    /// Restrict results by date (e.g. "d1", "w1", "m1", "y1")
    #[serde(default)]
    pub date_restrict: Option<String>,

    /// Phrase that all documents in the search results must contain
    #[serde(default)]
    pub exact_terms: Option<String>,

    /// Word or phrase that should not appear in any documents in search results
    #[serde(default)]
    pub exclude_terms: Option<String>,

    /// Sort expression to apply to the results (e.g. "date")
    #[serde(default)]
    pub sort: Option<String>,

    /// SafeSearch filter level ("active" or "off", default "active")
    #[serde(default)]
    pub safe: Option<String>,

    /// Language restriction code (e.g. "lang_en")
    #[serde(default)]
    pub language: Option<String>,

    /// Geolocation country code (e.g. "us", "uk")
    #[serde(default)]
    pub country: Option<String>,

    /// Search type ("web" or "image", default "web")
    #[serde(default)]
    pub search_type: Option<String>,

    /// Optional per-call Google API Key override
    #[serde(default)]
    pub api_key: Option<String>,

    /// Optional per-call Custom Search Engine ID override
    #[serde(default)]
    pub cx: Option<String>,
}

/// Raw Google Custom Search API JSON response root.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct GoogleSearchApiResponse {
    /// List of search result items
    #[serde(default)]
    pub items: Option<Vec<GoogleSearchItem>>,

    /// Search metadata and performance stats
    #[serde(default, rename = "searchInformation")]
    pub search_information: Option<GoogleSearchInformation>,

    /// Suggested spelling correction
    #[serde(default)]
    pub spelling: Option<GoogleSearchSpelling>,

    /// Error payload (if returned in 200/error envelope)
    #[serde(default)]
    pub error: Option<GoogleApiErrorPayload>,
}

/// A single search result item returned by Google Custom Search API.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct GoogleSearchItem {
    /// Title of the search result
    #[serde(default)]
    pub title: Option<String>,

    /// Full URL link
    #[serde(default)]
    pub link: Option<String>,

    /// Descriptive snippet
    #[serde(default)]
    pub snippet: Option<String>,

    /// Display domain (e.g. "www.rust-lang.org")
    #[serde(default, rename = "displayLink")]
    pub display_link: Option<String>,

    /// Formatted display URL
    #[serde(default, rename = "formattedUrl")]
    pub formatted_url: Option<String>,

    /// Embedded page metadata
    #[serde(default)]
    pub pagemap: Option<serde_json::Value>,
}

/// Search information metadata returned by Google Custom Search API.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct GoogleSearchInformation {
    /// Total search results found (string format from Google)
    #[serde(default, rename = "totalResults")]
    pub total_results: Option<String>,

    /// Formatted total results string (e.g. "1,230,000")
    #[serde(default, rename = "formattedTotalResults")]
    pub formatted_total_results: Option<String>,

    /// Search execution time in seconds
    #[serde(default, rename = "searchTime")]
    pub search_time: Option<f64>,

    /// Formatted search time string (e.g. "0.32")
    #[serde(default, rename = "formattedSearchTime")]
    pub formatted_search_time: Option<String>,
}

/// Suggested spelling correction from Google.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct GoogleSearchSpelling {
    /// Corrected query suggestion
    #[serde(default, rename = "correctedQuery")]
    pub corrected_query: Option<String>,

    /// HTML-formatted corrected query suggestion
    #[serde(default, rename = "htmlCorrectedQuery")]
    pub html_corrected_query: Option<String>,
}

/// Google API Error envelope returned on HTTP error statuses.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct GoogleApiErrorResponse {
    #[serde(default)]
    pub error: Option<GoogleApiErrorPayload>,
}

/// Detailed error payload from Google API.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct GoogleApiErrorPayload {
    /// HTTP error status code
    #[serde(default)]
    pub code: Option<u16>,

    /// Error message
    #[serde(default)]
    pub message: Option<String>,

    /// Error status string (e.g. "PERMISSION_DENIED", "INVALID_ARGUMENT")
    #[serde(default)]
    pub status: Option<String>,

    /// List of detailed error objects
    #[serde(default)]
    pub errors: Option<Vec<GoogleApiErrorItem>>,
}

/// Individual error item inside Google's error payload.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct GoogleApiErrorItem {
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
}

/// Structured search output returned to the Carapace agent.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SearchResultOutput {
    /// The query that was searched
    pub query: String,

    /// Total search results estimated by Google
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_results: Option<String>,

    /// Search duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_time: Option<f64>,

    /// Suggested spelling correction if query had typos
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spelling_suggestion: Option<String>,

    /// Number of results returned in this response
    pub count: usize,

    /// List of formatted search result items
    pub results: Vec<SearchResultItem>,
}

/// Formatted individual result item.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SearchResultItem {
    /// Title of the webpage or result
    pub title: String,

    /// URL link to the page
    pub link: String,

    /// Summary snippet of the page
    pub snippet: String,

    /// Domain display link
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_link: Option<String>,
}
