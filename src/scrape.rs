pub use async_trait::async_trait;
use item_core::item_data::ItemData;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug)]
pub enum ScrapeError {
    ReqwestError(reqwest::Error),
}

impl Display for ScrapeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ScrapeError::ReqwestError(err) => write!(f, "Reqwest error: {}", err),
        }
    }
}

impl Error for ScrapeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ScrapeError::ReqwestError(err) => Some(err),
        }
    }
}

#[async_trait]
pub trait Scrape: Send + Sync {
    async fn gather_page(
        &self,
        page_num: i16,
        client: &reqwest::Client,
    ) -> Result<Vec<ItemData>, ScrapeError>;

    async fn gather(
        &self,
        client: &reqwest::Client,
        sleep_between_pages_millis: Option<u64>,
    ) -> Result<Vec<ItemData>, ScrapeError> {
        let mut all_items = Vec::new();
        let mut i: i16 = 1;
        loop {
            let items = self.gather_page(i, client).await?;
            if items.is_empty() {
                break;
            } else {
                all_items.extend(items);
            }
            if sleep_between_pages_millis.is_some() {
                sleep(Duration::from_millis(sleep_between_pages_millis.unwrap())).await;
            }
            i += 1;
        }
        Ok(all_items)
    }
}
