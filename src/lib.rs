pub mod hash_comparison;
pub mod lambda_handler;
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
use lambda_runtime::Diagnostic;
use std::error::Error;
use std::fmt::Display;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};
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

impl Into<Diagnostic> for ScrapePushError {
    fn into(self) -> Diagnostic {
        match self {
            ScrapePushError::QueryItemEventHashesError(err) => Diagnostic {
                error_type: "QueryItemEventHashesError".to_string(),
                error_message: err.to_string(),
            },
        }
    }
}

pub async fn scrape_and_push(
    scraper: &impl Scraper,
    scraper_config: &ScraperConfig,
    reqwest_client: &reqwest::Client,
    sqs_client: &aws_sdk_sqs::Client,
    dynamodb_client: &aws_sdk_dynamodb::Client,
    item_write_lambda_q_url: &str,
) -> Result<usize, ScrapePushError> {
    let total_sent_count = Arc::new(Mutex::new(0usize));
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
                        warn!(error = %e, "Scraping item failed.");
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
                            error = %e,
                            body = ?diff,
                            "Serializing ItemData failed.",
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
                    let failures = batch_output.failed;
                    let successes = batch_output.successful.len();
                    info!(
                        successful = successes,
                        failed = failures.len(),
                        failures = ?failures,
                        "Successfully sent batch."
                    );

                    *total_sent_count.lock().await += successes;
                }
                Err(e) => warn!(error = %e, "Failed message batch."),
            }
        })
        .await;

    Ok(*total_sent_count.lock().await)
}
