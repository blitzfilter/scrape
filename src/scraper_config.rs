use item_core::language::Language;
use item_core::price::Currency;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ScraperConfig {
    #[serde(rename = "baseUrl")]
    pub base_url: String,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub currency: Option<Currency>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub language: Option<Language>,

    #[serde(
        rename = "shopDimension",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub shop_dimension: Option<u64>,

    #[serde(
        rename = "sleepBetweenPagesMillis",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub sleep_between_pages_millis: Option<u64>,
}

impl ScraperConfig {
    pub fn new(base_url: String) -> Self {
        ScraperConfig {
            base_url,
            currency: None,
            language: None,
            shop_dimension: None,
            sleep_between_pages_millis: None,
        }
    }

    // region fluent_setter

    pub fn base_url(&mut self, base_url: String) -> &mut Self {
        self.base_url = base_url;
        self
    }

    pub fn currency(&mut self, currency: Currency) -> &mut Self {
        self.currency = Some(currency);
        self
    }

    pub fn language(&mut self, language: Language) -> &mut Self {
        self.language = Some(language);
        self
    }

    pub fn shop_dimension(&mut self, shop_dimension: u64) -> &mut Self {
        self.shop_dimension = Some(shop_dimension);
        self
    }

    pub fn sleep_between_pages_millis(&mut self, sleep_between_pages_millis: u64) -> &mut Self {
        self.sleep_between_pages_millis = Some(sleep_between_pages_millis);
        self
    }

    // endregion
}
