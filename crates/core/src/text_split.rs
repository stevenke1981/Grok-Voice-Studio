use crate::MAX_TTS_CHARS;

/// Split text into chunks at sentence boundaries when over max chars.
pub fn split_long_text(text: &str, max_chars: usize) -> Vec<String> {
    if text.chars().count() <= max_chars {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut current = String::new();

    for sentence in split_sentences(text) {
        let combined_len = current.chars().count() + sentence.chars().count();
        if combined_len > max_chars && !current.is_empty() {
            chunks.push(current.trim().to_string());
            current = String::new();
        }
        if sentence.chars().count() > max_chars {
            if !current.is_empty() {
                chunks.push(current.trim().to_string());
                current = String::new();
            }
            for part in hard_split(&sentence, max_chars) {
                chunks.push(part);
            }
        } else {
            current.push_str(&sentence);
        }
    }

    if !current.trim().is_empty() {
        chunks.push(current.trim().to_string());
    }

    chunks
}

fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut buf = String::new();
    for ch in text.chars() {
        buf.push(ch);
        if matches!(ch, '。' | '！' | '？' | '!' | '?' | '\n') {
            sentences.push(buf.clone());
            buf.clear();
        }
    }
    if !buf.trim().is_empty() {
        sentences.push(buf);
    }
    if sentences.is_empty() {
        sentences.push(text.to_string());
    }
    sentences
}

fn hard_split(text: &str, max_chars: usize) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    chars
        .chunks(max_chars)
        .map(|c| c.iter().collect())
        .collect()
}

pub fn default_split(text: &str) -> Vec<String> {
    split_long_text(text, MAX_TTS_CHARS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_short_text() {
        let r = split_long_text("你好。", 100);
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn splits_long_text() {
        let text = "第一句。".repeat(2000);
        let r = split_long_text(&text, 100);
        assert!(r.len() > 1);
        for chunk in &r {
            assert!(chunk.chars().count() <= 100);
        }
    }
}