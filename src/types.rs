//! Types and data structures for Google Vertex AI Search tool and API responses.

use serde::{Deserialize, Serialize};

/// Tool input arguments for the `google_search` tool.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct SearchArgs {
    /// Search query string (required)
    pub query: String,

    /// Number of search results to return (1-100, default 5)
    #[serde(default, alias = "page_size")]
    pub num: Option<u32>,

    /// Index of first result to return (1-based index or 0-based offset, for pagination)
    #[serde(default)]
    pub start: Option<u32>,

    /// 0-based offset of results to return (alternative to start)
    #[serde(default)]
    pub offset: Option<u32>,

    /// Google Cloud Project ID (can also be configured via credentials/config)
    #[serde(default)]
    pub project_id: Option<String>,

    /// Vertex AI Search Data Store ID
    #[serde(default, alias = "datastore_id", alias = "cx")]
    pub data_store_id: Option<String>,

    /// Vertex AI Search App / Engine ID (alternative to data_store_id)
    #[serde(default, alias = "app_id")]
    pub engine_id: Option<String>,

    /// Google Cloud location / region (default: "global")
    #[serde(default)]
    pub location: Option<String>,

    /// Vertex AI Search collection ID (default: "default_collection")
    #[serde(default)]
    pub collection_id: Option<String>,

    /// Vertex AI Search serving config ID (default: "default_search")
    #[serde(default, alias = "serving_config")]
    pub serving_config_id: Option<String>,

    /// Optional per-call Google Cloud API Key
    #[serde(default)]
    pub api_key: Option<String>,

    /// Optional per-call OAuth2 Access / Bearer Token
    #[serde(default, alias = "access_token", alias = "bearer_token")]
    pub auth_token: Option<String>,

    /// Filter expression for structured / metadata filtering
    #[serde(default)]
    pub filter: Option<String>,

    /// Order / sort expression (e.g. "publish_time desc")
    #[serde(default, alias = "sort")]
    pub order_by: Option<String>,

    /// Restrict search to a specific site or domain (appended as site: query filter)
    #[serde(default)]
    pub site: Option<String>,

    /// Restrict results to files with specified extension (appended as filetype: query filter)
    #[serde(default)]
    pub file_type: Option<String>,

    /// Phrase that all documents in the search results must contain
    #[serde(default)]
    pub exact_terms: Option<String>,

    /// Word or phrase that should not appear in any documents in search results
    #[serde(default)]
    pub exclude_terms: Option<String>,

    /// SafeSearch filter level ("active" or "off")
    #[serde(default)]
    pub safe: Option<String>,

    /// Language restriction code (e.g. "en")
    #[serde(default)]
    pub language: Option<String>,
}

/// JSON request body sent to Vertex AI Search `servingConfigs.search`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct VertexSearchRequestBody {
    /// The search query
    pub query: String,

    /// Number of results per page
    #[serde(rename = "pageSize", skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,

    /// 0-based offset for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,

    /// Filter expression
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,

    /// Order expression
    #[serde(rename = "orderBy", skip_serializing_if = "Option::is_none")]
    pub order_by: Option<String>,

    /// Content search specifications (snippets, summary, extractive answers)
    #[serde(rename = "contentSearchSpec", skip_serializing_if = "Option::is_none")]
    pub content_search_spec: Option<VertexContentSearchSpec>,

    /// Spell correction specifications
    #[serde(rename = "spellCorrectionSpec", skip_serializing_if = "Option::is_none")]
    pub spell_correction_spec: Option<VertexSpellCorrectionSpec>,
}

/// Content search specifications for Vertex AI Search.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct VertexContentSearchSpec {
    /// Snippet extraction configuration
    #[serde(rename = "snippetSpec", skip_serializing_if = "Option::is_none")]
    pub snippet_spec: Option<VertexSnippetSpec>,

    /// Extractive content configuration
    #[serde(rename = "extractiveContentSpec", skip_serializing_if = "Option::is_none")]
    pub extractive_content_spec: Option<VertexExtractiveContentSpec>,

    /// Generative summary configuration
    #[serde(rename = "summarySpec", skip_serializing_if = "Option::is_none")]
    pub summary_spec: Option<VertexSummarySpec>,
}

/// Snippet extraction specification.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct VertexSnippetSpec {
    /// Whether to return snippets for matching documents
    #[serde(rename = "returnSnippet")]
    pub return_snippet: bool,
}

/// Extractive content specification.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct VertexExtractiveContentSpec {
    /// Maximum number of extractive answers
    #[serde(rename = "maxExtractiveAnswerCount", skip_serializing_if = "Option::is_none")]
    pub max_extractive_answer_count: Option<u32>,

    /// Maximum number of extractive segments
    #[serde(rename = "maxExtractiveSegmentCount", skip_serializing_if = "Option::is_none")]
    pub max_extractive_segment_count: Option<u32>,
}

/// Generative summary specification.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct VertexSummarySpec {
    /// Number of top search results to summarize
    #[serde(rename = "summaryResultCount", skip_serializing_if = "Option::is_none")]
    pub summary_result_count: Option<u32>,

    /// Whether to include citations in the summary
    #[serde(rename = "includeCitations", skip_serializing_if = "Option::is_none")]
    pub include_citations: Option<bool>,
}

/// Spell correction specification.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct VertexSpellCorrectionSpec {
    /// Spell correction mode ("SUGGESTION_ONLY" or "AUTO")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

/// Raw Vertex AI Search / Discovery Engine JSON response.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct VertexSearchApiResponse {
    /// Search result items
    #[serde(default)]
    pub results: Option<Vec<VertexSearchResult>>,

    /// Total matching results count (can be integer or string)
    #[serde(default, rename = "totalSize")]
    pub total_size: Option<serde_json::Value>,

    /// Suggested spelling correction
    #[serde(default, rename = "correctedQuery")]
    pub corrected_query: Option<String>,

    /// Next page token for pagination
    #[serde(default, rename = "nextPageToken")]
    pub next_page_token: Option<String>,

    /// Generative summary if requested
    #[serde(default)]
    pub summary: Option<VertexSearchSummary>,

    /// Error payload (if returned in 200/error envelope)
    #[serde(default)]
    pub error: Option<GoogleApiErrorPayload>,
}

/// A single search result item returned by Vertex AI Search.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct VertexSearchResult {
    /// Result ID
    #[serde(default)]
    pub id: Option<String>,

    /// Document data
    #[serde(default)]
    pub document: Option<VertexDocument>,
}

/// Document object inside Vertex AI Search result.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct VertexDocument {
    /// Full resource name of the document
    #[serde(default)]
    pub name: Option<String>,

    /// Document ID
    #[serde(default)]
    pub id: Option<String>,

    /// Derived structured data populated by Vertex AI Search crawler/indexer
    #[serde(default, rename = "derivedStructData")]
    pub derived_struct_data: Option<VertexDerivedStructData>,

    /// User-provided structured data if available
    #[serde(default, rename = "structData")]
    pub struct_data: Option<serde_json::Value>,
}

/// Derived struct data inside Vertex AI document.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct VertexDerivedStructData {
    /// Title of the document / webpage
    #[serde(default)]
    pub title: Option<String>,

    /// Link / URL to the document / webpage
    #[serde(default)]
    pub link: Option<String>,

    /// Alternative URI field
    #[serde(default)]
    pub uri: Option<String>,

    /// Alternative URL field
    #[serde(default)]
    pub url: Option<String>,

    /// Display link / domain
    #[serde(default, rename = "displayLink")]
    pub display_link: Option<String>,

    /// Formatted URL
    #[serde(default, rename = "formattedUrl")]
    pub formatted_url: Option<String>,

    /// Extracted snippets
    #[serde(default)]
    pub snippets: Option<Vec<VertexSnippetItem>>,

    /// Extractive answers
    #[serde(default, rename = "extractive_answers")]
    pub extractive_answers: Option<Vec<VertexExtractiveItem>>,

    /// Extractive segments
    #[serde(default, rename = "extractive_segments")]
    pub extractive_segments: Option<Vec<VertexExtractiveItem>>,
}

/// Snippet object inside derived struct data.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct VertexSnippetItem {
    /// Snippet text
    #[serde(default)]
    pub snippet: Option<String>,

    /// Snippet status (e.g. "SUCCESS", "NO_SNIPPET_AVAILABLE")
    #[serde(default, rename = "snippet_status")]
    pub snippet_status: Option<String>,
}

/// Extractive answer / segment item.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct VertexExtractiveItem {
    /// Extracted content
    #[serde(default)]
    pub content: Option<String>,

    /// Page number
    #[serde(default, rename = "pageNumber")]
    pub page_number: Option<String>,
}

/// Summary object returned by Vertex AI Search.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct VertexSearchSummary {
    /// Generated summary text
    #[serde(default, rename = "summaryText")]
    pub summary_text: Option<String>,

    /// Whether summary generation was skipped
    #[serde(default, rename = "summarySkipped")]
    pub summary_skipped: Option<bool>,
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

    /// Details array from Google RPC error
    #[serde(default)]
    pub details: Option<Vec<serde_json::Value>>,
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

    /// Total search results estimated by Vertex AI Search
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_results: Option<String>,

    /// Generative summary if produced by Vertex AI Search
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

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

    /// Summary snippet or extractive content of the page
    pub snippet: String,

    /// Domain display link
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_link: Option<String>,
}
