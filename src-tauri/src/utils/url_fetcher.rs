use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;

use crate::models::Message;

static URL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"https?://[^\s<>\[\]"'{}]+"#).unwrap()
});

/// Extract unique URLs from a text string.
/// Strips common trailing punctuation that isn't part of the URL.
pub fn extract_urls(text: &str) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut urls = Vec::new();

    for m in URL_RE.find_iter(text) {
        let mut url = m.as_str().to_string();
        // Strip trailing punctuation that's unlikely to be part of a URL
        while url.ends_with(['.', ',', ';', ':', '!', '?', ')']) {
            // Keep ')' if there's a matching '(' in the URL (e.g. Wikipedia links)
            if url.ends_with(')') {
                let open = url.chars().filter(|&c| c == '(').count();
                let close = url.chars().filter(|&c| c == ')').count();
                if open >= close {
                    break;
                }
            }
            url.pop();
        }
        if seen.insert(url.clone()) {
            urls.push(url);
        }
    }
    urls
}

/// Fetch content from a single URL and convert HTML to readable text.
/// Returns the text content truncated to 5000 characters.
pub async fn fetch_url_content(url: &str) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let response = client
        .get(url)
        .header("User-Agent", "AI-Group-Chat/0.1 (URL content reader)")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch {}: {}", url, e))?;

    if !response.status().is_success() {
        return Err(format!(
            "HTTP {} fetching {}",
            response.status(),
            url
        ));
    }

    // Limit body to 2MB
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response body from {}: {}", url, e))?;

    if bytes.len() > 2 * 1024 * 1024 {
        return Err(format!("Response too large from {} ({} bytes)", url, bytes.len()));
    }

    let text = html2text::from_read(&bytes[..], 80)
        .map_err(|e| format!("Failed to parse HTML from {}: {}", url, e))?;

    // Truncate to 5000 chars
    if text.len() > 5000 {
        let truncated: String = text.chars().take(5000).collect();
        Ok(format!("{}...[truncated]", truncated))
    } else {
        Ok(text)
    }
}

/// Extract URLs from all human messages and fetch their content concurrently.
/// Returns a cache mapping URL -> fetched text content.
/// Failed fetches are silently skipped.
pub async fn fetch_all_urls(messages: &[Message]) -> HashMap<String, String> {
    let mut all_urls = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for msg in messages {
        if msg.sender_type == "human" {
            for url in extract_urls(&msg.content) {
                if seen.insert(url.clone()) {
                    all_urls.push(url);
                }
            }
        }
    }

    if all_urls.is_empty() {
        return HashMap::new();
    }

    let fetches: Vec<_> = all_urls
        .iter()
        .map(|url| async move {
            let result = fetch_url_content(url).await;
            (url.clone(), result)
        })
        .collect();

    let results = futures::future::join_all(fetches).await;

    let mut cache = HashMap::new();
    for (url, result) in results {
        if let Ok(content) = result {
            cache.insert(url, content);
        }
    }
    cache
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_single_url() {
        let urls = extract_urls("Check out https://example.com please");
        assert_eq!(urls, vec!["https://example.com"]);
    }

    #[test]
    fn test_extract_multiple_urls() {
        let urls = extract_urls("See https://a.com and http://b.com/path");
        assert_eq!(urls, vec!["https://a.com", "http://b.com/path"]);
    }

    #[test]
    fn test_extract_urls_deduplication() {
        let urls = extract_urls("https://example.com and again https://example.com");
        assert_eq!(urls, vec!["https://example.com"]);
    }

    #[test]
    fn test_extract_no_urls() {
        let urls = extract_urls("No links here at all");
        assert!(urls.is_empty());
    }

    #[test]
    fn test_extract_urls_trailing_punctuation() {
        let urls = extract_urls("Visit https://example.com. Or https://other.com!");
        assert_eq!(urls, vec!["https://example.com", "https://other.com"]);
    }

    #[test]
    fn test_extract_urls_with_query_params() {
        let urls = extract_urls("See https://example.com/page?q=rust&lang=en#section");
        assert_eq!(urls, vec!["https://example.com/page?q=rust&lang=en#section"]);
    }

    #[tokio::test]
    async fn test_fetch_url_content_real() {
        let result = fetch_url_content("https://example.com").await;
        assert!(result.is_ok(), "Failed to fetch example.com: {:?}", result);
        let content = result.unwrap();
        assert!(
            content.contains("Example Domain"),
            "Content should contain 'Example Domain', got: {}",
            &content[..content.len().min(200)]
        );
    }

    #[tokio::test]
    async fn test_fetch_all_urls_real() {
        let messages = vec![Message {
            id: "msg-1".to_string(),
            topic_id: "topic-1".to_string(),
            sender_type: "human".to_string(),
            sender_bot_id: None,
            content: "What does https://example.com say?".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            attachments: Vec::new(),
        }];
        let cache = fetch_all_urls(&messages).await;
        assert!(cache.contains_key("https://example.com"), "Cache should contain example.com");
        assert!(cache["https://example.com"].contains("Example Domain"));
    }

    #[tokio::test]
    async fn test_fetch_url_content_invalid_url() {
        let result = fetch_url_content("https://this-domain-does-not-exist-abc123.com").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_urls_preserves_balanced_parens() {
        let urls = extract_urls("https://en.wikipedia.org/wiki/Rust_(programming_language)");
        assert_eq!(
            urls,
            vec!["https://en.wikipedia.org/wiki/Rust_(programming_language)"]
        );
    }
}
