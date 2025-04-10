use item::item::{ItemDiff, ItemEventHash, hash_item_details};
use std::collections::HashMap;

// assumes to be sorted by latest event_id
pub fn drop_irrelevant_diffs(diffs: &mut Vec<ItemDiff>, hashes: &Vec<ItemEventHash>) {
    let mut item_id_hash_map: HashMap<&str, &str> = HashMap::new();
    for item_event_hash in hashes {
        item_id_hash_map
            .entry(item_event_hash.get_item_id())
            .or_insert(&item_event_hash.hash);
    }

    diffs.retain(|diff| {
        let old_hash = item_id_hash_map.get(diff.item_id.as_str());
        if old_hash.is_none() {
            return true;
        } else {
            let new_hash = &hash_item_details(
                diff.item_state,
                diff.currency,
                diff.lower_price,
                diff.upper_price,
                diff.url.clone(),
            );
            old_hash.unwrap().ne(new_hash)
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::hash_comparison::drop_irrelevant_diffs;
    use item::currency::Currency::EUR;
    use item::item::{ItemDiff, ItemEventHash, hash_item_details};
    use item::itemstate::ItemState::{AVAILABLE, LISTED, RESERVED, SOLD};

    fn make_items_diffs() -> Vec<ItemDiff> {
        vec![
            ItemDiff::new("item#foo#bar".to_string())
                .item_state(SOLD)
                .url("https://foo.com/item=bar".to_string())
                .clone(),
            ItemDiff::new("item#foo#baz".to_string())
                .item_state(AVAILABLE)
                .currency(EUR)
                .lower_price(42f32)
                .upper_price(42f32)
                .url("https://foo.com/item=baz".to_string())
                .clone(),
        ]
    }

    #[test]
    fn should_not_drop_any_diffs_when_all_hashes_differ() {
        let expected: Vec<ItemDiff> = make_items_diffs();
        let mut actual: Vec<ItemDiff> = expected.clone();
        let hashes: Vec<ItemEventHash> = vec![
            ItemEventHash {
                event_id: "item#foo#bar#2025-01-02T12:00:00.001+01:00".to_string(),
                source_id: "item#foo".to_string(),
                hash: hash_item_details(
                    Some(AVAILABLE),
                    Some(EUR),
                    Some(42f32),
                    Some(42f32),
                    Some("https://foo.com/item=bar".to_string()),
                ),
            },
            ItemEventHash {
                event_id: "item#foo#bar#2025-01-01T12:00:00.001+01:00".to_string(),
                source_id: "item#foo".to_string(),
                hash: hash_item_details(
                    Some(LISTED),
                    None,
                    None,
                    None,
                    Some("https://foo.com/item=bar".to_string()),
                ),
            },
        ];

        drop_irrelevant_diffs(&mut actual, &hashes);

        assert_eq!(expected, actual);
    }

    #[test]
    fn should_drop_all_diffs_when_all_latest_hashes_match() {
        let mut actual: Vec<ItemDiff> = make_items_diffs();
        let hashes: Vec<ItemEventHash> = vec![
            ItemEventHash {
                event_id: "item#foo#bar#2025-02-01T12:00:00.001+01:00".to_string(),
                source_id: "item#foo".to_string(),
                hash: hash_item_details(
                    Some(SOLD),
                    None,
                    None,
                    None,
                    Some("https://foo.com/item=bar".to_string()),
                ),
            },
            ItemEventHash {
                event_id: "item#foo#bar#2025-01-01T12:00:00.001+01:00".to_string(),
                source_id: "item#foo".to_string(),
                hash: hash_item_details(
                    Some(RESERVED),
                    None,
                    None,
                    None,
                    Some("https://foo.com/item=bar".to_string()),
                ),
            },
            ItemEventHash {
                event_id: "item#foo#baz#2025-01-01T12:00:00.001+01:00".to_string(),
                source_id: "item#foo".to_string(),
                hash: hash_item_details(
                    Some(AVAILABLE),
                    Some(EUR),
                    Some(42f32),
                    Some(42f32),
                    Some("https://foo.com/item=baz".to_string()),
                ),
            },
        ];

        drop_irrelevant_diffs(&mut actual, &hashes);

        assert!(actual.is_empty());
    }

    #[test]
    fn should_retain_only_actual_diffs() {
        let mut actual: Vec<ItemDiff> = make_items_diffs();
        let hashes: Vec<ItemEventHash> = vec![
            ItemEventHash {
                event_id: "item#foo#bar#2025-02-01T12:00:00.001+01:00".to_string(),
                source_id: "item#foo".to_string(),
                hash: hash_item_details(
                    Some(RESERVED),
                    None,
                    None,
                    None,
                    Some("https://foo.com/item=bar".to_string()),
                ),
            },
            ItemEventHash {
                event_id: "item#foo#bar#2025-01-01T12:00:00.001+01:00".to_string(),
                source_id: "item#foo".to_string(),
                hash: hash_item_details(
                    Some(LISTED),
                    None,
                    None,
                    None,
                    Some("https://foo.com/item=bar".to_string()),
                ),
            },
            ItemEventHash {
                event_id: "item#foo#baz#2025-01-01T12:00:00.001+01:00".to_string(),
                source_id: "item#foo".to_string(),
                hash: hash_item_details(
                    Some(AVAILABLE),
                    Some(EUR),
                    Some(42f32),
                    Some(42f32),
                    Some("https://foo.com/item=baz".to_string()),
                ),
            },
        ];

        drop_irrelevant_diffs(&mut actual, &hashes);

        let expected = vec![
            ItemDiff::new("item#foo#bar".to_string())
                .item_state(SOLD)
                .url("https://foo.com/item=bar".to_string())
                .clone(),
        ];
        assert_eq!(expected, actual);
    }
}
