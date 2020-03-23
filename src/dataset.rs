use randomx4r_sys::*;
use std::sync::Arc;
use std::thread;
use std::vec::Vec;

use crate::cache::*;
use crate::error::*;
use crate::flags::*;

pub struct RandomxDataset {
    pub(crate) dataset: *mut randomx_dataset,
}

impl RandomxDataset {
    pub fn new(flags: RandomxFlags, key: &[u8], num_threads: u8) -> Result<Self, RandomxError> {
        assert!(num_threads > 0);

        let cache = RandomxCache::new(flags, key)?;
        let dataset = unsafe { randomx_alloc_dataset(flags.bits()) };

        if dataset.is_null() {
            return Err(RandomxError::DatasetAllocError);
        }

        let mut dataset = RandomxDataset { dataset };

        let count = unsafe { randomx_dataset_item_count() };

        if num_threads == 1 {
            unsafe {
                randomx_init_dataset(dataset.dataset, cache.cache, 0, count);
            }
        } else {
            let mut handles = Vec::new();
            let cache_arc = Arc::new(cache);
            let dataset_arc = Arc::new(dataset);

            let size = count / num_threads as u64;
            let last = count % num_threads as u64;
            let mut start = 0;

            for i in 0..num_threads {
                let cache = cache_arc.clone();
                let dataset = dataset_arc.clone();
                let mut this_size = size;
                if i == num_threads - 1 {
                    this_size += last;
                }
                let this_start = start;

                handles.push(thread::spawn(move || unsafe {
                    randomx_init_dataset(dataset.dataset, cache.cache, this_start, this_size);
                }));

                start += this_size;
            }

            for handle in handles {
                let _ = handle.join();
            }

            dataset = match Arc::try_unwrap(dataset_arc) {
                Ok(dataset) => dataset,
                Err(_) => return Err(RandomxError::DatasetAllocError),
            };
        }

        Ok(dataset)
    }
}

impl Drop for RandomxDataset {
    fn drop(&mut self) {
        unsafe { randomx_release_dataset(self.dataset) }
    }
}

unsafe impl Send for RandomxDataset {}

unsafe impl Sync for RandomxDataset {}
