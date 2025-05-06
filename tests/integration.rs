use async_trait::async_trait;
use item_core::item_data::ItemData;
use item_core::item_state::ItemState;
use item_core::language::Language::{DE, EN};
use item_core::price::Currency::EUR;
use item_core::price::Price;
use item_read::item_hash::get_item_event_hashes_by_source_id;
use scrape::scrape_and_push;
use scrape::scraper::{ScrapeError, Scraper};
use scrape::scraper_config::ScraperConfig;
use std::collections::HashMap;
use std::time::Duration;
use test_api::generator::Generator;
use test_api::localstack::{get_dynamodb_client, get_sqs_client};
use test_api::test_api_macros::blitzfilter_data_ingestion_test;
use tokio::time::sleep;

#[blitzfilter_data_ingestion_test]
async fn should_scrape_and_push() {
    struct TestScraper {}

    #[async_trait]
    impl Scraper for TestScraper {
        async fn scrape_page(
            &self,
            page_num: i16,
            _: &reqwest::Client,
        ) -> Result<Vec<ItemData>, ScrapeError> {
            match page_num {
                1 => Ok(vec![
                    ItemData::generate()
                        .source_id("https://foo.bar".to_string())
                        .to_owned(),
                ]),
                _ => Ok(vec![]),
            }
        }
    }

    let scraper = TestScraper {};
    let scraper_config = ScraperConfig::new("https://foo.bar".to_string());
    let reqwest_client = reqwest::Client::new();

    let scrape_and_push_res = scrape_and_push(
        &scraper,
        &scraper_config,
        &reqwest_client,
        get_sqs_client().await,
        get_dynamodb_client().await,
        "http://sqs.eu-central-1.localhost.localstack.cloud:4566/000000000000/write_lambda_queue",
    )
    .await;
    assert!(scrape_and_push_res.is_ok());
    let pushed_count = scrape_and_push_res.unwrap();
    assert_eq!(pushed_count, 1);

    // Wait for SQS and Lambda to work...
    sleep(Duration::from_secs(15)).await;

    let read_res =
        get_item_event_hashes_by_source_id("https://foo.bar", false, get_dynamodb_client().await)
            .await;
    assert!(read_res.is_ok());

    let read = read_res.unwrap();
    assert_eq!(read.len(), 1);
}

#[blitzfilter_data_ingestion_test]
async fn should_scrape_and_push_only_diffs() {
    struct TestScraper1 {}
    #[async_trait]
    impl Scraper for TestScraper1 {
        async fn scrape_page(
            &self,
            page_num: i16,
            _: &reqwest::Client,
        ) -> Result<Vec<ItemData>, ScrapeError> {
            match page_num {
                1 => Ok(vec![ItemData {
                    item_id: "https://foo.bar#123456".to_string(),
                    created: Some("2010-01-01T12:00:00.001+01:00".to_string()),
                    source_id: Some("https://foo.bar".to_string()),
                    state: Some(ItemState::AVAILABLE),
                    price: Some(Price::new(EUR, 42f32)),
                    category: Some("foo".to_string()),
                    name: HashMap::from([(EN, "bar".to_string()), (DE, "balken".to_string())]),
                    description: HashMap::from([
                        (EN, "baz".to_string()),
                        (DE, "basis".to_string()),
                    ]),
                    url: Some("https://foo.bar?item=123456".to_string()),
                    image_url: Some("https://foo.bar?item_img=123456".to_string()),
                }]),
                _ => Ok(vec![]),
            }
        }
    }

    struct TestScraper2 {}
    #[async_trait]
    impl Scraper for TestScraper2 {
        async fn scrape_page(
            &self,
            page_num: i16,
            _: &reqwest::Client,
        ) -> Result<Vec<ItemData>, ScrapeError> {
            match page_num {
                1 => Ok(vec![
                    ItemData {
                        item_id: "https://foo.bar#123456".to_string(),
                        created: Some("2010-01-01T12:00:00.001+01:00".to_string()),
                        source_id: Some("https://foo.bar".to_string()),
                        state: Some(ItemState::AVAILABLE),
                        price: Some(Price::new(EUR, 42f32)),
                        category: Some("foo".to_string()),
                        name: HashMap::from([(EN, "bar".to_string()), (DE, "balken".to_string())]),
                        description: HashMap::from([
                            (EN, "baz".to_string()),
                            (DE, "basis".to_string()),
                        ]),
                        url: Some("https://foo.bar?item=123456".to_string()),
                        image_url: Some("https://foo.bar?item_img=123456".to_string()),
                    },
                    ItemData {
                        item_id: "https://foo.bar#123457".to_string(),
                        created: Some("2011-01-01T12:00:00.001+01:00".to_string()),
                        source_id: Some("https://foo.bar".to_string()),
                        state: Some(ItemState::RESERVED),
                        price: None,
                        category: Some("foo".to_string()),
                        name: HashMap::from([(EN, "bar".to_string()), (DE, "balken".to_string())]),
                        description: HashMap::from([
                            (EN, "baz".to_string()),
                            (DE, "basis".to_string()),
                        ]),
                        url: Some("https://foo.bar?item=123457".to_string()),
                        image_url: Some("https://foo.bar?item_img=123457".to_string()),
                    },
                ]),
                _ => Ok(vec![]),
            }
        }
    }

    let scraper1 = TestScraper1 {};
    let scraper_config = ScraperConfig::new("https://foo.bar".to_string());
    let reqwest_client = reqwest::Client::new();

    let scrape_and_push_res = scrape_and_push(
        &scraper1,
        &scraper_config,
        &reqwest_client,
        get_sqs_client().await,
        get_dynamodb_client().await,
        "http://sqs.eu-central-1.localhost.localstack.cloud:4566/000000000000/write_lambda_queue",
    )
    .await;
    assert!(scrape_and_push_res.is_ok());
    let pushed_count = scrape_and_push_res.unwrap();
    assert_eq!(pushed_count, 1);

    // Wait for SQS and Lambda to work...
    sleep(Duration::from_secs(15)).await;

    let read_res =
        get_item_event_hashes_by_source_id("https://foo.bar", false, get_dynamodb_client().await)
            .await;
    assert!(read_res.is_ok());

    let read = read_res.unwrap();
    assert_eq!(read.len(), 1);

    // Only push second item which is new, first one stayed unchanged
    let scraper2 = TestScraper2 {};
    let reqwest_client = reqwest::Client::new();

    let scrape_and_push_res = scrape_and_push(
        &scraper2,
        &scraper_config,
        &reqwest_client,
        get_sqs_client().await,
        get_dynamodb_client().await,
        "http://sqs.eu-central-1.localhost.localstack.cloud:4566/000000000000/write_lambda_queue",
    )
    .await;
    assert!(scrape_and_push_res.is_ok());
    let pushed_count = scrape_and_push_res.unwrap();
    assert_eq!(pushed_count, 1);

    // Wait for SQS and Lambda to work...
    sleep(Duration::from_secs(15)).await;

    let read_res =
        get_item_event_hashes_by_source_id("https://foo.bar", false, get_dynamodb_client().await)
            .await;
    assert!(read_res.is_ok());

    let read = read_res.unwrap();
    assert_eq!(read.len(), 2);
}
