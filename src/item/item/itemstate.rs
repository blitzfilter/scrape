use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ItemState {
    LISTED,
    AVAILABLE,
    RESERVED,
    SOLD,
    REMOVED,
}