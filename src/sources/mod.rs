pub mod sources {
    use crate::item::item::item::Item;
    use reqwest::Client;
    use std::error::Error;

    pub trait Source {
        async fn gather_page(
            &self,
            page_num: i16,
            client: &Client,
        ) -> Result<Vec<Item>, Box<dyn Error>>;

        async fn gather(&self, client: &Client) -> Result<Vec<Item>, Box<dyn Error>> {
            let mut all_items = Vec::new();
            let mut i: i16 = 1;
            loop {
                let items = self.gather_page(i, client).await?;
                if items.is_empty() {
                    break;
                } else {
                    all_items.extend(items);
                }
                i += 1;
            }
            Ok(all_items)
        }
    }

    pub(crate) mod militariamart;
}
