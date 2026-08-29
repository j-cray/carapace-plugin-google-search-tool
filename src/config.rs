//! Configuration and credentials resolution for Google Vertex AI Search.

use crate::types::SearchArgs;

/// Candidate keys for Google Cloud Project ID.
pub const PROJECT_ID_CANDIDATE_KEYS: &[&str] = &[
    "project_id",
    "project-id",
    "project",
    "gcp_project",
    "gcp_project_id",
    "gcp-project-id",
];

/// Candidate keys for Vertex AI Search Data Store ID or Engine ID.
pub const TARGET_ID_CANDIDATE_KEYS: &[&str] = &[
    "data_store_id",
    "data-store-id",
    "datastore_id",
    "datastore",
    "data_store",
    "engine_id",
    "engine-id",
    "engine",
    "app_id",
    "app-id",
    "search_engine_id",
    "cx",
];

/// Candidate keys for Google Cloud region / location.
pub const LOCATION_CANDIDATE_KEYS: &[&str] = &["location", "region"];

/// Candidate keys for Vertex collection ID.
pub const COLLECTION_CANDIDATE_KEYS: &[&str] = &["collection_id", "collection-id", "collection"];

/// Candidate keys for Vertex serving configuration ID.
pub const SERVING_CONFIG_CANDIDATE_KEYS: &[&str] = &[
    "serving_config_id",
    "serving-config-id",
    "serving_config",
    "serving-config",
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

/// Candidate keys for Google Cloud API Key.
pub const API_KEY_CANDIDATE_KEYS: &[&str] = &[
    "api_key",
    "api-key",
    "key",
    "google_api_key",
    "vertex_api_key",
];

/// Default location for Vertex AI Search.
pub const DEFAULT_LOCATION: &str = "global";
/// Default collection for Vertex AI Search.
pub const DEFAULT_COLLECTION: &str = "default_collection";
/// Default serving config ID for Vertex AI Search.
pub const DEFAULT_SERVING_CONFIG: &str = "default_search";

/// Resolved search target (Data Store or Engine / Search App).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchTarget {
    DataStore(String),
    Engine(String),
}

/// Resolved authentication credentials.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedAuth {
    pub auth_token: Option<String>,
    pub api_key: Option<String>,
}

/// Fully resolved configuration for executing a Vertex AI Search request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedConfig {
    pub project_id: String,
    pub target: SearchTarget,
    pub location: String,
    pub collection_id: String,
    pub serving_config_id: String,
    pub auth: ResolvedAuth,
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

/// Resolves the GCP Project ID.
pub fn resolve_project_id_with<FC, FK>(
    args_project_id: Option<&str>,
    credential_get: &FC,
    config_get: &FK,
) -> Result<String, String>
where
    FC: Fn(&str) -> Option<String>,
    FK: Fn(&str) -> Option<String>,
{
    resolve_candidate_value(
        args_project_id,
        PROJECT_ID_CANDIDATE_KEYS,
        credential_get,
        config_get,
    )
    .ok_or_else(|| {
        "Missing Google Cloud Project ID. Configure it with 'carapace credentials set google-search-tool:project_id <PROJECT_ID>', \
         in carapace.json under plugins.google-search-tool.project_id, or via the 'project_id' tool parameter."
            .to_string()
    })
}

/// Resolves the Search Target (Data Store ID or Engine ID).
pub fn resolve_target_with<FC, FK>(
    args: &SearchArgs,
    credential_get: &FC,
    config_get: &FK,
) -> Result<SearchTarget, String>
where
    FC: Fn(&str) -> Option<String>,
    FK: Fn(&str) -> Option<String>,
{
    // Check explicit engine_id in args first
    if let Some(ref eng) = args.engine_id {
        let trimmed = eng.trim();
        if !trimmed.is_empty() {
            return Ok(SearchTarget::Engine(trimmed.to_string()));
        }
    }

    // Check explicit data_store_id in args
    if let Some(ref ds) = args.data_store_id {
        let trimmed = ds.trim();
        if !trimmed.is_empty() {
            return Ok(SearchTarget::DataStore(trimmed.to_string()));
        }
    }

    // Check credential / config for engine_id specifically
    if let Some(eng) = resolve_candidate_value(
        None,
        &["engine_id", "engine-id", "engine", "app_id", "app-id"],
        credential_get,
        config_get,
    ) {
        return Ok(SearchTarget::Engine(eng));
    }

    // Check general target candidates (defaulting to DataStore)
    if let Some(target) =
        resolve_candidate_value(None, TARGET_ID_CANDIDATE_KEYS, credential_get, config_get)
    {
        return Ok(SearchTarget::DataStore(target));
    }

    Err(
        "Missing Vertex AI Search Data Store ID or Engine ID. Configure it with 'carapace credentials set google-search-tool:data_store_id <DATA_STORE_ID>' \
         or 'carapace credentials set google-search-tool:engine_id <ENGINE_ID>', \
         in carapace.json, or via the 'data_store_id'/'engine_id' tool parameter."
            .to_string(),
    )
}

/// Resolves the Google Cloud location / region (defaults to "global").
pub fn resolve_location_with<FC, FK>(
    args_location: Option<&str>,
    credential_get: &FC,
    config_get: &FK,
) -> String
where
    FC: Fn(&str) -> Option<String>,
    FK: Fn(&str) -> Option<String>,
{
    resolve_candidate_value(
        args_location,
        LOCATION_CANDIDATE_KEYS,
        credential_get,
        config_get,
    )
    .unwrap_or_else(|| DEFAULT_LOCATION.to_string())
}

/// Resolves the Vertex collection ID (defaults to "default_collection").
pub fn resolve_collection_with<FC, FK>(
    args_collection: Option<&str>,
    credential_get: &FC,
    config_get: &FK,
) -> String
where
    FC: Fn(&str) -> Option<String>,
    FK: Fn(&str) -> Option<String>,
{
    resolve_candidate_value(
        args_collection,
        COLLECTION_CANDIDATE_KEYS,
        credential_get,
        config_get,
    )
    .unwrap_or_else(|| DEFAULT_COLLECTION.to_string())
}

/// Resolves the Vertex serving configuration ID (defaults to "default_search").
pub fn resolve_serving_config_with<FC, FK>(
    args_serving_config: Option<&str>,
    credential_get: &FC,
    config_get: &FK,
) -> String
where
    FC: Fn(&str) -> Option<String>,
    FK: Fn(&str) -> Option<String>,
{
    resolve_candidate_value(
        args_serving_config,
        SERVING_CONFIG_CANDIDATE_KEYS,
        credential_get,
        config_get,
    )
    .unwrap_or_else(|| DEFAULT_SERVING_CONFIG.to_string())
}

/// Resolves authentication credentials (API Key or Bearer Token).
pub fn resolve_auth_with<FC, FK>(
    args: &SearchArgs,
    credential_get: &FC,
    config_get: &FK,
) -> Result<ResolvedAuth, String>
where
    FC: Fn(&str) -> Option<String>,
    FK: Fn(&str) -> Option<String>,
{
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

    if auth_token.is_none() && api_key.is_none() {
        return Err(
            "Missing authentication credentials. Configure an API key with 'carapace credentials set google-search-tool:api_key <API_KEY>' \
             or an OAuth2 Bearer token with 'carapace credentials set google-search-tool:auth_token <ACCESS_TOKEN>', \
             in carapace.json, or via tool parameters."
                .to_string(),
        );
    }

    Ok(ResolvedAuth {
        auth_token,
        api_key,
    })
}

/// Resolves full configuration required to invoke Vertex AI Search.
pub fn resolve_full_config_with<FC, FK>(
    args: &SearchArgs,
    credential_get: &FC,
    config_get: &FK,
) -> Result<ResolvedConfig, String>
where
    FC: Fn(&str) -> Option<String>,
    FK: Fn(&str) -> Option<String>,
{
    let project_id =
        resolve_project_id_with(args.project_id.as_deref(), credential_get, config_get)?;
    let target = resolve_target_with(args, credential_get, config_get)?;
    let location = resolve_location_with(args.location.as_deref(), credential_get, config_get);
    let collection_id =
        resolve_collection_with(args.collection_id.as_deref(), credential_get, config_get);
    let serving_config_id = resolve_serving_config_with(
        args.serving_config_id.as_deref(),
        credential_get,
        config_get,
    );
    let auth = resolve_auth_with(args, credential_get, config_get)?;

    Ok(ResolvedConfig {
        project_id,
        target,
        location,
        collection_id,
        serving_config_id,
        auth,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_project_id_from_args() {
        let res = resolve_project_id_with(Some("my-proj"), &|_| None, &|_| None);
        assert_eq!(res.unwrap(), "my-proj");
    }

    #[test]
    fn test_resolve_project_id_from_credentials() {
        let res = resolve_project_id_with(
            None,
            &|k| if k == "project_id" { Some("cred-proj".to_string()) } else { None },
            &|_| None,
        );
        assert_eq!(res.unwrap(), "cred-proj");
    }

    #[test]
    fn test_resolve_project_id_from_config() {
        let res = resolve_project_id_with(
            None,
            &|_| None,
            &|k| if k == "gcp_project" { Some("cfg-proj".to_string()) } else { None },
        );
        assert_eq!(res.unwrap(), "cfg-proj");
    }

    #[test]
    fn test_resolve_project_id_missing() {
        let res = resolve_project_id_with(None, &|_| None, &|_| None);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Missing Google Cloud Project ID"));
    }

    #[test]
    fn test_resolve_target_datastore_from_args() {
        let args = SearchArgs {
            query: "test".to_string(),
            data_store_id: Some("my-ds".to_string()),
            ..Default::default()
        };
        let res = resolve_target_with(&args, &|_| None, &|_| None).unwrap();
        assert_eq!(res, SearchTarget::DataStore("my-ds".to_string()));
    }

    #[test]
    fn test_resolve_target_engine_from_args() {
        let args = SearchArgs {
            query: "test".to_string(),
            engine_id: Some("my-engine".to_string()),
            ..Default::default()
        };
        let res = resolve_target_with(&args, &|_| None, &|_| None).unwrap();
        assert_eq!(res, SearchTarget::Engine("my-engine".to_string()));
    }

    #[test]
    fn test_resolve_target_legacy_cx_from_credentials() {
        let args = SearchArgs {
            query: "test".to_string(),
            ..Default::default()
        };
        let res = resolve_target_with(
            &args,
            &|k| if k == "cx" { Some("legacy-cx-id".to_string()) } else { None },
            &|_| None,
        )
        .unwrap();
        assert_eq!(res, SearchTarget::DataStore("legacy-cx-id".to_string()));
    }

    #[test]
    fn test_resolve_auth_api_key() {
        let args = SearchArgs {
            query: "test".to_string(),
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };
        let auth = resolve_auth_with(&args, &|_| None, &|_| None).unwrap();
        assert_eq!(auth.api_key.as_deref(), Some("test-key"));
        assert_eq!(auth.auth_token, None);
    }

    #[test]
    fn test_resolve_auth_token_from_credentials() {
        let args = SearchArgs {
            query: "test".to_string(),
            ..Default::default()
        };
        let auth = resolve_auth_with(
            &args,
            &|k| if k == "access_token" { Some("token123".to_string()) } else { None },
            &|_| None,
        )
        .unwrap();
        assert_eq!(auth.auth_token.as_deref(), Some("token123"));
    }

    #[test]
    fn test_resolve_auth_missing() {
        let args = SearchArgs {
            query: "test".to_string(),
            ..Default::default()
        };
        let res = resolve_auth_with(&args, &|_| None, &|_| None);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Missing authentication credentials"));
    }

    #[test]
    fn test_resolve_full_config() {
        let args = SearchArgs {
            query: "rust".to_string(),
            project_id: Some("test-proj".to_string()),
            data_store_id: Some("test-ds".to_string()),
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };
        let cfg = resolve_full_config_with(&args, &|_| None, &|_| None).unwrap();
        assert_eq!(cfg.project_id, "test-proj");
        assert_eq!(cfg.target, SearchTarget::DataStore("test-ds".to_string()));
        assert_eq!(cfg.location, "global");
        assert_eq!(cfg.collection_id, "default_collection");
        assert_eq!(cfg.serving_config_id, "default_search");
        assert_eq!(cfg.auth.api_key.as_deref(), Some("test-key"));
    }
}
