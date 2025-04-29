use crate::scraper::ScrapeError::ReqwestError;
use async_stream::try_stream;
pub use async_trait::async_trait;
use futures::stream::BoxStream;
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
            ReqwestError(err) => write!(f, "Reqwest error: {}", err),
        }
    }
}

impl Error for ScrapeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ReqwestError(err) => Some(err),
        }
    }
}

impl From<reqwest::Error> for ScrapeError {
    fn from(value: reqwest::Error) -> Self {
        ReqwestError(value)
    }
}

#[async_trait]
pub trait Scraper: Send + Sync {
    async fn scrape_page(
        &self,
        page_num: i16,
        client: &reqwest::Client,
    ) -> Result<Vec<ItemData>, ScrapeError>;

    fn scrape(
        &self,
        client: &reqwest::Client,
        sleep_between_pages_millis: Option<u64>,
    ) -> BoxStream<Result<ItemData, ScrapeError>> {
        let client = client.clone();

        Box::pin(try_stream! {
            let mut i: i16 = 1;
            loop {
                let items = self.scrape_page(i, &client).await?;
                if items.is_empty() {
                    break;
                }
                for item in items {
                    yield item;
                }
                if let Some(duration) = sleep_between_pages_millis {
                    sleep(Duration::from_millis(duration)).await;
                }
                i += 1;
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::scraper::{Scraper, ScrapeError};
    use async_trait::async_trait;
    use futures::StreamExt;
    use item_core::item_data::ItemData;
    use reqwest::Client;
    use test_api::generator::Generator;

    struct TestScraper {}

    #[async_trait]
    impl Scraper for TestScraper {
        async fn scrape_page(
            &self,
            page_num: i16,
            _: &Client,
        ) -> Result<Vec<ItemData>, ScrapeError> {
            match page_num {
                1 => Ok(ItemData::generate_many(10)),
                2 => Ok(ItemData::generate_many(5)),
                _ => Ok(vec![]),
            }
        }
    }

    #[tokio::test]
    async fn should_scrape_all_pages_for_scrape() {
        let client = Client::new();
        let items_count = TestScraper {}.scrape(&client, None).count().await;

        assert_eq!(items_count, 15);
    }
}
