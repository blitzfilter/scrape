pub mod sources {
    use crate::item::item::item::Item;
    use std::error::Error;
    use reqwest::Client;

    pub trait Source {
        async fn gather(&self, client: &Client) -> Result<Vec<Item>, Box<dyn Error>>;
        async fn gather_page(&self, page_num: i16, client: &Client) -> Result<Vec<Item>, Box<dyn Error>>;
    }

    pub(crate) mod militariamart;
}
