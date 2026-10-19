use core::ops::Range;

use crate::{memory::vma::VmaType, process_manager};
use x86_64::structures::paging::PageTableFlags;

/// Map a physical address range into the same address range in virtual kernel space
pub fn mmio_map(addr_range: Range<u64>, tag: &str) {
    let process = process_manager().read().kernel_process().expect("Failed to get kernel process");
    process.virtual_address_space.kernel_map_devm_identity(
        addr_range.start,
        addr_range.end,
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE,
        VmaType::DeviceMemory,
        tag,
    );
}
