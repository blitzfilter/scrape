mod item;
mod sources;

use crate::item::item::currency::Currency::{EUR, GBP};
use crate::sources::sources::Source;
use crate::sources::sources::militariamart::Militariamart;
use reqwest::Client;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let century20militaria =
        Militariamart::new("https://20thcenturymilitaria.com/".to_string(), GBP);
    let antiquitiesofthereich =
        Militariamart::new("https://antiquitiesofthereich.com/".to_string(), GBP);
    let hollandpatch = Militariamart::new("https://hollandpatch.com/".to_string(), EUR);
    let khakicolonel = Militariamart::new("https://khakicolonel.com/".to_string(), GBP);

    let client = Client::new();
    let items = hollandpatch.gather(&client).await?;

    let size = items.len();

    for item in items {
        println!("{}\n", serde_json::to_string_pretty(&item).unwrap())
    }

    println!("Size: {}", size);

    Ok(())
}
