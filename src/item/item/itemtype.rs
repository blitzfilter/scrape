use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ItemType {
    ORIGINAL,
    REPLICA,
}
