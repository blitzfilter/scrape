use item_core::item_data::ItemData;
use item_core::item_hash::ItemHash;
use std::collections::HashMap;

pub fn drop_unchanged_diffs(diffs: &mut Vec<ItemData>, item_id_hash_map: &HashMap<String, String>) {
    diffs.retain(|diff| {
        let old_hash = item_id_hash_map.get(diff.item_id.as_str());
        if old_hash.is_none() {
            return true;
        } else {
            let new_hash = &diff.hash();
            old_hash.unwrap().ne(new_hash)
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::hash_comparison::drop_unchanged_diffs;
    use item_core::item_data::ItemData;
    use item_core::item_hash::hash_item_details;
    use item_core::item_state::ItemState::{AVAILABLE, SOLD};
    use item_core::price::Currency::EUR;
    use item_core::price::Price;
    use std::collections::HashMap;

    fn make_items_diffs() -> Vec<ItemData> {
        vec![
            ItemData::new("foo#bar".to_string())
                .state(SOLD)
                .url("https://foo.com/item=bar".to_string())
                .clone(),
            ItemData::new("foo#baz".to_string())
                .state(AVAILABLE)
                .price(Price::new(EUR, 42f32))
                .url("https://foo.com/item=baz".to_string())
                .clone(),
        ]
    }

    #[test]
    fn should_not_drop_any_diffs_when_all_latest_hashes_differ() {
        let expected: Vec<ItemData> = make_items_diffs();
        let mut actual: Vec<ItemData> = expected.clone();
        let hashes = HashMap::from([(
            "foo#bar".to_string(),
            hash_item_details(Some(AVAILABLE), Some(42f32)),
        )]);

        drop_unchanged_diffs(&mut actual, &hashes);

        assert_eq!(expected, actual);
    }

    #[test]
    fn should_drop_all_diffs_when_all_latest_hashes_match() {
        let mut actual: Vec<ItemData> = make_items_diffs();
        let hashes = HashMap::from([
            ("foo#bar".to_string(), hash_item_details(Some(SOLD), None)),
            (
                "foo#baz".to_string(),
                hash_item_details(Some(AVAILABLE), Some(42f32)),
            ),
        ]);

        drop_unchanged_diffs(&mut actual, &hashes);

        assert!(actual.is_empty());
    }

    #[test]
    fn should_retain_only_actual_diffs() {
        let mut actual: Vec<ItemData> = make_items_diffs();
        let hashes = HashMap::from([
            (
                "foo#bar".to_string(),
                hash_item_details(Some(AVAILABLE), Some(42f32)),
            ),
            (
                "foo#baz".to_string(),
                hash_item_details(Some(AVAILABLE), Some(42f32)),
            ),
        ]);

        drop_unchanged_diffs(&mut actual, &hashes);

        let expected = vec![
            ItemData::new("foo#bar".to_string())
                .state(SOLD)
                .url("https://foo.com/item=bar".to_string())
                .clone(),
        ];
        assert_eq!(expected, actual);
    }
}
