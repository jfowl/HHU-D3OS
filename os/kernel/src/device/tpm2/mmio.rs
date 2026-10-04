///
/// Stolen from https://git.hhu.de/bsinfo/thesis/ma-narei109/-/blob/master/project/D3OS/os/kernel/src/device/nvme/tock_registers.rs?ref_type=heads
///
use core::ops::Deref;
use core::ptr::NonNull;

/// Deref Wrapper for the register struct returned by the register_struct! macro
/// This is used so that register structs can be accessed safely and don't need to be dereferenced in an unsafe block
/// copied from TockOs' static_ref: https://github.com/tock/tock/blob/master/kernel/src/utilities/static_ref.rs
pub struct MMIO<T> {
    ptr: NonNull<T>,
}

unsafe impl<T> Sync for MMIO<T> {}
unsafe impl<T> Send for MMIO<T> {}

impl<T> MMIO<T> {
    pub fn new(ptr: *const T) -> MMIO<T> {
        unsafe {
            MMIO {
                ptr: NonNull::new_unchecked(ptr.cast_mut()),
            }
        }
    }
}

impl<T> Clone for MMIO<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for MMIO<T> {}

impl<T> Deref for MMIO<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}
