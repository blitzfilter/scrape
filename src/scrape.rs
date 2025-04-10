pub use async_trait::async_trait;
use item::item::ItemDiff;
use reqwest::Client;
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;

#[async_trait]
pub trait Scrape: Send + Sync {
    async fn gather_page(
        &self,
        page_num: i16,
        client: &Client,
    ) -> Result<Vec<ItemDiff>, Box<dyn Error + Send + Sync>>;

    async fn gather(
        &self,
        client: &Client,
        sleep_between_pages_millis: Option<u64>,
    ) -> Result<Vec<ItemDiff>, Box<dyn Error + Send + Sync>> {
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
