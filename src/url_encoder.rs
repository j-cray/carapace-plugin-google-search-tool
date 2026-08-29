//! URL query parameter encoding utilities.
//!
//! Provides pure-Rust percent-encoding for URL query parameters without external OS/libc dependencies.

use core::fmt::Write;

/// Percent-encode a string according to RFC 3986 (query component rules).
///
/// Unreserved characters: A-Z, a-z, 0-9, '-', '_', '.', '~'
/// All other characters are encoded as %XX in uppercase hexadecimal.
pub fn url_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len());
    for byte in input.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(*byte as char);
            }
            _ => {
                let _ = write!(encoded, "%{:02X}", byte);
            }
        }
    }
    encoded
}

/// Helper struct for building a query string.
#[derive(Debug, Default)]
pub struct QueryBuilder {
    params: Vec<(String, String)>,
}

impl QueryBuilder {
    /// Create a new query builder.
    pub fn new() -> Self {
        Self { params: Vec::new() }
    }

    /// Add a key-value pair to the query string.
    pub fn add_param(&mut self, key: &str, value: &str) -> &mut Self {
        self.params.push((key.to_string(), value.to_string()));
        self
    }

    /// Add an optional key-value pair if the value is `Some`.
    pub fn add_optional_param(&mut self, key: &str, value: Option<&str>) -> &mut Self {
        if let Some(v) = value {
            if !v.trim().is_empty() {
                self.add_param(key, v.trim());
            }
        }
        self
    }

    /// Build the query string starting with '?' or empty if no params.
    pub fn build_query_string(&self) -> String {
        if self.params.is_empty() {
            return String::new();
        }

        let mut query = String::from("?");
        for (i, (k, v)) in self.params.iter().enumerate() {
            if i > 0 {
                query.push('&');
            }
            query.push_str(&url_encode(k));
            query.push('=');
            query.push_str(&url_encode(v));
        }
        query
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_encode_simple() {
        assert_eq!(url_encode("hello"), "hello");
        assert_eq!(url_encode("hello world"), "hello%20world");
        assert_eq!(url_encode("rust & wasm"), "rust%20%26%20wasm");
        assert_eq!(url_encode("foo/bar?baz=1"), "foo%2Fbar%3Fbaz%3D1");
        assert_eq!(url_encode("test-value_1.0~v2"), "test-value_1.0~v2");
    }

    #[test]
    fn test_query_builder() {
        let mut qb = QueryBuilder::new();
        qb.add_param("q", "rust wasm component");
        qb.add_param("num", "10");
        qb.add_optional_param("site", Some("docs.rs"));
        qb.add_optional_param("empty", None);
        qb.add_optional_param("blank", Some("   "));

        let query = qb.build_query_string();
        assert_eq!(query, "?q=rust%20wasm%20component&num=10&site=docs.rs");
    }
}
