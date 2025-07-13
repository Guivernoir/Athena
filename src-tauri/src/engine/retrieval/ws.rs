use serde::{Deserialize, Serialize};
use reqwest::{Client, StatusCode};
use tokio::sync::{Mutex, Semaphore};
use std::sync::Arc;
use std::time::{Duration, Instant};
use lru_time_cache::LruCache;
use tokio::time;
use tracing::{info, warn, error};
use thiserror::Error;
use directories::ProjectDirs;
use std::path::PathBuf;
use tokio::fs;

// Configuration constants
const MAX_CONCURRENT_REQUESTS: usize = 2;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(8);
const RATE_LIMIT_DELAY: Duration = Duration::from_secs(1);
const MAX_RETRIES: u32 = 2;
const CACHE_TTL: Duration = Duration::from_secs(3600 * 24 * 7); // 1 week

#[derive(Debug, Error)]
pub enum WebSearchError {
    #[error("API request failed: {0}")]
    RequestFailed(String),
    #[error("API quota exceeded")]
    QuotaExceeded,
    #[error("Request timed out")]
    Timeout,
    #[error("Offline mode - using cached results")]
    OfflineMode(Vec<SearchResult>),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Invalid API key")]
    InvalidApiKey,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub title: String,
    pub link: String,
    pub snippet: String,
    pub relevance_score: f32,
    pub cached_at: Option<i64>,
}

pub struct WebSearcher {
    client: Client,
    api_key: Option<String>,
    cache: Arc<Mutex<LruCache<String, Vec<SearchResult>>>>,
    semaphore: Arc<Semaphore>,
    last_request: Arc<Mutex<Instant>>,
    cache_dir: PathBuf,
    is_offline: bool,
}

impl WebSearcher {
    pub async fn new() -> Result<Self, WebSearchError> {
        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .user_agent("NativeApp/1.0")
            .build()
            .map_err(|e| WebSearchError::ConfigError(e.to_string()))?;

        // Set up cache directory
        let proj_dirs = ProjectDirs::from("com", "yourcompany", "yourapp")
            .ok_or_else(|| WebSearchError::ConfigError("Could not get project directories".into()))?;
        let cache_dir = proj_dirs.cache_dir().to_path_buf();
        
        // Create cache dir if it doesn't exist
        fs::create_dir_all(&cache_dir).await.ok();

        Ok(Self {
            client,
            api_key: None,
            cache: Arc::new(Mutex::new(LruCache::with_expiry_duration(CACHE_TTL))),
            semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS)),
            last_request: Arc::new(Mutex::new(Instant::now())),
            cache_dir,
            is_offline: false,
        })
    }

    pub fn set_api_key(&mut self, key: String) {
        self.api_key = Some(key);
    }

    pub fn set_offline_mode(&mut self, offline: bool) {
        self.is_offline = offline;
    }

    #[instrument(skip(self))]
    pub async fn search(&self, query: &str, num_results: usize) -> Result<Vec<SearchResult>, WebSearchError> {
        // Check memory cache first
        if let Some(cached) = self.check_memory_cache(query).await {
            return Ok(cached);
        }

        // Check disk cache if offline
        if self.is_offline {
            if let Some(disk_cached) = self.check_disk_cache(query).await? {
                return Err(WebSearchError::OfflineMode(disk_cached));
            }
            return Err(WebSearchError::RequestFailed("Offline mode with no cached results".into()));
        }

        // Validate API key
        let api_key = self.api_key.as_ref()
            .ok_or(WebSearchError::InvalidApiKey)?;

        // Rate limiting
        self.enforce_rate_limit().await;

        // Execute with concurrency control
        let permit = self.semaphore.acquire().await
            .map_err(|_| WebSearchError::RequestFailed("Semaphore closed".into()))?;

        let result = self.execute_search_with_retry(query, num_results, api_key).await;

        drop(permit); // Release semaphore permit

        // Cache successful results
        if let Ok(results) = &result {
            self.cache_results(query, results.clone()).await;
        }

        result
    }

    async fn execute_search_with_retry(
        &self,
        query: &str,
        num_results: usize,
        api_key: &str,
    ) -> Result<Vec<SearchResult>, WebSearchError> {
        let mut retries = 0;
        let mut last_error = None;

        while retries <= MAX_RETRIES {
            match self.execute_search(query, num_results, api_key).await {
                Ok(results) => return Ok(results),
                Err(e) => {
                    last_error = Some(e);
                    if retries < MAX_RETRIES {
                        let delay = RATE_LIMIT_DELAY * 2u32.pow(retries);
                        time::sleep(delay).await;
                        retries += 1;
                    }
                }
            }
        }

        Err(last_error.unwrap_or(WebSearchError::RequestFailed("Unknown error".into())))
    }

    async fn execute_search(
        &self,
        query: &str,
        num_results: usize,
        api_key: &str,
    ) -> Result<Vec<SearchResult>, WebSearchError> {
        let now = Instant::now();
        {
            let mut last_request = self.last_request.lock().await;
            *last_request = now;
        }

        let response = self.client
            .post("https://google.serper.dev/search")
            .header("X-API-KEY", api_key)
            .json(&serde_json::json!({
                "q": query,
                "num": num_results,
            }))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    WebSearchError::Timeout
                } else if e.is_connect() {
                    self.set_offline(true);
                    WebSearchError::RequestFailed("Network unavailable".into())
                } else {
                    WebSearchError::RequestFailed(e.to_string())
                }
            })?;

        match response.status() {
            StatusCode::OK => {
                let mut results: Vec<SearchResult> = response
                    .json()
                    .await
                    .map_err(|e| WebSearchError::RequestFailed(e.to_string()))?;

                // Add cache timestamp
                let cached_at = Some(chrono::Utc::now().timestamp());
                for result in &mut results {
                    result.cached_at = cached_at;
                }

                Ok(results)
            }
            StatusCode::TOO_MANY_REQUESTS => Err(WebSearchError::QuotaExceeded),
            StatusCode::UNAUTHORIZED => Err(WebSearchError::InvalidApiKey),
            _ => Err(WebSearchError::RequestFailed(format!("HTTP {}", response.status()))),
        }
    }

    // Cache management functions
    async fn check_memory_cache(&self, query: &str) -> Option<Vec<SearchResult>> {
        let mut cache = self.cache.lock().await;
        cache.get(query).cloned()
    }

    async fn check_disk_cache(&self, query: &str) -> Result<Option<Vec<SearchResult>>, WebSearchError> {
        let cache_file = self.cache_dir.join(format!("{}.json", sanitize_filename(query)));
        if cache_file.exists() {
            let data = fs::read_to_string(&cache_file).await
                .map_err(|e| WebSearchError::ConfigError(e.to_string()))?;
            let results: Vec<SearchResult> = serde_json::from_str(&data)
                .map_err(|e| WebSearchError::ConfigError(e.to_string()))?;
            Ok(Some(results))
        } else {
            Ok(None)
        }
    }

    async fn cache_results(&self, query: &str, results: Vec<SearchResult>) {
        // Memory cache
        {
            let mut cache = self.cache.lock().await;
            cache.insert(query.to_string(), results.clone());
        }

        // Disk cache (fire and forget)
        let cache_file = self.cache_dir.join(format!("{}.json", sanitize_filename(query)));
        if let Ok(data) = serde_json::to_string(&results) {
            tokio::spawn(async move {
                fs::write(cache_file, data).await.ok();
            });
        }
    }

    async fn enforce_rate_limit(&self) {
        let delay = {
            let last_request = self.last_request.lock().await;
            let elapsed = last_request.elapsed();
            if elapsed < RATE_LIMIT_DELAY {
                RATE_LIMIT_DELAY - elapsed
            } else {
                Duration::from_secs(0)
            }
        };

        if !delay.is_zero() {
            time::sleep(delay).await;
        }
    }
}

fn sanitize_filename(query: &str) -> String {
    query.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Server};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_offline_mode() {
        let temp_dir = tempdir().unwrap();
        let mut searcher = WebSearcher::new().await.unwrap();
        searcher.cache_dir = temp_dir.path().to_path_buf();
        searcher.set_offline_mode(true);

        // Should fail with offline error
        let result = searcher.search("test", 1).await;
        assert!(matches!(result, Err(WebSearchError::RequestFailed(_))));

        // Add to cache then try again
        let test_results = vec![SearchResult {
            title: "Cached".to_string(),
            link: "https://example.com".to_string(),
            snippet: "Cached result".to_string(),
            relevance_score: 0.9,
            cached_at: None,
        }];
        searcher.cache_results("test", test_results.clone()).await;

        let result = searcher.search("test", 1).await;
        assert!(matches!(result, Err(WebSearchError::OfflineMode(_))));
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let mut server = Server::new();
        let _mock = mock("POST", "/search")
            .with_status(200)
            .with_body(r#"{"results":[]}"#)
            .create();

        let searcher = WebSearcher::new().await.unwrap();
        searcher.set_api_key("test".to_string());

        let start = Instant::now();
        let _ = searcher.search("test1", 1).await;
        let _ = searcher.search("test2", 1).await;
        let duration = start.elapsed();

        assert!(duration >= RATE_LIMIT_DELAY);
    }
}