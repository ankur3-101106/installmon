use regex::Regex;
use std::sync::LazyLock;

static SENSITIVE_FLAG_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^--?(password|pass|passwd|token|auth|api[-_]?key|secret|access[-_]?key|private[-_]?key)$").unwrap()
});

static INLINE_FLAG_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(--?(?:password|pass|passwd|token|auth|api[-_]?key|secret|access[-_]?key|private[-_]?key))=(\S+)$").unwrap()
});

static URL_CREDENTIAL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([a-zA-Z][a-zA-Z0-9+.-]*://[^:/]+):([^@\s]+)@").unwrap());

static AUTH_HEADER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(Authorization:\s*(?:Bearer|Basic|Token)\s+)\S+").unwrap());

static PRIVATE_KEY_BLOCK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"-----BEGIN [A-Z ]*PRIVATE KEY-----[\s\S]*?-----END [A-Z ]*PRIVATE KEY-----")
        .unwrap()
});

/// Redacted marker string used in place of sensitive content.
pub const REDACTED_VALUE: &str = "[REDACTED]";

/// Sanitizes a command-line argument list, ensuring sensitive credentials,
/// passwords, tokens, and keys are scrubbed before persistence or logging.
pub fn sanitize_arguments(args: &[String]) -> Vec<String> {
    let mut sanitized = Vec::with_capacity(args.len());
    let mut redact_next = false;

    for arg in args {
        if redact_next {
            sanitized.push(REDACTED_VALUE.to_string());
            redact_next = false;
            continue;
        }

        // Check if current arg is an inline sensitive flag: e.g. --token=xyz
        if let Some(caps) = INLINE_FLAG_REGEX.captures(arg) {
            let flag = caps.get(1).map_or("", |m| m.as_str());
            sanitized.push(format!("{}={}", flag, REDACTED_VALUE));
            continue;
        }

        // Check if current arg is a flag that takes a sensitive value next
        if SENSITIVE_FLAG_REGEX.is_match(arg) {
            sanitized.push(arg.clone());
            redact_next = true;
            continue;
        }

        // Check for curl -u user:password
        if arg == "-u" || arg == "--user" {
            sanitized.push(arg.clone());
            redact_next = true;
            continue;
        }

        // Sanitize string content (embedded URLs, auth headers, private keys)
        let cleaned = sanitize_string(arg);
        sanitized.push(cleaned);
    }

    sanitized
}

/// Sanitizes an individual string containing potential sensitive data.
pub fn sanitize_string(input: &str) -> String {
    let mut result = input.to_string();

    // Redact private key blocks
    if PRIVATE_KEY_BLOCK_REGEX.is_match(&result) {
        result = PRIVATE_KEY_BLOCK_REGEX
            .replace_all(&result, "[REDACTED_PRIVATE_KEY]")
            .into_owned();
    }

    // Redact credentials in URLs: https://user:pass@host -> https://user:[REDACTED]@host
    if URL_CREDENTIAL_REGEX.is_match(&result) {
        result = URL_CREDENTIAL_REGEX
            .replace_all(&result, "$1:[REDACTED]@")
            .into_owned();
    }

    // Redact HTTP Authorization headers
    if AUTH_HEADER_REGEX.is_match(&result) {
        result = AUTH_HEADER_REGEX
            .replace_all(&result, "${1}[REDACTED]")
            .into_owned();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitizes_flag_with_separate_value() {
        let args = vec![
            "curl".into(),
            "--token".into(),
            "secret_token_12345".into(),
            "https://example.com".into(),
        ];
        let cleaned = sanitize_arguments(&args);
        assert_eq!(
            cleaned,
            vec!["curl", "--token", "[REDACTED]", "https://example.com"]
        );
    }

    #[test]
    fn test_sanitizes_inline_flag() {
        let args = vec![
            "myapp".into(),
            "--password=mySuperSecretPass".into(),
            "--api-key=xyz987".into(),
        ];
        let cleaned = sanitize_arguments(&args);
        assert_eq!(
            cleaned,
            vec!["myapp", "--password=[REDACTED]", "--api-key=[REDACTED]"]
        );
    }

    #[test]
    fn test_sanitizes_url_credentials() {
        let args = vec![
            "git".into(),
            "clone".into(),
            "https://ankur:secretpass@github.com/repo/installmon.git".into(),
        ];
        let cleaned = sanitize_arguments(&args);
        assert_eq!(
            cleaned,
            vec![
                "git",
                "clone",
                "https://ankur:[REDACTED]@github.com/repo/installmon.git"
            ]
        );
    }

    #[test]
    fn test_sanitizes_curl_user_flag() {
        let args = vec![
            "curl".into(),
            "-u".into(),
            "admin:password123".into(),
            "https://api.test.org".into(),
        ];
        let cleaned = sanitize_arguments(&args);
        assert_eq!(
            cleaned,
            vec!["curl", "-u", "[REDACTED]", "https://api.test.org"]
        );
    }

    #[test]
    fn test_sanitizes_authorization_header() {
        let header = "Authorization: Bearer eyJhbGciOi...secret";
        let cleaned = sanitize_string(header);
        assert_eq!(cleaned, "Authorization: Bearer [REDACTED]");
    }
}
