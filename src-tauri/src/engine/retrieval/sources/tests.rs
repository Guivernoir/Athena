#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn web_smoke() {
        let src = WebSource::new();
        let res = src.search("rust programming", 3).await.unwrap();
        assert!(!res.is_empty());
    }
}