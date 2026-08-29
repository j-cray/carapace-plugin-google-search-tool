//! Configuration and credentials resolution for Google Search Tool.

use crate::types::SearchArgs;

/// Default Gemini model to use for grounded live web search.
pub const DEFAULT_MODEL: &str = "gemini-2.5-flash";

/// Default Google Cloud region for Vertex AI.
pub const DEFAULT_VERTEX_LOCATION: &str = "us-central1";

/// Candidate keys for Gemini / Google API Key.
pub const API_KEY_CANDIDATE_KEYS: &[&str] = &[
    "api_key",
    "api-key",
    "gemini_api_key",
    "gemini-api-key",
    "google_api_key",
    "google-api-key",
    "key",
];

/// Candidate keys for OAuth2 Bearer / access token.
pub const AUTH_TOKEN_CANDIDATE_KEYS: &[&str] = &[
    "auth_token",
    "auth-token",
    "access_token",
    "access-token",
    "bearer_token",
    "bearer-token",
    "token",
];

/// Candidate keys for Google Cloud Project ID.
pub const PROJECT_ID_CANDIDATE_KEYS: &[&str] = &[
    "project_id",
    "project-id",
    "project",
    "gcp_project",
    "gcp_project_id",
    "gcp-project-id",
];

/// Candidate keys for Google Cloud region / location.
pub const LOCATION_CANDIDATE_KEYS: &[&str] = &["location", "region"];

/// Candidate keys for Gemini model override.
pub const MODEL_CANDIDATE_KEYS: &[&str] = &["model", "model_id", "model-id", "gemini_model"];

/// Resolved Backend mode for Google Search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendMode {
    /// Google AI Studio endpoint (simple API key, global web search)
    GoogleAiStudio { api_key: String },
    /// Google Cloud Vertex AI endpoint (GCP Project ID + OAuth Bearer token or API key)
    VertexAi {
        project_id: String,
        location: String,
        auth_token: Option<String>,
        api_key: Option<String>,
    },
}

/// Fully resolved configuration for executing a search request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedConfig {
    pub model: String,
    pub backend: BackendMode,
}

/// Resolves a single string value from args, credential store, or config store using key candidates.
fn resolve_candidate_value<FC, FK>(
    arg_val: Option<&str>,
    candidate_keys: &[&str],
    credential_get: &FC,
    config_get: &FK,
) -> Option<String>
where
    FC: Fn(&str) -> Option<String>,
    FK: Fn(&str) -> Option<String>,
{
    if let Some(val) = arg_val {
        let trimmed = val.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    for key_name in candidate_keys {
        if let Some(val) = credential_get(key_name) {
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    for key_name in candidate_keys {
        if let Some(val) = config_get(key_name) {
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    None
}

/// Resolves the Gemini model name (default: "gemini-2.5-flash").
pub fn resolve_model_with<FC, FK>(
    args_model: Option<&str>,
    credential_get: &FC,
    config_get: &FK,
) -> String
where
    FC: Fn(&str) -> Option<String>,
    FK: Fn(&str) -> Option<String>,
{
    resolve_candidate_value(args_model, MODEL_CANDIDATE_KEYS, credential_get, config_get)
        .unwrap_or_else(|| DEFAULT_MODEL.to_string())
}

/// Resolves full configuration required to invoke Google Search Grounding.
pub fn resolve_full_config_with<FC, FK>(
    args: &SearchArgs,
    credential_get: &FC,
    config_get: &FK,
) -> Result<ResolvedConfig, String>
where
    FC: Fn(&str) -> Option<String>,
    FK: Fn(&str) -> Option<String>,
{
    let model = resolve_model_with(args.model.as_deref(), credential_get, config_get);

    let project_id = resolve_candidate_value(
        args.project_id.as_deref(),
        PROJECT_ID_CANDIDATE_KEYS,
        credential_get,
        config_get,
    );

    let auth_token = resolve_candidate_value(
        args.auth_token.as_deref(),
        AUTH_TOKEN_CANDIDATE_KEYS,
        credential_get,
        config_get,
    );

    let api_key = resolve_candidate_value(
        args.api_key.as_deref(),
        API_KEY_CANDIDATE_KEYS,
        credential_get,
        config_get,
    );

    let location = resolve_candidate_value(
        args.location.as_deref(),
        LOCATION_CANDIDATE_KEYS,
        credential_get,
        config_get,
    )
    .unwrap_or_else(|| DEFAULT_VERTEX_LOCATION.to_string());

    if let Some(proj) = project_id {
        if auth_token.is_none() && api_key.is_none() {
            return Err(
                "When using Google Cloud Vertex AI (project_id), you must provide authentication credentials \
                 via 'carapace credentials set google-search-tool:auth_token <ACCESS_TOKEN>' or 'carapace credentials set google-search-tool:api_key <API_KEY>'."
                    .to_string(),
            );
        }

        return Ok(ResolvedConfig {
            model,
            backend: BackendMode::VertexAi {
                project_id: proj,
                location,
                auth_token,
                api_key,
            },
        });
    }

    if let Some(token) = auth_token {
        // If auth_token is provided without project_id, check if project_id is needed or use token for Vertex AI
        return Err(
            "OAuth2 Bearer tokens require a Google Cloud Project ID. Set it with 'carapace credentials set google-search-tool:project_id <PROJECT_ID>'."
                .to_string(),
        );
    }

    if let Some(key) = api_key {
        return Ok(ResolvedConfig {
            model,
            backend: BackendMode::GoogleAiStudio { api_key: key },
        });
    }

    Err(
        "Missing Google Search API credentials. To search the entire web, set your Gemini API key with:\n\
         'carapace credentials set google-search-tool:api_key <YOUR_GEMINI_API_KEY>'\n\
         (Get a free API key at https://aistudio.google.com/app/apikey)\n\
         Or configure a Google Cloud Vertex AI project with 'project_id' and 'auth_token'."
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_google_ai_studio_api_key_from_args() {
        let args = SearchArgs {
            query: "test".to_string(),
            api_key: Some("test-key-123".to_string()),
            ..Default::default()
        };
        let cfg = resolve_full_config_with(&args, &|_| None, &|_| None).unwrap();
        assert_eq!(cfg.model, DEFAULT_MODEL);
        assert_eq!(
            cfg.backend,
            BackendMode::GoogleAiStudio {
                api_key: "test-key-123".to_string()
            }
        );
    }

    #[test]
    fn test_resolve_google_ai_studio_api_key_from_credentials() {
        let args = SearchArgs {
            query: "test".to_string(),
            ..Default::default()
        };
        let cfg = resolve_full_config_with(
            &args,
            &|k| if k == "gemini_api_key" { Some("key-from-cred".to_string()) } else { None },
            &|_| None,
        )
        .unwrap();
        assert_eq!(
            cfg.backend,
            BackendMode::GoogleAiStudio {
                api_key: "key-from-cred".to_string()
            }
        );
    }

    #[test]
    fn test_resolve_vertex_ai_mode() {
        let args = SearchArgs {
            query: "test".to_string(),
            project_id: Some("my-gcp-proj".to_string()),
            auth_token: Some("token123".to_string()),
            model: Some("gemini-2.0-flash".to_string()),
            ..Default::default()
        };
        let cfg = resolve_full_config_with(&args, &|_| None, &|_| None).unwrap();
        assert_eq!(cfg.model, "gemini-2.0-flash");
        assert_eq!(
            cfg.backend,
            BackendMode::VertexAi {
                project_id: "my-gcp-proj".to_string(),
                location: "us-central1".to_string(),
                auth_token: Some("token123".to_string()),
                api_key: None,
            }
        );
    }

    #[test]
    fn test_resolve_missing_credentials_error() {
        let args = SearchArgs {
            query: "test".to_string(),
            ..Default::default()
        };
        let res = resolve_full_config_with(&args, &|_| None, &|_| None);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Missing Google Search API credentials"));
    }
}
