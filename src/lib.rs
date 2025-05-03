pub mod hash_comparison;
pub mod scraper;
pub mod scraper_config;

use crate::hash_comparison::drop_unchanged_diffs;
use crate::scraper::Scraper;
use crate::scraper_config::ScraperConfig;
use aws_sdk_dynamodb::operation::query::QueryError;
use aws_sdk_sqs::config::http::HttpResponse;
use aws_sdk_sqs::error::SdkError;
use aws_sdk_sqs::types::SendMessageBatchRequestEntry;
use futures::StreamExt;
pub use item_core;
use item_read::item_hash::get_latest_item_event_hash_map_by_source_id;
use std::error::Error;
use std::fmt::Display;
use tracing::{error, warn};
use uuid::Uuid;

pub const MAX_SQS_BATCH_SIZE: usize = 10;

#[derive(Debug)]
pub enum ScrapePushError {
    QueryItemEventHashesError(SdkError<QueryError, HttpResponse>),
}

impl Display for ScrapePushError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScrapePushError::QueryItemEventHashesError(err) => {
                write!(f, "QueryItemEventHashesError error: {}", err)
            }
        }
    }
}

impl Error for ScrapePushError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ScrapePushError::QueryItemEventHashesError(err) => Some(err),
        }
    }
}

impl From<SdkError<QueryError, HttpResponse>> for ScrapePushError {
    fn from(err: SdkError<QueryError, HttpResponse>) -> Self {
        ScrapePushError::QueryItemEventHashesError(err)
    }
}

pub async fn scrape_and_push(
    scraper: &impl Scraper,
    scraper_config: &ScraperConfig,
    reqwest_client: &reqwest::Client,
    sqs_client: &aws_sdk_sqs::Client,
    dynamodb_client: &aws_sdk_dynamodb::Client,
    item_write_lambda_q_url: &str,
) -> Result<(), ScrapePushError> {
    let source_id = format!("source#{}", scraper_config.base_url);
    let item_hashes_map =
        get_latest_item_event_hash_map_by_source_id(&source_id, dynamodb_client).await?;
    scraper
        .scrape(reqwest_client, scraper_config.sleep_between_pages_millis)
        .chunks(MAX_SQS_BATCH_SIZE)
        .for_each_concurrent(5, |diff_results| async {
            let mut diffs = diff_results
                .into_iter()
                .filter_map(|diff_result| match diff_result {
                    Ok(diff) => Some(diff),
                    Err(e) => {
                        warn!("Scrape error: {}", e);
                        None
                    }
                })
                .collect::<Vec<_>>();

            drop_unchanged_diffs(&mut diffs, &item_hashes_map);

            let msg_entries = diffs
                .into_iter()
                .filter_map(|diff| match serde_json::to_string(&diff) {
                    Ok(body) => Some(
                        SendMessageBatchRequestEntry::builder()
                            .message_body(body)
                            .id(Uuid::new_v4().to_string())
                            .build()
                            .expect("shouldn't fail because 'id' and 'message_body' have been set"),
                    ),
                    Err(e) => {
                        error!(
                            "Failed serializing ItemData. ItemData: {:?}. Error: {e}",
                            &diff
                        );
                        None
                    }
                })
                .collect::<Vec<_>>();

            let batch_output_res = sqs_client
                .send_message_batch()
                .queue_url(item_write_lambda_q_url)
                .set_entries(Some(msg_entries))
                .send()
                .await;

            match batch_output_res {
                Ok(batch_output) => {
                    let failed = batch_output.failed;
                    let failed_count = failed.len();
                    if failed_count > 0 {
                        warn!(
                            "Sending batch messages was successful, but failed '{failed_count}' messages.",
                        );
                        failed.into_iter().for_each(|failure|warn!("Failed message: {:?}", failure));
                    }
                }
                Err(e) => warn!("Failed message batch: {e}"),
            }
        })
        .await;

    Ok(())
}
