use core::{ops::Range, pin::Pin};

use crate::{
    acpi_tables,
    device::tpm2::tpm2::{PlattformClass, Tpm2Table},
    memory::vma::VmaType,
    process_manager,
};
use acpi::{AcpiTable, sdt::SdtHeader};
use alloc::borrow::ToOwned;
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

pub fn try_get_tpm2_via_acpi() -> Option<u64> {
    let Ok(tpm2_table_mapping) = acpi_tables().lock().find_table::<Tpm2Table>() else {
        return None;
    };

    let tpm2_table = tpm2_table_mapping.get();

    assert_eq!(tpm2_table.header().signature, acpi::sdt::Signature::TPM2);
    assert_eq!(tpm2_table.plattform_class(), Ok(PlattformClass::Client));
    assert_eq!(tpm2_table.start_method(), Ok(crate::device::tpm2::tpm2::StartMethodType::CrbInterface));

    Some(tpm2_table.get_control_area_address())
}
