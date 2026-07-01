use std::future::Future;
use std::sync::{OnceLock, RwLock};

/// Progress update for long-running story conversion (API / JSON repair).
#[derive(Debug, Clone)]
pub struct StoryConvertProgress {
    pub attempt: u32,
    pub phase: &'static str,
    pub message: Option<String>,
}

/// Optional hook installed by the desktop shell to surface API retries in logs/UI.
#[derive(Debug, Clone)]
pub struct RetryNotification {
    pub attempt: u32,
    pub max_retries: u32,
    pub delay_secs: u64,
    pub error: String,
    pub category: &'static str,
    pub context: Option<String>,
}

type RetryHook = Box<dyn Fn(RetryNotification) + Send + Sync>;
type StoryConvertHook = Box<dyn Fn(StoryConvertProgress) + Send + Sync>;

static HOOK: OnceLock<RwLock<Option<RetryHook>>> = OnceLock::new();
static STORY_HOOK: OnceLock<RwLock<Option<StoryConvertHook>>> = OnceLock::new();

tokio::task_local! {
    static RETRY_CONTEXT: Option<String>;
}

pub fn install_retry_hook(hook: RetryHook) {
    let slot = HOOK.get_or_init(|| RwLock::new(None));
    if let Ok(mut guard) = slot.write() {
        *guard = Some(hook);
    }
}

pub fn install_story_convert_hook(hook: StoryConvertHook) {
    let slot = STORY_HOOK.get_or_init(|| RwLock::new(None));
    if let Ok(mut guard) = slot.write() {
        *guard = Some(hook);
    }
}

pub fn emit_story_convert(progress: StoryConvertProgress) {
    let Some(slot) = STORY_HOOK.get() else {
        return;
    };
    let Ok(guard) = slot.read() else {
        return;
    };
    if let Some(hook) = guard.as_ref() {
        hook(progress);
    }
}

/// Suggest a lower concurrency when batch TTS hit rate limits.
pub fn suggest_concurrency(current: u32, retry_count: usize) -> Option<u32> {
    if retry_count == 0 || current <= 1 {
        return None;
    }
    if retry_count >= 3 {
        return Some(1);
    }
    Some(current.saturating_sub(1).max(1))
}

pub fn emit_retry(notification: RetryNotification) {
    let Some(slot) = HOOK.get() else {
        return;
    };
    let Ok(guard) = slot.read() else {
        return;
    };
    if let Some(hook) = guard.as_ref() {
        hook(notification);
    }
}

pub fn retry_context_label() -> Option<String> {
    RETRY_CONTEXT.try_with(|ctx| ctx.clone()).ok().flatten()
}

pub async fn with_retry_context<F, Fut, T>(label: Option<String>, f: F) -> T
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = T>,
{
    RETRY_CONTEXT.scope(label, f()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggest_lower_concurrency_after_retries() {
        assert_eq!(suggest_concurrency(3, 0), None);
        assert_eq!(suggest_concurrency(3, 1), Some(2));
        assert_eq!(suggest_concurrency(3, 3), Some(1));
        assert_eq!(suggest_concurrency(1, 5), None);
    }
}