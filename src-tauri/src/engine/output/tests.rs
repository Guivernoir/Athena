#[cfg(test)]
mod tests {
    use super::*;
    use engine::retrieval::result::SearchResult;

    #[test]
    fn format_and_inject() {
        let system = "sys".into();
        let memory = vec![SearchResult {
            score: 0.9,
            content: "m1".into(),
            source: "memory",
        }];
        let payload = format_results(system, memory, vec![], vec![]);
        let prompt = inject(payload);
        assert!(prompt.contains("m1"));
    }
}