use randomx_bindings_sys::*;
use std::convert::TryInto;

use crate::error::*;
use crate::flags::*;

/// Dataset cache for light-mode hashing.
pub struct RandomxCache {
    pub(crate) cache: *mut randomx_cache,
}

impl RandomxCache {
    pub fn new(flags: RandomxFlags, key: &[u8]) -> Result<Self, RandomxError> {
        let cache = unsafe { randomx_alloc_cache(flags.bits()) };

        if cache.is_null() {
            return Err(RandomxError::CacheAllocError);
        }

        unsafe {
            randomx_init_cache(
                cache,
                key.as_ptr() as *const std::ffi::c_void,
                key.len().try_into().unwrap(),
            );
        }

        Ok(RandomxCache { cache })
    }
}

impl Drop for RandomxCache {
    fn drop(&mut self) {
        unsafe { randomx_release_cache(self.cache) }
    }
}

unsafe impl Send for RandomxCache {}

unsafe impl Sync for RandomxCache {}
