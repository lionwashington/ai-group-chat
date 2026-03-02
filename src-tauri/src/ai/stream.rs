use futures::StreamExt;
use reqwest::Response;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct StreamChunk {
    choices: Option<Vec<StreamChoice>>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: Option<StreamDelta>,
}

#[derive(Debug, Deserialize)]
struct StreamDelta {
    content: Option<String>,
}

/// Process an SSE stream from an OpenAI-compatible API response.
///
/// Calls `on_delta` for each content delta received. Returns the full
/// accumulated content string when the stream is complete.
///
/// Handles:
/// - Lines starting with "data: "
/// - "[DONE]" terminator
/// - Empty lines and comments (lines starting with ':')
/// - Partial chunks across buffer boundaries
/// - Malformed JSON (skip gracefully)
pub async fn process_stream<F>(response: Response, mut on_delta: F) -> Result<String, String>
where
    F: FnMut(&str),
{
    let mut full_content = String::new();
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Stream error: {}", e))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim().to_string();
            buffer = buffer[pos + 1..].to_string();

            if line.is_empty() || line.starts_with(':') {
                continue;
            }

            if let Some(data) = line.strip_prefix("data: ") {
                if data.trim() == "[DONE]" {
                    return Ok(full_content);
                }
                if let Ok(chunk) = serde_json::from_str::<StreamChunk>(data) {
                    if let Some(choices) = chunk.choices {
                        for choice in choices {
                            if let Some(delta) = choice.delta {
                                if let Some(content) = delta.content {
                                    full_content.push_str(&content);
                                    on_delta(&content);
                                }
                            }
                        }
                    }
                }
                // Malformed JSON is silently skipped
            }
        }
    }
    Ok(full_content)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a mock Response from raw SSE text and parse it.
    async fn parse_sse_string(sse_data: &str) -> (String, Vec<String>) {
        let body = reqwest::Body::from(sse_data.to_string());
        let response = http::Response::builder()
            .status(200)
            .body(body)
            .unwrap();
        let response = reqwest::Response::from(response);

        let mut deltas = Vec::new();
        let full = process_stream(response, |d| deltas.push(d.to_string()))
            .await
            .unwrap();
        (full, deltas)
    }

    /// UT-AI-07: Parse SSE: single delta -> correct text
    #[tokio::test]
    async fn test_sse_single_delta() {
        let sse = "data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\ndata: [DONE]\n\n";
        let (full, deltas) = parse_sse_string(sse).await;
        assert_eq!(full, "Hello");
        assert_eq!(deltas, vec!["Hello"]);
    }

    /// UT-AI-08: Parse SSE: multiple deltas -> accumulated
    #[tokio::test]
    async fn test_sse_multiple_deltas() {
        let sse = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\" world\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"!\"}}]}\n\n",
            "data: [DONE]\n\n",
        );
        let (full, deltas) = parse_sse_string(sse).await;
        assert_eq!(full, "Hello world!");
        assert_eq!(deltas, vec!["Hello", " world", "!"]);
    }

    /// UT-AI-09: Parse SSE: [DONE] -> returns full content
    #[tokio::test]
    async fn test_sse_done_returns_full_content() {
        let sse = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"A\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"B\"}}]}\n\n",
            "data: [DONE]\n\n",
        );
        let (full, _deltas) = parse_sse_string(sse).await;
        assert_eq!(full, "AB");
    }

    /// UT-AI-10: Parse SSE: empty lines/comments -> skipped
    #[tokio::test]
    async fn test_sse_empty_lines_and_comments_skipped() {
        let sse = concat!(
            ": this is a comment\n",
            "\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"Hi\"}}]}\n",
            "\n",
            ": another comment\n",
            "\n",
            "data: [DONE]\n\n",
        );
        let (full, deltas) = parse_sse_string(sse).await;
        assert_eq!(full, "Hi");
        assert_eq!(deltas, vec!["Hi"]);
    }

    /// UT-AI-11: Parse SSE: malformed JSON -> skipped gracefully
    #[tokio::test]
    async fn test_sse_malformed_json_skipped() {
        let sse = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"Good\"}}]}\n\n",
            "data: {malformed json\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\" day\"}}]}\n\n",
            "data: [DONE]\n\n",
        );
        let (full, deltas) = parse_sse_string(sse).await;
        assert_eq!(full, "Good day");
        assert_eq!(deltas, vec!["Good", " day"]);
    }

    /// UT-AI-12: Parse SSE: chunked partial lines -> buffer handles correctly
    #[tokio::test]
    async fn test_sse_chunked_partial_lines() {
        // Simulate data arriving in two chunks that split in the middle of a line
        let chunk1 = "data: {\"choices\":[{\"delt";
        let chunk2 = "a\":{\"content\":\"Hi\"}}]}\n\ndata: [DONE]\n\n";

        // We need to build a response that sends data in two chunks.
        // We'll concatenate and use the standard helper since reqwest::Body::from(String)
        // delivers it as one chunk. To truly test chunking, we use a stream body.
        use futures::stream;
        let chunks: Vec<Result<bytes::Bytes, std::io::Error>> = vec![
            Ok(bytes::Bytes::from(chunk1.to_string())),
            Ok(bytes::Bytes::from(chunk2.to_string())),
        ];
        let body = reqwest::Body::wrap_stream(stream::iter(chunks));
        let response = http::Response::builder()
            .status(200)
            .body(body)
            .unwrap();
        let response = reqwest::Response::from(response);

        let mut deltas = Vec::new();
        let full = process_stream(response, |d| deltas.push(d.to_string()))
            .await
            .unwrap();
        assert_eq!(full, "Hi");
        assert_eq!(deltas, vec!["Hi"]);
    }

    /// UT-AI-12b: SSE stream with no [DONE] marker (stream just ends) -> returns accumulated content
    #[tokio::test]
    async fn test_sse_no_done_marker() {
        let sse = "data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\n";
        let (full, deltas) = parse_sse_string(sse).await;
        assert_eq!(full, "Hello");
        assert_eq!(deltas, vec!["Hello"]);
    }

    /// UT-AI-11b: Parse SSE: delta with no content field -> skipped
    #[tokio::test]
    async fn test_sse_delta_no_content() {
        let sse = concat!(
            "data: {\"choices\":[{\"delta\":{\"role\":\"assistant\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"Hi\"}}]}\n\n",
            "data: [DONE]\n\n",
        );
        let (full, deltas) = parse_sse_string(sse).await;
        assert_eq!(full, "Hi");
        assert_eq!(deltas, vec!["Hi"]);
    }
}
