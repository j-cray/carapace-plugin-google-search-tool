//! Configuration and credentials resolution for the Google Search tool.

/// Keys checked in the credential and configuration stores for the Google API key.
pub const API_KEY_CANDIDATE_KEYS: &[&str] = &["api_key", "api-key", "key"];

/// Keys checked in the credential and configuration stores for the Custom Search Engine ID.
pub const CX_CANDIDATE_KEYS: &[&str] = &["cx", "search_engine_id", "search-engine-id"];

/// Resolves the Google API Key following the priority chain:
/// 1. Per-call tool argument (`api_key`)
/// 2. Host credential store (`credential_get`)
/// 3. Host plugin configuration (`config_get`)
pub fn resolve_api_key_with<FC, FK>(
    args_key: Option<&str>,
    credential_get: FC,
    config_get: FK,
) -> Result<String, String>
where
    FC: Fn(&str) -> Option<String>,
    FK: Fn(&str) -> Option<String>,
{
    if let Some(key) = args_key {
        let trimmed = key.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    for key_name in API_KEY_CANDIDATE_KEYS {
        if let Some(val) = credential_get(key_name) {
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
    }

    for key_name in API_KEY_CANDIDATE_KEYS {
        if let Some(val) = config_get(key_name) {
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
    }

    Err(
        "Missing Google API Key. Configure it with 'carapace credentials set google-search-tool:api_key <KEY>', \
         in carapace.json under plugins.google-search-tool.api_key, or via the 'api_key' tool parameter."
            .to_string(),
    )
}

/// Resolves the Custom Search Engine ID (`cx`) following the priority chain:
/// 1. Per-call tool argument (`cx`)
/// 2. Host credential store (`credential_get`)
/// 3. Host plugin configuration (`config_get`)
pub fn resolve_cx_with<FC, FK>(
    args_cx: Option<&str>,
    credential_get: FC,
    config_get: FK,
) -> Result<String, String>
where
    FC: Fn(&str) -> Option<String>,
    FK: Fn(&str) -> Option<String>,
{
    if let Some(cx) = args_cx {
        let trimmed = cx.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    for cx_name in CX_CANDIDATE_KEYS {
        if let Some(val) = credential_get(cx_name) {
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
    }

    for cx_name in CX_CANDIDATE_KEYS {
        if let Some(val) = config_get(cx_name) {
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
    }

    Err(
        "Missing Google Custom Search Engine ID (cx). Configure it with 'carapace credentials set google-search-tool:cx <CX>', \
         in carapace.json under plugins.google-search-tool.cx, or via the 'cx' tool parameter."
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_api_key_from_args() {
        let res = resolve_api_key_with(Some("arg_key_123"), |_| None, |_| None);
        assert_eq!(res.unwrap(), "arg_key_123");
    }

    #[test]
    fn test_resolve_api_key_from_credentials() {
        let res = resolve_api_key_with(
            None,
            |k| if k == "api_key" { Some("cred_key".to_string()) } else { None },
            |_| None,
        );
        assert_eq!(res.unwrap(), "cred_key");
    }

    #[test]
    fn test_resolve_api_key_from_config() {
        let res = resolve_api_key_with(
            None,
            |_| None,
            |k| if k == "key" { Some("cfg_key".to_string()) } else { None },
        );
        assert_eq!(res.unwrap(), "cfg_key");
    }

    #[test]
    fn test_resolve_api_key_missing() {
        let res = resolve_api_key_with(None, |_| None, |_| None);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Missing Google API Key"));
    }

    #[test]
    fn test_resolve_cx_from_args() {
        let res = resolve_cx_with(Some("0123:cx"), |_| None, |_| None);
        assert_eq!(res.unwrap(), "0123:cx");
    }

    #[test]
    fn test_resolve_cx_from_credentials() {
        let res = resolve_cx_with(
            None,
            |k| if k == "search_engine_id" { Some("cx_cred".to_string()) } else { None },
            |_| None,
        );
        assert_eq!(res.unwrap(), "cx_cred");
    }

    #[test]
    fn test_resolve_cx_missing() {
        let res = resolve_cx_with(None, |_| None, |_| None);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Missing Google Custom Search Engine ID"));
    }
}
