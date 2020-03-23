use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum RandomxError {
    /// Occurs when allocating the RandomX cache fails.
    ///
    /// Reasons include:
    ///  * Memory allocation fails
    ///  * The JIT flag is set but the current platform does not support it
    ///  * An invalid or unsupported ARGON2 value is set
    CacheAllocError,

    /// Occurs when allocating a RandomX dataset fails.
    ///
    /// Reasons include:
    ///  * Memory allocation fails
    DatasetAllocError,

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
            RandomxError::DatasetAllocError => write!(f, "Failed to allocate datataset"),
            RandomxError::VmAllocError => write!(f, "Failed to create VM"),
        }
    }
}

impl Error for RandomxError {
    fn description(&self) -> &str {
        match *self {
            RandomxError::CacheAllocError => "Failed to allocate cache",
            RandomxError::DatasetAllocError => "Failed to allocate dataset",
            RandomxError::VmAllocError => "Failed to create VM",
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}
