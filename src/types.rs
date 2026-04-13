use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HttpMethod {
    Get, Post, Put, Delete, Head, Options, Patch,
}

impl FromStr for HttpMethod {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Self::Get), "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put), "DELETE" => Ok(Self::Delete),
            "HEAD" => Ok(Self::Head), "OPTIONS" => Ok(Self::Options),
            "PATCH" => Ok(Self::Patch),
            _ => Err(format!("Unsupported method: {}", s)),
        }
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Get => "GET", Self::Post => "POST", Self::Put => "PUT",
            Self::Delete => "DELETE", Self::Head => "HEAD", Self::Options => "OPTIONS",
            Self::Patch => "PATCH",
        }.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Protocol { Http, WebSocket, Tcp, Udp, Icmp }

/// Detect protocol from target string
pub fn detect_protocol(target: &str, force_ws: bool, force_icmp: bool) -> Protocol {
    if force_icmp { return Protocol::Icmp; }
    if force_ws { return Protocol::WebSocket; }
    let t = target.to_lowercase();
    if t.starts_with("ws://") || t.starts_with("wss://") { return Protocol::WebSocket; }
    if t.starts_with("http://") || t.starts_with("https://") { return Protocol::Http; }
    if t.starts_with("tcp://") { return Protocol::Tcp; }
    if t.starts_with("udp://") { return Protocol::Udp; }
    if t.starts_with("ping://") || t.starts_with("icmp://") { return Protocol::Icmp; }
    // Default: host:port → TCP, else → ICMP
    if target.contains(':') && !target.starts_with('[') { Protocol::Tcp } else { Protocol::Icmp }
}

/// Parse "host:port" with optional scheme prefix
pub fn parse_addr(target: &str, default_port: u16) -> Result<(String, u16), String> {
    let target = target
        .strip_prefix("tcp://").or_else(|| target.strip_prefix("udp://"))
        .or_else(|| target.strip_prefix("ping://")).or_else(|| target.strip_prefix("icmp://"))
        .unwrap_or(target);

    if let Some(addr) = target.strip_prefix('[') {
        if let Some(bracket_end) = addr.find(']') {
            let host = &addr[..bracket_end];
            let rest = &addr[bracket_end + 1..];
            if let Some(port_str) = rest.strip_prefix(':') {
                return port_str.parse::<u16>().map(|p| (host.to_string(), p)).map_err(|e| e.to_string());
            }
            return Ok((host.to_string(), default_port));
        }
    }

    if let Some(pos) = target.rfind(':') {
        let host = &target[..pos];
        if let Ok(port) = target[pos + 1..].parse::<u16>() {
            return Ok((host.to_string(), port));
        }
    }
    Ok((target.to_string(), default_port))
}

/// Parse "Key: Value" headers
pub fn parse_headers(headers: &[String]) -> Vec<(String, String)> {
    headers.iter().filter_map(|h| {
        let mut parts = h.splitn(2, ':');
        match (parts.next(), parts.next()) {
            (Some(k), Some(v)) => Some((k.trim().to_string(), v.trim().to_string())),
            _ => None,
        }
    }).collect()
}

/// Ensure URL has scheme
pub fn ensure_http_url(target: &str) -> String {
    if target.starts_with("http://") || target.starts_with("https://") {
        target.to_string()
    } else {
        format!("https://{}", target)
    }
}

/// Convert to ws:// URL
pub fn ensure_ws_url(target: &str) -> String {
    if target.starts_with("ws://") || target.starts_with("wss://") {
        target.to_string()
    } else if target.starts_with("http://") {
        target.replacen("http://", "ws://", 1)
    } else if target.starts_with("https://") {
        target.replacen("https://", "wss://", 1)
    } else {
        format!("wss://{}", target)
    }
}
