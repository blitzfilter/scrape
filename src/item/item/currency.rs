use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Display, EnumString)]
pub enum Currency {
    EUR,
    GBP,
    USD,
}