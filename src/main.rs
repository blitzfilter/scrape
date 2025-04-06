mod item;
mod sources;

use crate::item::item::currency::Currency::{EUR, GBP};
use crate::sources::sources::Source;
use crate::sources::sources::militariamart::Militariamart;
use reqwest::Client;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let hollandpatch = Militariamart::new("https://hollandpatch.com".to_string(), None, EUR);
    let a2zmilitarycollectables1 = Militariamart::new("https://a2zmilitarycollectables.co.uk".to_string(), Some(1), GBP);
    let a2zmilitarycollectables2 = Militariamart::new("https://a2zmilitarycollectables.co.uk".to_string(), Some(2), GBP);
    
    // This one is good for testing/demo as they only have a few items
    let liverpoolmilitaria = Militariamart::new("https://liverpoolmilitaria.com".to_string(), None, GBP);

    let client = Client::new();
    let items = liverpoolmilitaria.gather(&client).await?;
    let size = items.len();

    for item in items {
        println!("{}\n", serde_json::to_string_pretty(&item).unwrap())
    }

    println!("Size: {}", size);

    Ok(())
}
