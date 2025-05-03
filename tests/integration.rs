use async_trait::async_trait;
use item_core::item_data::ItemData;
use item_read::item_hash::get_item_event_hashes_by_source_id;
use scrape::scrape_and_push;
use scrape::scraper::{ScrapeError, Scraper};
use scrape::scraper_config::ScraperConfig;
use std::time::Duration;
use test_api::generator::Generator;
use test_api::localstack::{get_dynamodb_client, get_sqs_client};
use test_api::test_api_macros::blitzfilter_data_ingestion_test;
use tokio::time::sleep;

struct TestScraper {}

#[async_trait]
impl Scraper for TestScraper {
    async fn scrape_page(
        &self,
        page_num: i16,
        _: &reqwest::Client,
    ) -> Result<Vec<ItemData>, ScrapeError> {
        match page_num {
            1 => Ok(vec![ItemData::generate().source_id("http://foo.bar".to_string()).to_owned()]),
            _ => Ok(vec![]),
        }
    }
}

#[blitzfilter_data_ingestion_test]
async fn should_scrape_push_sqs_trigger_lambda_insert_dynamodb() {
    let scraper = TestScraper {};
    let scraper_config = ScraperConfig {
        sleep_between_pages_millis: None,
        base_url: "http://foo.bar".to_string(),
    };
    let reqwest_client = reqwest::Client::new();
    let sqs_client = get_sqs_client().await;
    let dynamodb_client = get_dynamodb_client().await;

    let scrape_and_push_res = scrape_and_push(
        &scraper,
        &scraper_config,
        &reqwest_client,
        sqs_client,
        dynamodb_client,
        "http://sqs.eu-central-1.localhost.localstack.cloud:4566/000000000000/write_lambda_queue"
    )
    .await;
    assert!(scrape_and_push_res.is_ok());

    // Wait for SQS and Lambda to work...
    sleep(Duration::from_secs(15)).await;

    let read_res =
        get_item_event_hashes_by_source_id("http://foo.bar", false, dynamodb_client).await;
    assert!(read_res.is_ok());
    
    let read = read_res.unwrap();
    assert_eq!(read.len(), 1);
}
