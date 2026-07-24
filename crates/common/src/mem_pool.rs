use crate::types::PoolIdx;

pub const POOL_IDX_NULL: PoolIdx = u32::MAX;

struct ObjectBlock<T> {
    object: T,
    is_free: bool,
}

pub struct MemPool<T> {
    store: Vec<ObjectBlock<T>>,
    next_free_index: usize,
    in_use:usize,
}

impl<T: Default> MemPool<T> {
    pub fn new(capacity: usize) -> Self {
        let mut store = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            store.push(ObjectBlock {
                object: T::default(),
                is_free: true,
            });
        }

        Self {
            store,
            next_free_index: 0,
            in_use:0
        }
    }

    /// Equivalent of the C++ allocate(): O(1) amortized, no heap allocation.
    /// Returns a handle (index) instead of a pointer.
    pub fn allocate(&mut self) -> PoolIdx {
        let block = &mut self.store[self.next_free_index];
        assert!(
            block.is_free,
            "Expected free ObjectBlock at index {}",
            self.next_free_index
        );
        block.is_free = false;
        block.object = T::default();
        let idx = self.next_free_index as PoolIdx;
        self.update_next_free_index();
        self.in_use +=1;
        idx
    }

    pub fn deallocate(&mut self, idx: PoolIdx) {
        let block = &mut self.store[idx as usize];
        assert!(!block.is_free, "Expected in-use ObjectBlock at index {idx}");
        block.is_free = true;
        self.in_use -=1;
        if (idx as usize) < self.next_free_index {
            self.next_free_index = idx as usize;
        }
    }

    #[inline]
    pub fn get(&self, idx: PoolIdx) -> &T {
        &self.store[idx as usize].object
    }

    pub fn pool_usage_count(&self) -> usize {
        self.in_use
    }

    #[inline]
    pub fn get_mut(&mut self, idx: PoolIdx) -> &mut T {
        &mut self.store[idx as usize].object
    }

    fn update_next_free_index(&mut self) {
        let initial = self.next_free_index;
        loop {
            self.next_free_index = (self.next_free_index + 1) % self.store.len();

            if self.store[self.next_free_index].is_free {
                break;
            }

            assert!(self.next_free_index != initial, "Memory pool out of space.");
        }
    }
}
