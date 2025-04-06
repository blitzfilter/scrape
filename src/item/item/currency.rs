use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum Currency {
    EUR,
    GBP,
    USD,
}