use crate::item::item::currency::Currency;
use crate::item::item::item::Item;
use crate::item::item::itemstate::ItemState;
use crate::item::item::itemtype::ItemType;
use crate::sources::sources::Source;
use reqwest::Client;
use scraper::{ElementRef, Html, Selector};
use std::error::Error;
use std::fmt::format;

pub struct Militariamart {
    base_url: String,
    currency: Currency,
}

impl Militariamart {
    pub fn new(base_url: String, currency: Currency) -> Self {
        Self { base_url, currency }
    }
}

impl Source for Militariamart {
    async fn gather(&self) -> Result<Vec<Item>, Box<dyn Error>> {
        let client = Client::new();

        let html = client
            .get(format!("{}shop.php?pg=10", &self.base_url))
            .send()
            .await?
            .text()
            .await?;
        let document = Html::parse_document(&html);
        let shop_items = document
            .select(&Selector::parse("div.shopitem > div.inner-wrapper").unwrap())
            .map(|shop_item| {
                let item_id = extract_item_id(shop_item).unwrap();
                return Item::new(
                    item_id.clone(),
                    extract_name(shop_item).unwrap(),
                    extract_description(shop_item),
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(self.currency),
                    None,
                    extract_state(shop_item).unwrap(),
                    format!("{}shop.php?code={}", &self.base_url, item_id.clone()),
                    extract_image_url(shop_item).map(|relative_image_url| {
                        format!("{}{}", &self.base_url, relative_image_url)
                    }),
                );
            })
            .collect::<Vec<_>>();

        Ok(shop_items)
    }
}

fn extract_item_id(shop_item: ElementRef) -> Option<String> {
    shop_item
        .select(&Selector::parse("div.block-text > p.itemCode > a").unwrap())
        .next()
        .unwrap()
        .attr("href")
        .map(|href| href.strip_prefix("?code="))
        .flatten()
        .map(String::from)
}

fn extract_name(shop_item: ElementRef) -> Option<String> {
    shop_item
        .select(&Selector::parse("div.block-text > a.shopitemTitle").unwrap())
        .next()
        .unwrap()
        .attr("title")
        .map(String::from)
}

fn extract_description(shop_item: ElementRef) -> Option<String> {
    // TODO: This only gathers the description for the catalog-page.
    //       It may have been shortened. If so, it ends with '...'.
    //       If it does, go the the items page and parse full description there
    shop_item
        .select(&Selector::parse("div.block-text > p.itemDescription").unwrap())
        .next()
        .map(|desc_elem| desc_elem.text().next().map(|text| text.trim().to_string()))
        .flatten()
}

fn extract_image_url(shop_item: ElementRef) -> Option<String> {
    shop_item
        .select(&Selector::parse("div.block-image > a > img").unwrap())
        .next()
        .unwrap()
        .attr("src")
        .map(String::from)
}

fn extract_state(shop_item: ElementRef) -> Option<ItemState> {
    Some(ItemState::AVAILABLE)
}
