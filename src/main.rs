mod item;
mod sources;

use std::error::Error;
use crate::item::item::currency::Currency::GBP;
use crate::sources::sources::militariamart::Militariamart;
use crate::sources::sources::Source;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let militariamart_scraper =  Militariamart::new("https://20thcenturymilitaria.com/".to_string(), GBP);
    let items = militariamart_scraper.gather().await?;

    for item in items {
        println!("{}\n", serde_json::to_string_pretty(&item).unwrap())
    }
    
    Ok(())
}
