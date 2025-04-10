use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Serialize, Deserialize, Copy, Clone, Display, EnumString)]
pub enum Language {
    DE,
    EN
}