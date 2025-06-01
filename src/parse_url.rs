use url::Url;

fn try_parse_url(input: &str) -> Option<Url> {
    // Early validation: must contain a dot for a valid hostname
    if !input.contains('.') {
        return None;
    }
    
    // If input already contains a scheme (http:// or https://), parse directly
    if input.starts_with("http://") || input.starts_with("https://") {
        return Url::parse(input).ok();
    }
    
    // Try with http:// prefix - use a capacity hint to reduce allocations
    let mut url_with_scheme = String::with_capacity(7 + input.len()); // "http://" = 7 chars
    url_with_scheme.push_str("http://");
    url_with_scheme.push_str(input);
    
    Url::parse(&url_with_scheme).ok()
}

pub fn extract_hostname(input: &str) -> String {
    // Early validation: empty or no dot means invalid
    if input.is_empty() || !input.contains('.') {
        return String::new();
    }
    
    let parsed_url = try_parse_url(input);
    
    match parsed_url {
        Some(url) => {
            let host_str = url.host_str().unwrap_or_default();
            if let Some(port) = url.port() {
                // Use format! only when we actually have a port
                format!("{}:{}", host_str, port)
            } else {
                // Avoid .to_string() allocation by using String::from
                String::from(host_str)
            }
        }
        None => String::new(),
    }
}

pub fn read_url(input: &str) -> (bool, String, String) {
    let url = extract_hostname(input);
    if url.is_empty() {
        return (false, String::new(), String::new());
    }
    
    // Pre-allocate with capacity hints to reduce allocations
    let mut http_url = String::with_capacity(7 + url.len()); // "http://" = 7 chars
    http_url.push_str("http://");
    http_url.push_str(&url);
    
    let mut https_url = String::with_capacity(8 + url.len()); // "https://" = 8 chars
    https_url.push_str("https://");
    https_url.push_str(&url);
    
    (true, http_url, https_url)
}

#[cfg(test)]
mod tests {
    use super::extract_hostname;

    #[test]
    fn test_empty_string() {
        assert_eq!(extract_hostname(""), "");
    }

    #[test]
    fn test_invalid_characters() {
        assert_eq!(extract_hostname("aaa$@%"), "");
    }

    #[test]
    fn test_invalid_url_with_tld() {
        assert_eq!(extract_hostname("aa@%$^a.com/page"), "");
    }

    #[test]
    fn test_without_tld() {
        assert_eq!(extract_hostname("aaa"), "");
    }

    #[test]
    fn test_with_valid_tld() {
        assert_eq!(extract_hostname("aaa.com"), "aaa.com");
    }

    #[test]
    fn test_with_path() {
        assert_eq!(extract_hostname("aaa.com/page"), "aaa.com");
    }

    #[test]
    fn test_with_http() {
        assert_eq!(extract_hostname("http://aaa.com/"), "aaa.com");
    }

    #[test]
    fn test_with_http_and_path() {
        assert_eq!(extract_hostname("http://aaa.com/page"), "aaa.com");
    }

    #[test]
    fn test_with_port() {
        assert_eq!(extract_hostname("aaa.com:8080"), "aaa.com:8080");
    }

    #[test]
    fn test_with_ip() {
        assert_eq!(extract_hostname("192.168.1.1"), "192.168.1.1");
    }

    #[test]
    fn test_with_https() {
        assert_eq!(extract_hostname("https://aaa.com/"), "aaa.com");
    }

    #[test]
    fn test_with_https_and_path() {
        assert_eq!(extract_hostname("https://aaa.com/page"), "aaa.com");
    }
}
