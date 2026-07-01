use std::collections::hash_map::DefaultHasher;
use std::future::Future;
use std::hash::{Hash, Hasher};
use std::time::Duration;

use grok_voice_core::{emit_retry, retry_context_label, AppError, RetryNotification};

/// Total attempts = `MAX_RETRIES + 1`.
pub const MAX_RETRIES: u32 = 4;

pub fn is_rate_limit_message(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("rate limit")
        || lower.contains("too many requests")
        || lower.contains("429")
}

pub fn is_retryable(err: &AppError) -> bool {
    match err {
        AppError::RateLimited { .. } => true,
        AppError::ProviderUnavailable(message) => {
            let lower = message.to_ascii_lowercase();
            is_rate_limit_message(message)
                || lower.contains("timeout")
                || lower.contains("timed out")
                || lower.contains("connection")
                || lower.contains("temporarily unavailable")
                || lower.contains("http 5")
                || lower.contains("http 502")
                || lower.contains("http 503")
                || lower.contains("http 504")
                || lower.contains("websocket")
        }
        _ => false,
    }
}

pub fn retry_after_from_error(err: &AppError) -> Option<u64> {
    match err {
        AppError::RateLimited { retry_after_secs } => *retry_after_secs,
        _ => None,
    }
}

pub fn backoff_delay(attempt: u32, retry_after_secs: Option<u64>) -> Duration {
    if let Some(secs) = retry_after_secs {
        return Duration::from_secs(secs.max(1));
    }
    Duration::from_secs(2u64.saturating_pow(attempt.max(1).min(6)))
}

fn jitter_seed(context: Option<&str>, attempt: u32) -> u64 {
    let mut hasher = DefaultHasher::new();
    context.hash(&mut hasher);
    attempt.hash(&mut hasher);
    hasher.finish()
}

/// Exponential backoff with per-context jitter to desynchronize parallel batch retries.
pub fn backoff_delay_with_jitter(
    attempt: u32,
    retry_after_secs: Option<u64>,
    context: Option<&str>,
) -> Duration {
    let base = backoff_delay(attempt, retry_after_secs);
    let seed = jitter_seed(context, attempt);

    if retry_after_secs.is_some() {
        return base + Duration::from_millis(seed % 1000);
    }

    let jitter_pct = seed % 40;
    let extra_ms = (base.as_millis() as u64 * jitter_pct / 100).max(100);
    base + Duration::from_millis(extra_ms)
}

pub async fn with_retry_category<F, Fut, T>(
    mut operation: F,
    category: &'static str,
) -> Result<T, AppError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, AppError>>,
{
    let mut attempt = 0u32;
    loop {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(err) if is_retryable(&err) && attempt < MAX_RETRIES => {
                attempt += 1;
                let retry_after = retry_after_from_error(&err);
                let context = retry_context_label();
                let delay =
                    backoff_delay_with_jitter(attempt, retry_after, context.as_deref());
                tracing::warn!(
                    target: "xai_retry",
                    attempt,
                    max = MAX_RETRIES,
                    delay_secs = delay.as_secs(),
                    category,
                    error = %err,
                    "retrying xAI request"
                );
                emit_retry(RetryNotification {
                    attempt,
                    max_retries: MAX_RETRIES,
                    delay_secs: delay.as_secs(),
                    error: err.to_string(),
                    category,
                    context,
                });
                tokio::time::sleep(delay).await;
            }
            Err(err) => return Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_rate_limit_messages() {
        assert!(is_rate_limit_message("HTTP 429: Too Many Requests"));
        assert!(is_rate_limit_message("rate limit exceeded"));
        assert!(!is_rate_limit_message("invalid voice id"));
    }

    #[test]
    fn retryable_includes_transient_provider_errors() {
        assert!(is_retryable(&AppError::RateLimited {
            retry_after_secs: None,
        }));
        assert!(is_retryable(&AppError::ProviderUnavailable(
            "WebSocket connect timeout".into()
        )));
        assert!(!is_retryable(&AppError::AuthFailed));
        assert!(!is_retryable(&AppError::TextTooLong {
            chars: 20_000,
            max: 15_000,
        }));
    }

    #[test]
    fn honors_retry_after_header() {
        let delay = backoff_delay(1, Some(30));
        assert_eq!(delay, Duration::from_secs(30));
    }

    #[test]
    fn exponential_backoff_without_retry_after() {
        assert_eq!(backoff_delay(1, None), Duration::from_secs(2));
        assert_eq!(backoff_delay(2, None), Duration::from_secs(4));
        assert_eq!(backoff_delay(3, None), Duration::from_secs(8));
    }

    #[test]
    fn jitter_adds_delay_without_retry_after() {
        let base = backoff_delay(1, None);
        let with_jitter = backoff_delay_with_jitter(1, None, Some("seg-a"));
        assert!(with_jitter >= base);
        assert!(with_jitter <= base + Duration::from_millis(base.as_millis() as u64 * 40 / 100));
    }

    #[test]
    fn jitter_differs_by_context() {
        let a = backoff_delay_with_jitter(2, None, Some("segment-1"));
        let b = backoff_delay_with_jitter(2, None, Some("segment-2"));
        assert_ne!(a, b);
    }
}