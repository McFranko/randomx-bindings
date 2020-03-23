use randomx4r_sys::*;

use crate::cache::*;
use crate::error::*;
use crate::flags::*;

pub struct RandomxDataset {
    pub(crate) dataset: *mut randomx_dataset,
}

impl RandomxDataset {
    pub fn new(flags: RandomxFlags, key: &[u8]) -> Result<Self, RandomxError> {
        let cache = RandomxCache::new(flags, key)?;
        let dataset = unsafe { randomx_alloc_dataset(flags.bits()) };

        if dataset.is_null() {
            return Err(RandomxError::DatasetAllocError);
        }

        let count = unsafe { randomx_dataset_item_count() };

        unsafe {
            randomx_init_dataset(dataset, cache.cache, 0, count);
        }

        Ok(RandomxDataset { dataset })
    }
}

impl Drop for RandomxDataset {
    fn drop(&mut self) {
        unsafe { randomx_release_dataset(self.dataset) }
    }
}

unsafe impl Send for RandomxDataset {}
