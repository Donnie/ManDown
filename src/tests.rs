use crate::data::extract_hostname;


#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_with_https() {
        assert_eq!(extract_hostname("https://aaa.com/"), "aaa.com");
    }

    #[test]
    fn test_with_https_and_path() {
        assert_eq!(extract_hostname("https://aaa.com/page"), "aaa.com");
    }
}
