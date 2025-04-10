pub struct ScrapeConfig {
    pub base_url: String,
    pub language: Option<String>,
    pub sleep_between_pages_millis: Option<u64>,
}
