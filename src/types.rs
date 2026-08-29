//! Types and data structures for Google Search Tool via Google Search Grounding.

use serde::{Deserialize, Serialize};

/// Tool input arguments for the `google_search` tool.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct SearchArgs {
    /// Search query string (required)
    pub query: String,

    /// Number of search results / sources to return (default: 5, max: 20)
    #[serde(default, alias = "page_size")]
    pub num: Option<u32>,

    /// Gemini model to use for grounded web search (default: "gemini-2.5-flash")
    #[serde(default)]
    pub model: Option<String>,

    /// Optional per-call Google Gemini / Google Cloud API Key
    #[serde(default)]
    pub api_key: Option<String>,

    /// Optional per-call OAuth2 Bearer / access token (for Google Cloud Vertex AI)
    #[serde(default, alias = "access_token", alias = "bearer_token")]
    pub auth_token: Option<String>,

    /// Optional Google Cloud Project ID (for Vertex AI endpoint)
    #[serde(default)]
    pub project_id: Option<String>,

    /// Optional Google Cloud region / location (default: "us-central1" for Vertex AI)
    #[serde(default)]
    pub location: Option<String>,

    /// Restrict search to a specific site or domain (e.g. 'docs.rs' or 'github.com')
    #[serde(default)]
    pub site: Option<String>,

    /// Phrase that search results should contain
    #[serde(default)]
    pub exact_terms: Option<String>,

    /// Word or phrase to exclude from search results
    #[serde(default)]
    pub exclude_terms: Option<String>,
}

/// JSON request body for Google AI Studio / Gemini generateContent with search grounding.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct GeminiAiStudioRequestBody {
    pub contents: Vec<GeminiContent>,
    pub tools: Vec<GeminiAiStudioTool>,
}

/// Tool spec for Google AI Studio grounding.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct GeminiAiStudioTool {
    pub google_search: serde_json::Value,
}

/// JSON request body for Vertex AI generateContent with search grounding.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct VertexAiRequestBody {
    pub contents: Vec<GeminiContent>,
    pub tools: Vec<VertexAiTool>,
}

/// Tool spec for Vertex AI grounding.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct VertexAiTool {
    #[serde(rename = "googleSearch")]
    pub google_search: serde_json::Value,
}

/// Content object containing role and parts.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct GeminiContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    pub parts: Vec<GeminiPart>,
}

/// Individual part inside content.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct GeminiPart {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// Root JSON response from Gemini generateContent with search grounding.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct GeminiGenerateContentResponse {
    #[serde(default)]
    pub candidates: Option<Vec<GeminiCandidate>>,
    #[serde(default)]
    pub error: Option<GoogleApiErrorPayload>,
}

/// Candidate item in Gemini response.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct GeminiCandidate {
    #[serde(default)]
    pub content: Option<GeminiContent>,
    #[serde(default, rename = "groundingMetadata")]
    pub grounding_metadata: Option<GeminiGroundingMetadata>,
}

/// Grounding metadata containing web search queries and source chunks.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct GeminiGroundingMetadata {
    #[serde(default, rename = "webSearchQueries")]
    pub web_search_queries: Option<Vec<String>>,
    #[serde(default, rename = "groundingChunks")]
    pub grounding_chunks: Option<Vec<GeminiGroundingChunk>>,
    #[serde(default, rename = "groundingSupports")]
    pub grounding_supports: Option<Vec<GeminiGroundingSupport>>,
    #[serde(default, rename = "searchEntryPoint")]
    pub search_entry_point: Option<GeminiSearchEntryPoint>,
}

/// Grounding source chunk.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct GeminiGroundingChunk {
    #[serde(default)]
    pub web: Option<GeminiWebChunk>,
}

/// Web chunk containing page URI and Title.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct GeminiWebChunk {
    #[serde(default)]
    pub uri: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
}

/// Grounding support mapping text segments to chunk indices.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct GeminiGroundingSupport {
    #[serde(default, rename = "groundingChunkIndices")]
    pub grounding_chunk_indices: Option<Vec<usize>>,
    #[serde(default, rename = "confidenceScores")]
    pub confidence_scores: Option<Vec<f64>>,
    #[serde(default)]
    pub segment: Option<GeminiSegment>,
}

/// Segment of text supported by grounding sources.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct GeminiSegment {
    #[serde(default, rename = "startIndex")]
    pub start_index: Option<usize>,
    #[serde(default, rename = "endIndex")]
    pub end_index: Option<usize>,
    #[serde(default)]
    pub text: Option<String>,
}

/// Search entry point metadata.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct GeminiSearchEntryPoint {
    #[serde(default, rename = "renderedContent")]
    pub rendered_content: Option<String>,
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
    #[serde(default)]
    pub code: Option<u16>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub errors: Option<Vec<GoogleApiErrorItem>>,
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

    /// Comprehensive grounded answer synthesized from the live web
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// Live Google web search queries executed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_queries: Option<Vec<String>>,

    /// Number of web source citations returned
    pub count: usize,

    /// List of formatted search result sources from the whole web
    pub results: Vec<SearchResultItem>,
}

/// Formatted individual result item.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SearchResultItem {
    /// Title of the webpage or resource
    pub title: String,

    /// URL link to the page
    pub link: String,

    /// Excerpt or supported fact from the page
    pub snippet: String,

    /// Domain display link
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_link: Option<String>,
}
