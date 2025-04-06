use crate::item::item::currency::Currency;
use crate::item::item::itemstate::ItemState;
use crate::item::item::itemtype::ItemType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Item {
    item_id: String,
    name: String,
    description: Option<String>,
    category: Option<String>,
    lower_year: Option<i8>,
    upper_year: Option<i8>,
    lower_price: Option<f32>,
    upper_price: Option<f32>,
    currency: Option<Currency>,
    item_type: Option<ItemType>,
    item_state: ItemState,
    url: String,
    image_url: Option<String>,
}

impl Item {
    pub fn new(
        item_id: String,
        name: String,
        description: Option<String>,
        category: Option<String>,
        lower_year: Option<i8>,
        upper_year: Option<i8>,
        lower_price: Option<f32>,
        upper_price: Option<f32>,
        currency: Option<Currency>,
        item_type: Option<ItemType>,
        item_state: ItemState,
        url: String,
        image_url: Option<String>,
    ) -> Self {
        Self {
            item_id,
            name,
            description,
            category,
            lower_year,
            upper_year,
            lower_price,
            upper_price,
            currency,
            item_type,
            item_state,
            url,
            image_url,
        }
    }
}
