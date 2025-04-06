mod item;
mod sources;

use std::error::Error;
use reqwest::Client;
use crate::item::item::currency::Currency::{EUR, GBP};
use crate::sources::sources::militariamart::Militariamart;
use crate::sources::sources::Source;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let century20militaria = Militariamart::new("https://20thcenturymilitaria.com/".to_string(), GBP);
    let antiquitiesofthereich = Militariamart::new("https://antiquitiesofthereich.com/".to_string(), GBP);
    let hollandpatch = Militariamart::new("https://hollandpatch.com/".to_string(), EUR);
    
    let client = Client::new();
    let items = hollandpatch.gather(&client).await?;

    for item in items {
        println!("{}\n", serde_json::to_string_pretty(&item).unwrap())
    }
    
    Ok(())
}
