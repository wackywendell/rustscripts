/// Utility functions for my scripts.
#[warn(non_camel_case_types)]
#[warn(non_snake_case)]
#[warn(unused_qualifications)]
#[warn(non_upper_case_globals)]
#[warn(missing_docs)]

use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};

/// Count the number of occurrences of each value in an iterator
pub fn counter<K, I>(list: I) -> HashMap<K, u32>
    where K: Eq + std::hash::Hash,
          I: Iterator<Item = K>
{
    let mut counter: HashMap<K, u32> = HashMap::new();
    for key in list {
        match counter.entry(key) {
            Vacant(entry) => {
                entry.insert(1);
            }
            Occupied(entry) => {
                (*entry.into_mut()) += 1;
            }
        }
    }
    counter
}

#[test]
fn test_counter() {
    let my_list: Vec<&str> = vec![];
    let count: HashMap<&str, u32> = counter(my_list.into_iter());
    assert_eq!(count.get(&"a"), None);

    let my_list = vec!["a", "b", "cd", "a", "a", "b"];
    let count: HashMap<&str, u32> = counter(my_list.into_iter());

    assert_eq!(count.get(&"a"), Some(&3u32));
    assert_eq!(count.get(&"b"), Some(&2u32));
    assert_eq!(count.get(&"cd"), Some(&1u32));
    assert_eq!(count.get(&"e"), None);
}
