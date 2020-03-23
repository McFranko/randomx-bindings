use randomx4r_sys::*;
use std::convert::TryInto;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr;

use crate::cache::*;
use crate::dataset::*;
use crate::error::*;
use crate::flags::*;

pub struct RandomxVm<'a, T: 'a> {
    vm: *mut randomx_vm,
    phantom: PhantomData<&'a T>,
}

impl RandomxVm<'_, RandomxCache> {
    pub fn new(flags: RandomxFlags, cache: &'_ RandomxCache) -> Result<Self, RandomxError> {
        if flags.contains(RandomxFlags::FULLMEM) {
            return Err(RandomxError::VmAllocError);
        }

        let vm = unsafe { randomx_create_vm(flags.bits(), cache.cache, ptr::null_mut()) };

        if vm.is_null() {
            return Err(RandomxError::VmAllocError);
        }

        Ok(RandomxVm {
            vm,
            phantom: PhantomData,
        })
    }
}

impl RandomxVm<'_, RandomxDataset> {
    pub fn new_fast(
        flags: RandomxFlags,
        dataset: &'_ RandomxDataset,
    ) -> Result<Self, RandomxError> {
        if !flags.contains(RandomxFlags::FULLMEM) {
            return Err(RandomxError::VmAllocError);
        }

        let vm = unsafe { randomx_create_vm(flags.bits(), ptr::null_mut(), dataset.dataset) };

        if vm.is_null() {
            return Err(RandomxError::VmAllocError);
        }

        Ok(RandomxVm {
            vm,
            phantom: PhantomData,
        })
    }
}

impl<T> RandomxVm<'_, T> {
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
    pub fn hash(&self, input: &[u8]) -> [u8; RANDOMX_HASH_SIZE as usize] {
        let mut hash = MaybeUninit::<[u8; RANDOMX_HASH_SIZE as usize]>::uninit();

        unsafe {
            randomx_calculate_hash(
                self.vm,
                input.as_ptr() as *const std::ffi::c_void,
                input.len().try_into().unwrap(),
                hash.as_mut_ptr() as *mut std::ffi::c_void,
            );

            hash.assume_init()
        }
    }
}

impl<T> Drop for RandomxVm<'_, T> {
    fn drop(&mut self) {
        unsafe { randomx_destroy_vm(self.vm) }
    }
}

unsafe impl<T> Send for RandomxVm<'_, T> {}

#[cfg(test)]
mod tests {
    use crate::*;

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

    #[test]
    fn can_calc_hash_fast() {
        let flags = RandomxFlags::default() | RandomxFlags::FULLMEM;
        // TODO: Get system thread count, or use an environment variable?
        let dataset = RandomxDataset::new(flags, "RandomX example key\0".as_bytes(), 1).unwrap();
        let vm = RandomxVm::new_fast(flags, &dataset).unwrap();
        let hash = vm.hash("RandomX example input\0".as_bytes());
        let expected = [
            138, 72, 229, 249, 219, 69, 171, 121, 217, 8, 5, 116, 196, 216, 25, 84, 254, 106, 198,
            56, 66, 33, 74, 255, 115, 194, 68, 178, 99, 48, 183, 201,
        ];

        assert_eq!(expected, hash);
    }
}
