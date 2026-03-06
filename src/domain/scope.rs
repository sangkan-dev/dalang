//! Target scope derivation.
//!
//! Parses a raw target string (URL, domain, IP, CIDR) and derives the
//! authorized assessment scope so the AI agent can freely explore
//! subdomains, ports, and paths within that scope.

use url::Url;

/// The kind of target the user supplied.
#[derive(Debug, Clone, PartialEq)]
pub enum TargetType {
    /// A full URL like `https://example.com/app`
    Url,
    /// A bare domain like `example.com` or `app.example.com`
    Domain,
    /// A single IPv4/IPv6 address (no port)
    Ip,
    /// An IP with explicit port, e.g. `192.168.1.1:8080`
    IpPort,
    /// CIDR notation, e.g. `10.0.0.0/24`
    Cidr,
    /// Anything we cannot classify
    Unknown,
}

/// The derived scope for a given target.
#[derive(Debug, Clone)]
pub struct TargetScope {
    /// The raw string the user typed in
    pub raw_target: String,
    pub target_type: TargetType,
    /// Primary hostname or IP (without port / scheme / path)
    pub primary_host: String,
    /// Base/root domain for wildcard scope (e.g. `example.com`)
    pub base_domain: Option<String>,
    /// Explicit port if specified
    pub port: Option<u16>,
    /// Starting path for URL targets
    pub path: Option<String>,
    /// CIDR range string
    pub cidr: Option<String>,
}

impl TargetScope {
    /// Parse a raw target string and derive its scope.
    pub fn from_target(raw: &str) -> Self {
        let trimmed = raw.trim();

        // 1. Try to parse as a full URL (has scheme)
        if trimmed.contains("://")
            && let Ok(parsed) = Url::parse(trimmed) {
                let host = parsed.host_str().unwrap_or("").to_string();
                let port = parsed.port();
                let path = {
                    let p = parsed.path().to_string();
                    if p == "/" { None } else { Some(p) }
                };
                let is_ip = is_ip_address(&host);
                let base_domain = if is_ip { None } else { Some(extract_base_domain(&host)) };
                return Self {
                    raw_target: trimmed.to_string(),
                    target_type: TargetType::Url,
                    primary_host: host,
                    base_domain,
                    port,
                    path,
                    cidr: None,
                };
            }

        // 2. CIDR notation (contains '/' with digits on both sides)
        if trimmed.contains('/') {
            let parts: Vec<&str> = trimmed.splitn(2, '/').collect();
            if parts.len() == 2 && is_ip_address(parts[0]) && parts[1].parse::<u8>().is_ok() {
                return Self {
                    raw_target: trimmed.to_string(),
                    target_type: TargetType::Cidr,
                    primary_host: parts[0].to_string(),
                    base_domain: None,
                    port: None,
                    path: None,
                    cidr: Some(trimmed.to_string()),
                };
            }
        }

        // 3. IP with port (e.g. 192.168.1.1:8080)
        if let Some(colon_pos) = trimmed.rfind(':') {
            let maybe_host = &trimmed[..colon_pos];
            let maybe_port = &trimmed[colon_pos + 1..];
            if is_ip_address(maybe_host)
                && let Ok(port) = maybe_port.parse::<u16>() {
                    return Self {
                        raw_target: trimmed.to_string(),
                        target_type: TargetType::IpPort,
                        primary_host: maybe_host.to_string(),
                        base_domain: None,
                        port: Some(port),
                        path: None,
                        cidr: None,
                    };
                }
        }

        // 4. Bare IP address
        if is_ip_address(trimmed) {
            return Self {
                raw_target: trimmed.to_string(),
                target_type: TargetType::Ip,
                primary_host: trimmed.to_string(),
                base_domain: None,
                port: None,
                path: None,
                cidr: None,
            };
        }

        // 5. Bare domain (anything with a dot that isn't an IP)
        if trimmed.contains('.') && !trimmed.contains(' ') {
            // Could have an optional port: host:port
            if let Some(colon_pos) = trimmed.rfind(':') {
                let maybe_host = &trimmed[..colon_pos];
                let maybe_port = &trimmed[colon_pos + 1..];
                if let Ok(port) = maybe_port.parse::<u16>() {
                    let base = extract_base_domain(maybe_host);
                    return Self {
                        raw_target: trimmed.to_string(),
                        target_type: TargetType::Domain,
                        primary_host: maybe_host.to_string(),
                        base_domain: Some(base),
                        port: Some(port),
                        path: None,
                        cidr: None,
                    };
                }
            }
            let base = extract_base_domain(trimmed);
            return Self {
                raw_target: trimmed.to_string(),
                target_type: TargetType::Domain,
                primary_host: trimmed.to_string(),
                base_domain: Some(base),
                port: None,
                path: None,
                cidr: None,
            };
        }

        // Fallback
        Self {
            raw_target: trimmed.to_string(),
            target_type: TargetType::Unknown,
            primary_host: trimmed.to_string(),
            base_domain: None,
            port: None,
            path: None,
            cidr: None,
        }
    }

    /// Generate the scope section to inject into system prompts.
    pub fn to_prompt_section(&self) -> String {
        let mut s = String::from("### AUTHORIZED SCOPE\n");

        match self.target_type {
            TargetType::Url => {
                s.push_str(&format!("Primary target: `{}`\n", self.raw_target));
                if let Some(ref base) = self.base_domain {
                    s.push_str(&format!(
                        "Authorized scope: `*.{}` — all subdomains, all TCP/UDP ports, all URL paths and directories.\n",
                        base
                    ));
                } else {
                    // IP-based URL
                    s.push_str(&format!(
                        "Authorized scope: `{}` — all TCP/UDP ports, all URL paths and services.\n",
                        self.primary_host
                    ));
                }
                if let Some(ref path) = self.path {
                    s.push_str(&format!(
                        "Starting path: `{}` — begin here, then explore other paths, directories, and endpoints.\n",
                        path
                    ));
                }
            }
            TargetType::Domain => {
                s.push_str(&format!("Primary target: `{}`\n", self.primary_host));
                if let Some(ref base) = self.base_domain {
                    s.push_str(&format!(
                        "Authorized scope: `*.{}` — all subdomains, all TCP/UDP ports, all URL paths.\n",
                        base
                    ));
                }
            }
            TargetType::Ip => {
                s.push_str(&format!(
                    "Primary target: `{}`\n\
                    Authorized scope: all TCP/UDP ports on `{}`, all paths and services, any virtual hosts.\n",
                    self.primary_host, self.primary_host
                ));
            }
            TargetType::IpPort => {
                s.push_str(&format!(
                    "Primary target: `{}:{}`\n\
                    Authorized scope: all TCP/UDP ports on `{}`, all paths and services. \
                    Port {} is the starting point but explore all ports.\n",
                    self.primary_host, self.port.unwrap_or(0),
                    self.primary_host, self.port.unwrap_or(0)
                ));
            }
            TargetType::Cidr => {
                s.push_str(&format!(
                    "Primary target: `{}`\n\
                    Authorized scope: ALL hosts in the `{}` range, all TCP/UDP ports on each host.\n\
                    Strategy: Start with host discovery, then enumerate services on live hosts.\n",
                    self.raw_target,
                    self.cidr.as_deref().unwrap_or(&self.raw_target)
                ));
            }
            TargetType::Unknown => {
                s.push_str(&format!(
                    "Primary target: `{}`\n\
                    Authorized scope: the target and any directly related hosts/services.\n",
                    self.raw_target
                ));
            }
        }

        s.push_str(
            "\n### SCOPE RULES\n\
            - You are ENCOURAGED to explore different paths, directories, subdomains, and ports within the authorized scope.\n\
            - Start from the primary target, then expand your reconnaissance to discover hidden endpoints, \
            admin panels, API routes, backup files, alternate ports, and related subdomains.\n\
            - Use directory/path discovery tools (e.g., ffuf, dirsearch) and subdomain enumeration when appropriate.\n\
            - Do NOT scan hosts, domains, or IP ranges that fall outside the authorized scope.\n\
            - If you discover a new subdomain or service within scope, you MAY pivot to investigate it.\n"
        );

        s
    }
}

/// Simple check: does the string look like an IPv4 or IPv6 address?
fn is_ip_address(s: &str) -> bool {
    // IPv4
    if s.split('.').count() == 4 && s.split('.').all(|p| p.parse::<u8>().is_ok()) {
        return true;
    }
    // IPv6 (contains at least two colons)
    if s.contains(':') && s.chars().filter(|&c| c == ':').count() >= 2 {
        return true;
    }
    false
}

/// Extract the base/root domain from a hostname.
/// e.g. `app.staging.example.com` → `example.com`
///      `admin.example.co.id` → `example.co.id`
fn extract_base_domain(host: &str) -> String {
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() <= 2 {
        return host.to_string();
    }

    // Common two-part TLDs
    let two_part_tlds = [
        "co.uk", "co.id", "co.jp", "co.kr", "co.nz", "co.za", "co.in",
        "com.au", "com.br", "com.cn", "com.mx", "com.sg", "com.tw",
        "org.uk", "org.au", "net.au", "net.uk",
        "ac.uk", "ac.id", "ac.jp",
        "go.id", "go.jp", "go.kr",
        "or.id", "or.jp", "or.kr",
    ];

    let last_two = format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1]);
    if two_part_tlds.contains(&last_two.as_str()) && parts.len() >= 3 {
        // Take last 3 parts: e.g. example.co.id
        return parts[parts.len() - 3..].join(".");
    }

    // Default: last 2 parts
    parts[parts.len() - 2..].join(".")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_with_path() {
        let scope = TargetScope::from_target("https://app.example.com/admin");
        assert_eq!(scope.target_type, TargetType::Url);
        assert_eq!(scope.primary_host, "app.example.com");
        assert_eq!(scope.base_domain.as_deref(), Some("example.com"));
        assert_eq!(scope.path.as_deref(), Some("/admin"));
    }

    #[test]
    fn test_url_with_port() {
        let scope = TargetScope::from_target("http://10.0.0.1:8080/api");
        assert_eq!(scope.target_type, TargetType::Url);
        assert_eq!(scope.primary_host, "10.0.0.1");
        assert_eq!(scope.port, Some(8080));
        assert_eq!(scope.base_domain, None); // IP, no domain
    }

    #[test]
    fn test_bare_domain() {
        let scope = TargetScope::from_target("staging.example.com");
        assert_eq!(scope.target_type, TargetType::Domain);
        assert_eq!(scope.base_domain.as_deref(), Some("example.com"));
    }

    #[test]
    fn test_bare_ip() {
        let scope = TargetScope::from_target("192.168.1.1");
        assert_eq!(scope.target_type, TargetType::Ip);
        assert_eq!(scope.primary_host, "192.168.1.1");
    }

    #[test]
    fn test_ip_with_port() {
        let scope = TargetScope::from_target("10.0.0.5:443");
        assert_eq!(scope.target_type, TargetType::IpPort);
        assert_eq!(scope.port, Some(443));
    }

    #[test]
    fn test_cidr() {
        let scope = TargetScope::from_target("10.0.0.0/24");
        assert_eq!(scope.target_type, TargetType::Cidr);
        assert_eq!(scope.cidr.as_deref(), Some("10.0.0.0/24"));
    }

    #[test]
    fn test_two_part_tld() {
        let scope = TargetScope::from_target("admin.example.co.id");
        assert_eq!(scope.target_type, TargetType::Domain);
        assert_eq!(scope.base_domain.as_deref(), Some("example.co.id"));
    }

    #[test]
    fn test_prompt_section_url() {
        let scope = TargetScope::from_target("https://app.example.com/dashboard");
        let prompt = scope.to_prompt_section();
        assert!(prompt.contains("*.example.com"));
        assert!(prompt.contains("/dashboard"));
        assert!(prompt.contains("SCOPE RULES"));
        assert!(prompt.contains("ENCOURAGED"));
    }

    #[test]
    fn test_prompt_section_cidr() {
        let scope = TargetScope::from_target("192.168.0.0/16");
        let prompt = scope.to_prompt_section();
        assert!(prompt.contains("192.168.0.0/16"));
        assert!(prompt.contains("ALL hosts"));
        assert!(prompt.contains("host discovery"));
    }
}
