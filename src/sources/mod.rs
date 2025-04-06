pub mod sources {
    use crate::item::item::item::Item;
    use std::error::Error;

    pub trait Source {
        async fn gather(&self) -> Result<Vec<Item>, Box<dyn Error>>;
    }

    pub(crate) mod militariamart;
}
