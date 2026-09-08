// SPDX-License-Identifier: GPL-3.0-only
use std::collections::{HashMap, HashSet};

use serde::{Serialize, ser::SerializeMap};

/// Serializes a `HashMap` and ensures a sorted result.
pub fn serialize_map_sorted<S, K, V>(map: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    K: Serialize + Ord,
    V: Serialize,
{
    let mut entries: Vec<_> = map.iter().collect();
    entries.sort_by_key(|(k, _)| *k);

    let mut map_serializer = serializer.serialize_map(Some(entries.len()))?;

    for (key, value) in entries {
        map_serializer.serialize_entry(key, value)?;
    }

    map_serializer.end()
}

/// Serializes a `HashSet` and ensures a sorted result.
pub fn serialize_set_sorted<S, T>(set: &HashSet<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    T: Serialize + Ord,
{
    let mut values: Vec<_> = set.iter().collect();
    values.sort();

    values.serialize(serializer)
}
