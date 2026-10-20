// SPDX-License-Identifier: AGPL-3.0-or-later

const TRACKING_PARAMS: &[&str] = &[
    "utm_source", "utm_medium", "utm_campaign", "utm_term", "utm_content",
    "utm_id", "utm_reader", "utm_name", "utm_publ", "utm_swu",
    "fbclid", "gclid", "msclkid", "dclid", "twclid",
    "mc_cid", "mc_eid",
    "ref", "_ga", "_gl",
    "igshid",
];

pub struct Privacy;

impl Privacy {
    /// Strips known tracking query parameters from a URL.
    /// Returns the URL unchanged if it cannot be parsed.
    pub fn strip_tracking_params(url: &str) -> String {
        // Split on fragment first so we can restore it
        let (url_no_frag, fragment) = match url.split_once('#') {
            Some((u, f)) => (u, Some(f)),
            None => (url, None),
        };

        let (base, query_str) = match url_no_frag.split_once('?') {
            Some((b, q)) => (b, Some(q)),
            None => {
                // No query string — return original (with fragment if any)
                return match fragment {
                    Some(f) => format!("{}#{}", url_no_frag, f),
                    None => url.to_string(),
                };
            }
        };

        let clean_params: Vec<&str> = query_str
            .unwrap_or("")
            .split('&')
            .filter(|param| {
                let key = param.split('=').next().unwrap_or("");
                !TRACKING_PARAMS.iter().any(|&t| key.eq_ignore_ascii_case(t))
            })
            .collect();

        let result = if clean_params.is_empty() {
            base.to_string()
        } else {
            format!("{}?{}", base, clean_params.join("&"))
        };

        match fragment {
            Some(f) => format!("{}#{}", result, f),
            None => result,
        }
    }
}
