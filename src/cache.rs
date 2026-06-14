//! General-purpose caching utilities.
//!
//! This module contains cache types and helper traits that are used across
//! multiple modules (e.g., `document`, `fonts::cmap`).

use std::collections::HashMap;
use std::sync::Mutex;

/// Extension trait for Mutex to recover from poisoned locks.
///
/// When a thread panics while holding a Mutex, the lock becomes "poisoned"
/// and the standard `.lock().unwrap()` would cascade panics. This trait
/// provides `.lock_or_recover()` which discards the poison flag and returns
/// the inner data, since the Mutexes in PdfDocument protect caches and
/// bookkeeping (not safety-critical invariants).
pub(crate) trait MutexExt<T> {
    /// Lock the mutex, recovering from poison if needed.
    fn lock_or_recover(&self) -> std::sync::MutexGuard<'_, T>;
}

impl<T> MutexExt<T> for Mutex<T> {
    fn lock_or_recover(&self) -> std::sync::MutexGuard<'_, T> {
        self.lock().unwrap_or_else(|poisoned| {
            log::debug!("Mutex was poisoned, recovering");
            poisoned.into_inner()
        })
    }
}

/// Entry-count-bounded cache with LRU-style eviction.
///
/// Used for caches where estimating per-entry byte size is impractical
/// (e.g., `Vec<TextSpan>`, `Vec<PdfImage>`). On insert, when at capacity,
/// the least-recently-used entry is evicted. `get()` promotes the accessed
/// key to the most-recently-used position so hot entries survive eviction.
pub(crate) struct BoundedEntryCache<K: Eq + std::hash::Hash + Copy, V> {
    map: HashMap<K, V>,
    insertion_order: std::collections::VecDeque<K>,
    max_entries: usize,
}

impl<K: Eq + std::hash::Hash + Copy, V> BoundedEntryCache<K, V> {
    pub(crate) fn new(max_entries: usize) -> Self {
        Self {
            map: HashMap::new(),
            insertion_order: std::collections::VecDeque::new(),
            max_entries,
        }
    }

    pub(crate) fn get(&mut self, key: &K) -> Option<&V> {
        if self.map.contains_key(key) {
            if let Some(pos) = self.insertion_order.iter().position(|k| k == key) {
                self.insertion_order.remove(pos);
            }
            self.insertion_order.push_back(*key);
        }
        self.map.get(key)
    }

    pub(crate) fn insert(&mut self, key: K, value: V) {
        use std::collections::hash_map::Entry;
        // On re-insert: replace value and promote to most-recently-used position
        // so LRU eviction order stays accurate.
        if let Entry::Occupied(mut e) = self.map.entry(key) {
            e.insert(value);
            if let Some(pos) = self.insertion_order.iter().position(|k| k == &key) {
                self.insertion_order.remove(pos);
            }
            self.insertion_order.push_back(key);
            return;
        }
        // Evict oldest entries if at capacity
        while self.map.len() >= self.max_entries && !self.insertion_order.is_empty() {
            if let Some(old_key) = self.insertion_order.pop_front() {
                self.map.remove(&old_key);
            }
        }
        self.map.insert(key, value);
        self.insertion_order.push_back(key);
    }

    pub(crate) fn clear(&mut self) {
        self.map.clear();
        self.insertion_order.clear();
    }

    pub(crate) fn len(&self) -> usize {
        self.map.len()
    }
}

/// Byte-budget-bounded cache with LRU eviction.
///
/// Each entry carries its byte size. On insert, the least-recently-used
/// entries are evicted until the new entry fits within `max_bytes`; `get()`
/// promotes the accessed key to most-recently-used. Use this (rather than
/// [`BoundedEntryCache`], which bounds only entry *count*) for large,
/// variable-size values such as decoded images, where a fixed entry count
/// cannot bound memory.
///
/// LRU is the right policy for a streaming render workload — e.g. a RIP
/// rasterising thousands of pages that reuse a working set of recent images:
/// the active images stay resident while stale ones age out, so a job whose
/// total image bytes exceed the budget still gets near-full reuse of its hot
/// set.
pub(crate) struct ByteBoundedCache<K: Eq + std::hash::Hash + Copy, V> {
    map: HashMap<K, (V, usize)>,
    /// Front = least-recently-used, back = most-recently-used.
    lru: std::collections::VecDeque<K>,
    cur_bytes: usize,
    max_bytes: usize,
}

impl<K: Eq + std::hash::Hash + Copy, V> ByteBoundedCache<K, V> {
    pub(crate) fn new(max_bytes: usize) -> Self {
        Self {
            map: HashMap::new(),
            lru: std::collections::VecDeque::new(),
            cur_bytes: 0,
            max_bytes,
        }
    }

    pub(crate) fn get(&mut self, key: &K) -> Option<&V> {
        if !self.map.contains_key(key) {
            return None;
        }
        if let Some(pos) = self.lru.iter().position(|k| k == key) {
            self.lru.remove(pos);
        }
        self.lru.push_back(*key);
        self.map.get(key).map(|(v, _)| v)
    }

    pub(crate) fn insert(&mut self, key: K, value: V, size: usize) {
        // Replace an existing entry: drop its bytes and LRU position first.
        if let Some((_old, old_size)) = self.map.remove(&key) {
            self.cur_bytes = self.cur_bytes.saturating_sub(old_size);
            if let Some(pos) = self.lru.iter().position(|k| k == &key) {
                self.lru.remove(pos);
            }
        }
        // An item larger than the whole budget is never cached — storing it
        // would evict everything else and still not fit. The caller re-derives
        // it next time (correctness preserved, just no reuse).
        if size > self.max_bytes {
            return;
        }
        // Evict least-recently-used entries until the newcomer fits.
        while self.cur_bytes + size > self.max_bytes {
            match self.lru.pop_front() {
                Some(old) => {
                    if let Some((_v, s)) = self.map.remove(&old) {
                        self.cur_bytes = self.cur_bytes.saturating_sub(s);
                    }
                },
                None => break,
            }
        }
        self.map.insert(key, (value, size));
        self.lru.push_back(key);
        self.cur_bytes += size;
    }

    #[allow(dead_code)]
    pub(crate) fn clear(&mut self) {
        self.map.clear();
        self.lru.clear();
        self.cur_bytes = 0;
    }

    #[allow(dead_code)]
    pub(crate) fn len(&self) -> usize {
        self.map.len()
    }

    #[allow(dead_code)]
    pub(crate) fn bytes(&self) -> usize {
        self.cur_bytes
    }
}
