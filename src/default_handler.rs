use crate::scraper::Scraper;
use crate::scraper_config::ScraperConfig;
use crate::{ScrapePushError, scrape_and_push};
use lambda_runtime::LambdaEvent;
use tracing::{error, info};

#[tracing::instrument(
    skip(event, reqwest_client, sqs_client, dynamodb_client),
    fields(req_id = %event.context.request_id))
]
pub async fn default_function_handler<T>(
    event: LambdaEvent<ScraperConfig>,
    reqwest_client: &reqwest::Client,
    sqs_client: &aws_sdk_sqs::Client,
    dynamodb_client: &aws_sdk_dynamodb::Client,
    item_write_lambda_q_url: &str,
) -> Result<(), ScrapePushError>
where
    T: From<ScraperConfig> + Scraper,
{
    let scraper_cfg = event.payload;
    let scraper: T = scraper_cfg.clone().into();
    info!(
        scraperConfig = serde_json::to_string_pretty(&scraper_cfg)
            .expect("shouldn't fail serializing ScraperConfig"),
        "Handler invoked."
    );

    let res = scrape_and_push(
        &scraper,
        &scraper_cfg,
        reqwest_client,
        sqs_client,
        dynamodb_client,
        item_write_lambda_q_url,
    )
    .await;

    match res {
        Ok(total_sent_count) => {
            info!(total = total_sent_count, "Handler finished.");
            Ok(())
        }
        Err(e) => {
            error!(error = %e,"Handler failed.");
            Err(e)
        }
    }
}
