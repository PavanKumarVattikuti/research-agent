use std::net::IpAddr;
use url::Url;

/// Blocked hostnames for SSRF protection
const BLOCKED_HOSTS: &[&str] = &[
    "localhost",
    "127.0.0.1",
    "::1",
    "0.0.0.0",
    "host.docker.internal",
    "metadata.google.internal",
    "169.254.169.254", // AWS/GCP/Azure instance metadata
];

/// Validates that a target URL is an external, safe http/https URL.
pub fn validate_external_url(target: &str) -> Result<Url, String> {
    let parsed = Url::parse(target.trim()).map_err(|e| format!("Invalid URL: {}", e))?;

    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(format!("Only http/https URLs are allowed, got: {}", scheme));
    }

    let host_str = parsed
        .host_str()
        .ok_or_else(|| "URL has empty hostname".to_string())?
        .trim_matches(|c| c == '[' || c == ']');

    // Check explicit blocked hostnames
    let lower_host = host_str.to_lowercase();
    for blocked in BLOCKED_HOSTS {
        if lower_host == *blocked || lower_host.ends_with(&format!(".{}", blocked)) {
            return Err("Access to internal/private hostname is blocked for security".into());
        }
    }

    // Check IP literals
    if let Ok(ip) = host_str.parse::<IpAddr>() {
        if is_private_or_restricted(ip) {
            return Err("Access to private/local IP addresses is blocked for security".into());
        }
    }

    Ok(parsed)
}

/// Returns true if an IP address is private, loopback, multicast, or link-local.
fn is_private_or_restricted(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_multicast()
                || v4.is_broadcast()
                || v4.is_documentation()
                || v4.octets()[0] == 0 // 0.0.0.0/8
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_multicast()
                || (v6.segments()[0] & 0xfe00) == 0xfc00 // Unique Local Address fc00::/7
                || (v6.segments()[0] & 0xffc0) == 0xfe80 // Link-Local Unicast fe80::/10
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocks_localhost() {
        assert!(validate_external_url("http://localhost:8080").is_err());
        assert!(validate_external_url("http://127.0.0.1/api").is_err());
        assert!(validate_external_url("http://[::1]:7860").is_err());
        assert!(validate_external_url("http://192.168.1.1").is_err());
        assert!(validate_external_url("http://10.0.0.5").is_err());
        assert!(validate_external_url("file:///etc/passwd").is_err());
    }

    #[test]
    fn test_allows_public_urls() {
        assert!(validate_external_url("https://example.com").is_ok());
        assert!(validate_external_url("https://huggingface.co/models").is_ok());
        assert!(validate_external_url("https://duckduckgo.com").is_ok());
    }
}
