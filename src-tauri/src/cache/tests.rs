//! Unit tests for cache logic + flush behavior.

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn cache_hits_capacity_flush() {
        let cache = Cache::new(2, 60); // capacity 2, TTL 60 s
        cache.push(
            ChatMessage::new("user", "hello"),
            ChatMessage::new("assistant", "hi"),
        );
        cache.push(
            ChatMessage::new("user", "how are you"),
            ChatMessage::new("assistant", "i'm fine"),
        );
        // third push triggers flush
        cache.push(
            ChatMessage::new("user", "foo"),
            ChatMessage::new("assistant", "bar"),
        );
        thread::sleep(std::time::Duration::from_millis(1100));
        // TODO: assert file contents via mmap
    }
}