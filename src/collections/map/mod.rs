mod bucket;
mod utils;

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hash, Hasher};

use self::bucket::Bucket;

/// Thread-Safe map implemented as hash table.
pub struct Map<K, V, H = RandomState> {
    hash_builder: H,
    buckets: Vec<Bucket<K, V>>,
}

impl<K, V> Map<K, V, RandomState>
where
    K: Hash + Eq + Copy,
    V: Clone,
{
    /// Creates an empty `Map`
    ///
    /// The map will allocate a default number of buckets.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use palladiumdb::Map;
    /// let map: Map<&str, i32> = Map::new();
    /// ```
    pub fn new() -> Self {
        Self::with_bucket_count(Self::DEFAULT_BUCKET_COUNT)
    }

    /// Creates an empty `Map` with a given bucket count.
    ///
    /// The map will have `bucket_count` buckets allocated.
    ///
    /// # Panics
    ///
    /// This function will panic if `bucket_count` is 0.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use palladiumdb::Map;
    /// let map: Map<&str, i32> = Map::with_bucket_count(32);
    /// ```
    pub fn with_bucket_count(bucket_count: usize) -> Self {
        if bucket_count == 0usize {
            panic!()
        }

        Self::with_hasher_and_bucket_count(RandomState::new(), bucket_count)
    }
}

impl<K, V, H> Map<K, V, H>
where
    K: Hash + Eq + Copy,
    V: Clone,
    H: BuildHasher,
{
    const DEFAULT_BUCKET_COUNT: usize = 19;

    /// Creates an empty `Map` with `bucket_count` buckets allocated, using
    /// `hash_builder` to hash the keys.
    ///
    /// The `hash_builder` passed should implement the [`BuildHasher`] trait
    /// to be useful, see its documentation for details
    ///
    /// # Panics
    ///
    /// This function will panic if `bucket_count` is 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use palladiumdb::Map;
    /// use std::collections::hash_map::RandomState;
    ///
    /// let s = RandomState::new();
    /// let map = Map::with_hasher_and_bucket_count(s,32);
    ///
    /// map.put(&"Two", 2);
    /// ```
    pub fn with_hasher_and_bucket_count(hash_builder: H, bucket_count: usize) -> Self {
        let mut buckets = Vec::with_capacity(bucket_count);
        buckets.resize_with(bucket_count, || Bucket::new());

        Map {
            hash_builder,
            buckets,
        }
    }

    /// Creates an empty `Map` which will use the given hash builder to hash
    /// keys.
    ///
    /// The created `Map` has the default number of buckets allocated.
    ///
    /// The `hash_builder` passed should implement the [`BuildHasher`] trait
    /// to be useful, see its documentation for details.
    ///
    /// # Examples
    ///
    /// ```
    /// use palladiumdb::Map;
    /// use std::collections::hash_map::RandomState;
    ///
    /// let s = RandomState::new();
    /// let map = Map::with_hasher(s);
    ///
    /// map.put(&"Two",2)
    /// ```
    pub fn with_hasher(hash_builder: H) -> Self {
        Self::with_hasher_and_bucket_count(hash_builder, Self::DEFAULT_BUCKET_COUNT)
    }

    fn get_bucket(&self, key: &K) -> &Bucket<K, V> {
        let mut hasher = self.hash_builder.build_hasher();
        key.hash(&mut hasher);
        let hash = hasher.finish() as usize;
        let bucket_index = hash % self.buckets.len();

        &self.buckets[bucket_index]
    }

    /// Establishes a key value mapping for the key value pair.
    ///
    /// Creates a new key value pair in the `Map` if the mapping
    /// didn't exist before, Otherwise overwrites the old mapping with
    /// the new value.
    ///
    /// # Examples
    ///
    /// ```
    /// use palladiumdb::Map;
    ///
    /// let map = Map::new();
    ///
    /// map.put(&"First", 1);
    /// map.put(&"Two", 2);
    /// map.put(&"First", 0);
    ///
    /// assert_eq!(map.get(&"Two"), Some(2));
    /// assert_eq!(map.get(&"First"), Some(0));
    /// ```
    pub fn put(&self, key: &K, value: V) {
        self.get_bucket(key).put(key, value)
    }

    /// Returns the value corresponding to the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use palladiumdb::Map;
    ///
    /// let map = Map::new();
    /// map.put(&1, 'a');
    /// assert_eq!(map.get(&1), Some('a'));
    /// assert_eq!(map.get(&2), None);
    /// ```
    pub fn get(&self, key: &K) -> Option<V> {
        self.get_bucket(key).get(key)
    }

    /// Erases the value associated with `key`, if present,
    /// from the `Map`.
    ///
    /// # Examples
    ///
    /// ```
    /// use palladiumdb::Map;
    ///
    /// let map = Map::new();
    /// map.put(&"MyNumber", 35642);
    ///
    /// map.unmap(&"MyNumber");
    /// map.unmap(&"TheBestNumber");
    ///
    /// assert_eq!(map.get(&"MyNumber"), None);
    /// assert_eq!(map.get(&"TheBestNumber"), None);
    /// ```
    pub fn unmap(&self, key: &K) {
        self.get_bucket(key).unmap(key);
    }
}

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use super::Map;

    #[test]
    fn test_map_consistency() {
        let map = Arc::new(Map::new());
        use std::time::Duration;
        // random sleep durations
        let d1 = Duration::from_millis(276);
        let d2 = Duration::from_millis(532);

        let m = Arc::clone(&map);
        let put_thread_1 = std::thread::spawn(move || {
            m.put(&1, 2);
            std::thread::sleep(d1);
            m.put(&2, 3);
            std::thread::sleep(d2);
            m.put(&3, 4);
        });

        let m = Arc::clone(&map);
        let put_thread_2 = std::thread::spawn(move || {
            m.put(&5, 6);
            std::thread::sleep(d1);
            m.put(&7, 8);
            std::thread::sleep(d2);
            m.put(&9, 10);
        });

        put_thread_1.join().unwrap();
        put_thread_2.join().unwrap();

        let m = Arc::clone(&map);
        let get_thread_task = move || {
            assert_eq!(m.get(&1), Some(2));
            std::thread::sleep(d1);

            assert_eq!(m.get(&2), Some(3));
            std::thread::sleep(d2);

            assert_eq!(m.get(&3), Some(4));
            std::thread::sleep(d1);

            assert_eq!(m.get(&5), Some(6));
            std::thread::sleep(d2);

            assert_eq!(m.get(&7), Some(8));
            std::thread::sleep(d1);

            assert_eq!(m.get(&9), Some(10));
        };

        let get_thread_1 = std::thread::spawn(get_thread_task.clone());
        let get_thread_2 = std::thread::spawn(get_thread_task);

        get_thread_1.join().unwrap();
        get_thread_2.join().unwrap();
    }
}
