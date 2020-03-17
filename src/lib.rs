#[macro_use]
extern crate bitflags;
extern crate randomx4r_sys;

use std::convert::TryInto;
use std::error::Error;
use std::fmt;
use std::marker::PhantomData;
use std::ptr;

use randomx4r_sys::*;

#[derive(Debug)]
pub enum RandomxError {
    /// Occurs when allocating the RandomX cache fails.
    ///
    /// Reasons include:
    ///  * Memory allocation fails
    ///  * The JIT flag is set but the current platform does not support it
    ///  * An invalid or unsupported ARGON2 value is set
    CacheAllocError,

    /// Occurs when creating a VM fails.
    ///
    /// Reasons include:
    ///  * Scratchpad memory allocation fails
    ///  * Unsupported flags
    VmAllocError,
}

impl fmt::Display for RandomxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RandomxError::CacheAllocError => write!(f, "Failed to allocate cache"),
            RandomxError::VmAllocError => write!(f, "Failed to create VM"),
        }
    }
}

impl Error for RandomxError {
    fn description(&self) -> &str {
        match *self {
            RandomxError::CacheAllocError => "Failed to allocate cache",
            RandomxError::VmAllocError => "Failed to create VM",
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

bitflags! {
    /// Represents options that can be used when allocating the
    /// RandomX dataset or VM.
    pub struct RandomxFlags: u32 {
        /// Use defaults.
        const DEFAULT = randomx_flags_RANDOMX_FLAG_DEFAULT;

        /// Allocate memory in large pages.
        const LARGEPAGES = randomx_flags_RANDOMX_FLAG_LARGE_PAGES;

        /// The RandomX VM will use hardware accelerated AES.
        const HARDAES = randomx_flags_RANDOMX_FLAG_HARD_AES;

        /// The RandomX VM will use the full dataset.
        const FULLMEM = randomx_flags_RANDOMX_FLAG_FULL_MEM;

        /// The RandomX VM will use a JIT compiler.
        const JIT = randomx_flags_RANDOMX_FLAG_JIT;

        /// Make sure that JIT pages are never writable and executable
        /// at the same time.
        const SECURE = randomx_flags_RANDOMX_FLAG_SECURE;

        /// Use the SSSE3 extension to speed up Argon2 operations.
        const ARGON2_SSSE3 = randomx_flags_RANDOMX_FLAG_ARGON2_SSSE3;

        /// Use the AVX2 extension to speed up Argon2 operations.
        const ARGON2_AVX2 = randomx_flags_RANDOMX_FLAG_ARGON2_AVX2;

        /// Do not use SSSE3 or AVX2 extensions.
        const ARGON2 = randomx_flags_RANDOMX_FLAG_ARGON2;
    }
}

impl Default for RandomxFlags {
    /// Get the recommended flags to use on the current machine.
    ///
    /// Does not include any of the following flags:
    ///   * LARGEPAGES
    ///   * JIT
    ///   * SECURE
    fn default() -> RandomxFlags {
        // Explode if bits do not match up.
        unsafe { RandomxFlags::from_bits(randomx_get_flags()).unwrap() }
    }
}

/// Dataset cache for light-mode hashing.
pub struct RandomxCache {
    cache: *mut randomx_cache,
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

pub struct RandomxVm<'a> {
    vm: *mut randomx_vm,
    phantom: PhantomData<&'a RandomxCache>,
}

impl RandomxVm<'_> {
    pub fn new(flags: RandomxFlags, cache: &'_ RandomxCache) -> Result<RandomxVm, RandomxError> {
        let vm = unsafe { randomx_create_vm(flags.bits(), cache.cache, ptr::null_mut()) };

        if vm.is_null() {
            return Err(RandomxError::VmAllocError);
        }

        Ok(RandomxVm {
            vm,
            phantom: PhantomData,
        })
    }

    /// Calculate the RandomX hash of some data.
    ///
    /// ```no_run
    /// # // ^ no_run, this is already tested in the actual tests
    /// use randomx4r::*;
    /// let flags = RandomxFlags::default();
    /// let cache = RandomxCache::new(flags, "key".as_bytes())?;
    /// let vm = RandomxVm::new(flags, &cache)?;
    /// let hash = vm.hash("input".as_bytes());
    /// # Ok::<(), RandomxError>(())
    /// ```
    pub fn hash(&self, input: &[u8]) -> [u8; 32] {
        // TODO: Don't bother zeroing.
        let mut hash: [u8; 32] = [0; 32];

        unsafe {
            randomx_calculate_hash(
                self.vm,
                input.as_ptr() as *const std::ffi::c_void,
                input.len().try_into().unwrap(),
                hash.as_mut_ptr() as *mut std::ffi::c_void,
            );
        }

        hash
    }
}

impl Drop for RandomxVm<'_> {
    fn drop(&mut self) {
        unsafe { randomx_destroy_vm(self.vm) }
    }
}

unsafe impl Send for RandomxVm<'_> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_calc_hash() {
        let flags = RandomxFlags::default();
        let cache = RandomxCache::new(flags, "RandomX example key\0".as_bytes()).unwrap();
        let vm = RandomxVm::new(flags, &cache).unwrap();
        let hash = vm.hash("RandomX example input\0".as_bytes());
        let expected = [
            138, 72, 229, 249, 219, 69, 171, 121, 217, 8, 5, 116, 196, 216, 25, 84, 254, 106, 198,
            56, 66, 33, 74, 255, 115, 194, 68, 178, 99, 48, 183, 201,
        ];

        assert_eq!(expected, hash);
    }
}
