use std::sync::RwLock;

struct BucketValue<K, V>(K, V);
type BucketData<K, V> = Vec<BucketValue<K, V>>;

/// Hash bucket within the hash table, for storing
/// entries with clashing hash values.
pub struct Bucket<K, V> {
    // a multi-read, single-write wrapper
    data: RwLock<BucketData<K, V>>,
}

use super::utils::LockWrapper;

impl<K, V> Bucket<K, V>
where
    K: Eq + Copy,
    V: Clone,
{
    pub fn new() -> Self {
        Bucket {
            data: RwLock::new(Vec::new()),
        }
    }

    /// Searches and returns the first [`BucketValue`] within this bucket's
    /// data list that has the given `key`, along with an index of the
    /// returned [`BucketValue`].
    ///
    /// Both the index and [`BucketValue`] are returned together in a tuple
    ///
    /// # Arguments
    ///
    /// * `key`     - reference to the key
    /// * `data`    - a [`LockWrapper`] to the data that contains the
    ///               [`BucketValue`] to search in. The Lock can be both
    ///               a read lock as well as a write lock.
    ///
    /// # Returns
    ///
    /// An [`Option`]al tuple of the form `(index, &BucketValue)` where
    /// `index` is the current index of the [`BucketValue`] returned.
    fn find_entry_for<'gaurd>(
        key: &K,
        data: &'gaurd LockWrapper<Vec<BucketValue<K, V>>>,
    ) -> Option<(usize, &'gaurd BucketValue<K, V>)> {
        data.iter()
            .enumerate()
            .find(|(_, BucketValue(elem_key, _))| *elem_key == *key)
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let gaurd = LockWrapper::Read(self.data.read().unwrap());
        match Self::find_entry_for(key, &gaurd) {
            Some((_, BucketValue(_, value))) => Some(value.clone()),
            None => None,
        }
    }

    pub fn put(&self, key: &K, value: V) {
        let mut gaurd = LockWrapper::Write(self.data.write().unwrap());
        match Self::find_entry_for(key, &gaurd) {
            None => gaurd.push(BucketValue(*key, value)),
            Some((index, _)) => gaurd.get_mut(index).unwrap().1 = value,
        }
    }

    pub fn unmap(&self, key: &K) {
        let mut gaurd = LockWrapper::Write(self.data.write().unwrap());
        if let Some((index, _)) = Self::find_entry_for(key, &gaurd) {
            gaurd.swap_remove(index);
        }
    }
}
