use std::collections::hash_map::RandomState;
use std::hash::Hasher;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::sync::RwLockWriteGuard;

struct BucketValue<K, V>(K, V);
type BucketData<K, V> = Vec<BucketValue<K, V>>;

enum LockWrapper<'a, T> {
    Read(RwLockReadGuard<'a, T>),
    Write(RwLockWriteGuard<'a, T>),
}

impl<'a, T> Deref for LockWrapper<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            LockWrapper::Read(read_guard) => read_guard.deref(),
            LockWrapper::Write(write_guard) => write_guard.deref(),
        }
    }
}

impl<'a, T> DerefMut for LockWrapper<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            LockWrapper::Read(_) => panic!(),
            LockWrapper::Write(write_gaurd) => write_gaurd.deref_mut(),
        }
    }
}

struct Bucket<K, V> {
    data: RwLock<BucketData<K, V>>,
}

impl<K, V> Bucket<K, V>
where
    K: Hash + Eq + Copy,
    V: Copy,
{
    pub fn new() -> Self {
        Bucket {
            data: RwLock::new(Vec::new()),
        }
    }

    fn find_entry_for<'gaurd>(
        key: &K,
        data: &'gaurd LockWrapper<Vec<BucketValue<K, V>>>,
    ) -> Option<(usize, &'gaurd BucketValue<K, V>)> {
        data.iter()
            .enumerate()
            .find(|(_, BucketValue(elem_key, _))| *elem_key == *key)
    }

    pub fn get(&self, key: &K, default_value: V) -> V {
        let gaurd = LockWrapper::Read(self.data.read().unwrap());
        match Self::find_entry_for(key, &gaurd) {
            Some((_, &BucketValue(_, value))) => value,
            None => default_value,
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

use std::hash::{BuildHasher, Hash};

pub struct Map<K, V, H = RandomState>
where
    K: Hash + Eq,
    H: BuildHasher,
{
    hash_builder: H,
    buckets: Vec<Bucket<K, V>>,
}

impl<K, V> Map<K, V, RandomState>
where
    K: Hash + Eq + Copy,
    V: Copy,
{
    pub fn new(number_of_buckets: usize) -> Self {
        let mut buckets = Vec::with_capacity(number_of_buckets);
        buckets.resize_with(number_of_buckets, || Bucket::new());

        Map {
            hash_builder: RandomState::new(),
            buckets,
        }
    }
}

impl<K, V, H> Map<K, V, H>
where
    K: Hash + Eq + Copy,
    V: Default + Copy,
    H: BuildHasher,
{
    fn get_bucket(&self, key: &K) -> &Bucket<K, V> {
        let mut hasher = self.hash_builder.build_hasher();
        key.hash(&mut hasher);
        let hash = hasher.finish() as usize;
        let bucket_index = hash % self.buckets.len();
        &self.buckets[bucket_index]
    }

    pub fn put(&self, key: &K, value: V) {
        self.get_bucket(key).put(key, value)
    }

    pub fn get(&self, key: &K) -> V {
        self.get_bucket(key).get(key, V::default())
    }
}

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use crate::Map;

    #[test]
    fn constructs() {
        let _ = Map::<i32, i32>::new(19);
    }

    #[test]
    fn test_map_consistency() {
        let map = Arc::new(Map::new(19));
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
            assert_eq!(m.get(&1), 2);
            std::thread::sleep(d1);

            assert_eq!(m.get(&2), 3);
            std::thread::sleep(d2);

            assert_eq!(m.get(&3), 4);
            std::thread::sleep(d1);

            assert_eq!(m.get(&5), 6);
            std::thread::sleep(d2);

            assert_eq!(m.get(&7), 8);
            std::thread::sleep(d1);

            assert_eq!(m.get(&9), 10);
        };

        let get_thread_1 = std::thread::spawn(get_thread_task.clone());
        let get_thread_2 = std::thread::spawn(get_thread_task);

        get_thread_1.join().unwrap();
        get_thread_2.join().unwrap();
    }
}
