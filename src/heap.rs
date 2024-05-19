use std::{collections::HashMap, hash::Hash, marker::PhantomData, mem, sync::atomic::AtomicUsize};

pub const fn impossible_heap_elem_id<O>() -> HeapElemId<O> {
    HeapElemId {
        owner: PhantomData,
        id: usize::MAX,
    }
}

pub struct HeapElemId<O> {
    owner: PhantomData<O>,
    id: usize,
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
        }
    }
}

impl<O> Clone for HeapElemId<O> {
    fn clone(&self) -> Self {
        Self {
            owner: self.owner,
            id: self.id,
        }
    }
}

impl<O> Copy for HeapElemId<O> {}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
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

    fn new_key(&mut self) -> HeapElemId<O> {
        let next_id = self.next_id();

        HeapElemId {
            owner: self.owner,
            id: next_id,
        }
    }
}

struct HeapKeyLookupMap<O> {
    bucket_to_key_map: HashMap<HeapElemId<O>, BucketIdx>,
    key_bucket_to_map: HashMap<BucketIdx, HeapElemId<O>>,
}

impl<O> HeapKeyLookupMap<O> {
    fn new() -> Self {
        Self {
            bucket_to_key_map: HashMap::new(),
            key_bucket_to_map: HashMap::new(),
        }
    }

    fn bind(&mut self, key: HeapElemId<O>, bucket_idx: BucketIdx) {
        self.bucket_to_key_map.insert(key, bucket_idx);
        self.key_bucket_to_map.insert(bucket_idx, key);
    }

    fn unbind(&mut self, key: &HeapElemId<O>) {
        let bucket_idx = self.bucket_idx(&key).copied().unwrap();

        self.bucket_to_key_map.remove(&key);
        self.key_bucket_to_map.remove(&bucket_idx);
    }

    fn bucket_idx(&self, key: &HeapElemId<O>) -> Option<&BucketIdx> {
        self.bucket_to_key_map.get(key)
    }
}

pub struct Heap<O, V> {
    owner: PhantomData<O>,
    key_allocator: HeapKeyAlloc<O>,

    vacant_idxs: Vec<BucketIdx>,
    buckets: Vec<Option<V>>,
    lookup_map: HeapKeyLookupMap<O>,
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
            lookup_map: HeapKeyLookupMap::new(),
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

        let key = self.key_allocator.new_key();
        self.lookup_map.bind(key, bucket_idx);

        key
    }

    pub fn get(&self, key: &HeapElemId<O>) -> &V {
        let bucket_idx = if let Some(bucket_idx) = self.lookup_map.bucket_idx(&key) {
            bucket_idx
        } else {
            panic!("USE AFTER FREE: No bucket bound to key {}", key.id)
        };

        let bucket = if let Some(elem) = self.buckets.get(bucket_idx.idx) {
            elem.as_ref()
        } else {
            panic!(
                "READ FROM UNITIALIZED: Bucket {} does not exist",
                bucket_idx.idx
            )
        };

        if let Some(elem) = bucket {
            elem
        } else {
            panic!("USE AFTER FREE: bucket {} is already freed", bucket_idx.idx)
        }
    }

    pub fn replace(&mut self, key: &HeapElemId<O>, value: V) {
        let bucket_idx = if let Some(bucket_idx) = self.lookup_map.bucket_idx(&key) {
            bucket_idx
        } else {
            panic!("USE AFTER FREE: No bucket bound to key {}", key.id)
        };

        let bucket = if let Some(elem) = self.buckets.get_mut(bucket_idx.idx) {
            elem
        } else {
            panic!(
                "WRITE TO UNITIALIZED: Bucket {} does not exist",
                bucket_idx.idx
            )
        };

        mem::replace(bucket, Some(value));
    }

    pub fn free(&mut self, key: &HeapElemId<O>) {
        let bucket_idx = if let Some(bucket_idx) = self.lookup_map.bucket_idx(&key) {
            bucket_idx
        } else {
            panic!("DOUBLE FREE: No bucket bound to key {}", key.id)
        };

        let bucket = if let Some(elem) = self.buckets.get(bucket_idx.idx) {
            elem.as_ref()
        } else {
            panic!(
                "DROP OF UNITIALIZED: Bucket {} does not exist",
                bucket_idx.idx
            )
        };

        if bucket.is_some() {
            self.buckets[bucket_idx.idx] = None;
            self.lookup_map.unbind(key)
        } else {
            panic!("DOUBLE FREE: bucket {} is already freed", bucket_idx.idx)
        }
    }
}
