use std::{any::TypeId, hash::Hash, marker::PhantomData, mem, sync::atomic::AtomicUsize};

use ahash::AHashSet;
use dashmap::DashMap;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

pub const fn impossible_heap_elem_id<O>() -> HeapElemId<O> {
    HeapElemId {
        owner: PhantomData,
        id: usize::MAX,
        bucket_idx: usize::MAX,
    }
}

pub struct HeapElemId<O> {
    owner: PhantomData<O>,
    id: usize,
    bucket_idx: usize,
}

impl<O> PartialEq for HeapElemId<O> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<O> Eq for HeapElemId<O> {}

impl<O> Hash for HeapElemId<O> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<O> Default for HeapElemId<O> {
    fn default() -> Self {
        Self {
            owner: PhantomData,
            id: Default::default(),
            bucket_idx: Default::default(),
        }
    }
}

impl<O> Clone for HeapElemId<O> {
    fn clone(&self) -> Self {
        Self {
            owner: self.owner,
            id: self.id,
            bucket_idx: self.bucket_idx,
        }
    }
}

impl<O> Copy for HeapElemId<O> {}

#[derive(PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
struct BucketIdx {
    idx: usize,
}

struct HeapKeyAlloc<O> {
    owner: PhantomData<O>,
    next_id: AtomicUsize,
}

impl<O> HeapKeyAlloc<O> {
    fn new(owner: PhantomData<O>) -> Self {
        Self {
            owner,
            next_id: AtomicUsize::new(0),
        }
    }

    fn next_id(&mut self) -> usize {
        self.next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    fn reserve_ids(&mut self, n: usize) -> (usize, usize) {
        let first = self
            .next_id
            .fetch_add(n, std::sync::atomic::Ordering::Relaxed);
        let last = first + n;

        return (first, last);
    }

    fn n_new_keys(&mut self, buckets: &[usize]) -> Vec<HeapElemId<O>> {
        let (first, last) = self.reserve_ids(buckets.len());

        (first..last)
            .zip(buckets.iter())
            .map(|(id, &bucket_idx)| HeapElemId {
                owner: self.owner,
                id,
                bucket_idx,
            })
            .collect()
    }

    fn new_key(&mut self, bucket_idx: usize) -> HeapElemId<O> {
        let id = self.next_id();

        HeapElemId {
            owner: self.owner,
            id,
            bucket_idx,
        }
    }
}

pub struct Heap<O, V> {
    owner: PhantomData<O>,
    key_allocator: HeapKeyAlloc<O>,

    vacant_idxs: Vec<BucketIdx>,
    buckets: Vec<Option<V>>,
    valid_ids: AHashSet<HeapElemId<O>>,
}

impl<O, V> Heap<O, V> {
    pub fn new() -> Self {
        Self::with_capacity(4)
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            owner: PhantomData,
            key_allocator: HeapKeyAlloc::new(PhantomData),
            vacant_idxs: Vec::new(),
            buckets: Vec::with_capacity(cap),
            valid_ids: AHashSet::with_capacity(cap),
        }
    }

    pub fn alloc(&mut self) -> HeapElemId<O> {
        let bucket_idx = if let Some(vacant_bucket_idx) = self.vacant_idxs.pop() {
            vacant_bucket_idx
        } else {
            let new_bucket_idx = BucketIdx {
                idx: self.buckets.len(),
            };
            self.buckets.push(None);
            new_bucket_idx
        };

        let key = self.key_allocator.new_key(bucket_idx.idx);
        self.valid_ids.insert(key);
        key
    }

    pub(crate) fn alloc_n(&mut self, n: usize) -> Vec<HeapElemId<O>> {
        let buckets = (0..n).into_iter().map(|_| {
            if let Some(vacant_bucket_idx) = self.vacant_idxs.pop() {
                vacant_bucket_idx
            } else {
                let new_bucket_idx = BucketIdx {
                    idx: self.buckets.len(),
                };
                self.buckets.push(None);
                new_bucket_idx
            }.idx
        })
        .collect::<Vec<_>>();

        let keys = self.key_allocator.n_new_keys(&buckets);
        for key in &keys {
            self.valid_ids.insert(*key);
        }

        keys
    }

    pub fn get(&self, key: &HeapElemId<O>) -> &V {
        if !self.valid_ids.contains(&key) {
            panic!("SEGFAULT: No bucket bound to key {}", key.id);
        };

        let bucket = if let Some(elem) = self.buckets.get(key.bucket_idx) {
            elem.as_ref()
        } else {
            panic!(
                "READ FROM UNITIALIZED: Bucket {} does not exist",
                key.bucket_idx
            )
        };

        if let Some(elem) = bucket {
            elem
        } else {
            panic!("USE AFTER FREE: Read from empty bucket {}", key.bucket_idx)
        }
    }

    pub fn free(&mut self, key: &HeapElemId<O>) {
        self.valid_ids.remove(key);
    }
}

impl<O, T: TyId> Heap<O, T> {
    pub fn replace(&mut self, key: &HeapElemId<O>, value: T) {
        if !self.valid_ids.contains(&key) {
            panic!("SEGFAULT: No bucket bound to key {}", key.id);
        };

        let bucket = if let Some(elem) = self.buckets.get_mut(key.bucket_idx) {
            elem
        } else {
            panic!(
                "WRITE TO UNITIALIZED: Bucket {} does not exist",
                key.bucket_idx
            )
        };

        if let Some(old_val) = bucket {
            assert!(old_val.id() == value.id())
        }
        let _ = mem::replace(bucket, Some(value));
    }
}

pub trait TyId {
    fn id(&self) -> TypeId;
}
