pub mod hash_comparison;
pub mod scraper;
pub mod scraper_config;

use crate::hash_comparison::drop_unchanged_diffs;
use crate::scraper::Scraper;
use crate::scraper_config::ScraperConfig;
use aws_sdk_sqs::types::SendMessageBatchRequestEntry;
use futures::StreamExt;
pub use item_core;
use item_read::item_hash::get_latest_item_event_hash_map_by_source_id;
use uuid::Uuid;

pub const MAX_SQS_BATCH_SIZE: usize = 10;

pub async fn scrape_and_push(
    scraper: &impl Scraper,
    scraper_config: &ScraperConfig,
    reqwest_client: &reqwest::Client,
    sqs_client: &aws_sdk_sqs::Client,
    dynamodb_client: &aws_sdk_dynamodb::Client,
) {
    let source_id = format!("source#{}", scraper_config.base_url);
    let item_hashes_map = get_latest_item_event_hash_map_by_source_id(&source_id, dynamodb_client)
        .await
        .expect("TODO: Shouldn't fail retrieving latest item hashes");
    scraper
        .scrape(reqwest_client, scraper_config.sleep_between_pages_millis)
        .chunks(MAX_SQS_BATCH_SIZE)
        .for_each_concurrent(5, |diff_results| async {
            let mut diffs = diff_results
                .into_iter()
                .filter_map(|diff_result| diff_result.ok())
                .collect::<Vec<_>>();

            drop_unchanged_diffs(&mut diffs, &item_hashes_map);

            let msg_entries = diffs
                .into_iter()
                .map(|diff| {
                    SendMessageBatchRequestEntry::builder()
                        .message_body(
                            serde_json::to_string(&diff)
                                .expect("TODO: shouldn't fail converting to json"),
                        )
                        .id(Uuid::new_v4().to_string())
                        .build()
                        .expect("shouldn't fail because 'id' and 'message_body' have been set")
                })
                .collect::<Vec<_>>();

            sqs_client
                .send_message_batch()
                .queue_url("http://sqs.eu-central-1.localhost.localstack.cloud:4566/000000000000/write_lambda_queue")
                .set_entries(Some(msg_entries))
                .send()
                .await
                .expect("TODO: handle sqs error");
        })
        .await;
}
