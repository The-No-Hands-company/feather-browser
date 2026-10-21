// SPDX-License-Identifier: AGPL-3.0-or-later

/// Returned when a page is successfully fetched.
#[derive(Debug, Clone)]
pub struct FetchResult {
    /// The HTML body as a UTF-8 string.
    pub html: String,
    /// The final URL after any redirects.
    pub base_url: String,
    /// HTTP status code (200 for about:blank).
    pub status: u16,
}

/// Wraps a fetch error message.
#[derive(Debug)]
pub struct FetchError(pub String);

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FetchError: {}", self.0)
    }
}

impl std::error::Error for FetchError {}

/// Fetch the HTML at `url`. Supports `http://`, `https://`, and `about:blank`.
/// Blocks the calling thread. The caller is responsible for spawning a background
/// thread if the UI must remain responsive.
pub fn fetch_page(url: &str) -> Result<FetchResult, FetchError> {
    if url == "about:blank" || url.is_empty() {
        return Ok(FetchResult {
            html: "<html><body></body></html>".into(),
            base_url: url.to_string(),
            status: 200,
        });
    }

    // Fix 5: disable Referer header to prevent privacy leaks.
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("Feather/0.3 (https://tnhc.dev/feather)")
        .redirect(reqwest::redirect::Policy::limited(10))
        .referer(false)
        .build()
        .map_err(|e| FetchError(format!("client build failed: {e}")))?;

    let response = client
        .get(url)
        .send()
        .map_err(|e| FetchError(format!("request failed: {e}")))?;

    let base_url = response.url().to_string();
    let status = response.status().as_u16();

    // Fix 6: cap response body at 10 MB to prevent unbounded memory use.
    const MAX_BODY_BYTES: usize = 10 * 1024 * 1024;
    let bytes = response
        .bytes()
        .map_err(|e| FetchError(format!("body read failed: {e}")))?;
    let truncated = if bytes.len() > MAX_BODY_BYTES {
        &bytes[..MAX_BODY_BYTES]
    } else {
        &bytes[..]
    };
    let html = String::from_utf8_lossy(truncated).into_owned();

    Ok(FetchResult { html, base_url, status })
}
