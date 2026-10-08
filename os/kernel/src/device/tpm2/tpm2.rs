use core::{
    ops::Range,
    ptr::{read_volatile, write_volatile},
};

use acpi::{
    AcpiTable,
    sdt::{SdtHeader, Signature},
};
use log::{debug, info};
use num_enum::TryFromPrimitive;
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};
use tpm2::{Command, commands::GetRandom};
use x86_64::structures::paging::PageTableFlags;

use crate::{
    acpi_tables,
    device::tpm2::{
        crb::{LocalityControl, get_locality_0_regs},
        tpm2::TpmError::{InvalidPlattformClass, InvalidStartMethod},
        util::{mmio_map, try_get_tpm2_via_acpi},
    },
    memory::vma::VmaType,
    process_manager,
};

pub fn init_tpm2() {
    // Try finding the TPM2 via ACPI:

    let maybe_acpi_mmio_start_address = try_get_tpm2_via_acpi();
    match maybe_acpi_mmio_start_address {
        None => info!("No tpm2 mmio start address in acpi info"),
        Some(addr) => info!("Found mmio start address in acpi: {:#?}", addr),
    }

    // QEMU: The CRB interface makes a memory mapped IO region
    // in the area 0xfed40000-0xfed40fff (1 locality)
    // available to the guest operating system.
    let mmio_range = Range {
        start: 0xfed40000,
        end: 0xfed40fff,
    };
    mmio_map(mmio_range.clone(), "tpm2");

    let loc0_regs = get_locality_0_regs(mmio_range.start);

    loc0_regs
        .loc_ctrl
        .modify(LocalityControl::requestAccess::SET + LocalityControl::Relinquish::SET);

    info!("Asked TPM2 to give access to Locality 0");

    info!("LOC_STATE: {:#?}", loc0_regs.loc_state.extract().debug());
    info!("LOC_STATUS: {:#?}", loc0_regs.loc_sts.extract().debug());
    info!("INTF_ID: {:#?}", loc0_regs.crb_intf_id.extract().debug());
    info!("CRB_STATUS: {:#?}", loc0_regs.crb_ctrl_sts.extract().debug());
    let raw_crb_status: u32 = loc0_regs.crb_ctrl_sts.extract().into();
    info!("CRB_STATUS raw: {:#?}", raw_crb_status);

    panic!("Time to shut down!");
}

pub fn init_tpm2_old() {
    // Attempt to find TPM
    let tpm_header = acpi_tables()
        .lock()
        .headers()
        .find(|hdr| hdr.signature == acpi::sdt::Signature::TCPA || hdr.signature == acpi::sdt::Signature::TPM2)
        .expect("No TPM found in ACPI tables");

    info!("TPM found with sig {}", tpm_header.signature);

    let tpm2_mapping = acpi_tables()
        .lock()
        .find_table::<Tpm2Table>()
        .expect("TPM2 not available, despite tpm signature being found in acpi tables");

    let tpm2 = tpm2_mapping.get();
    info!("TPM Info: {:#?}", tpm2);
    // outputs the following on QEMU:
    // Tpm2Table {
    //     header: SdtHeader {
    //         signature: "TPM2",
    //         length: 76,
    //         revision: 4,
    //         checksum: 240,
    //         oem_id: [
    //             66,
    //             79,
    //             67,
    //             72,
    //             83,
    //             32,
    //         ],
    //         oem_table_id: [
    //             66,
    //             88,
    //             80,
    //             67,
    //             32,
    //             32,
    //             32,
    //             32,
    //         ],
    //         oem_revision: 1,
    //         creator_id: 1129338946,
    //         creator_revision: 1,
    //     },
    //     plattform_class: 0,
    //     reserved: 0,
    //     control_area_address: 0,
    //     start_method: 6,
    //     start_method_params: 0,
    // }

    // Map control registers into kernel memory:
    let mmio_start = if tpm2.control_area_address != 0 {
        tpm2.control_area_address
    } else {
        // Default on at least QEMU
        0xfed40000
    };
    let mmio_end = mmio_start + 0x5000;

    let process = process_manager().read().kernel_process().expect("Failed to get kernel process");
    process.virtual_address_space.kernel_map_devm_identity(
        mmio_start,
        mmio_end,
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE,
        VmaType::DeviceMemory,
        "tpm2",
    );
    info!(" mapped TPM2 MMIO region into kernel ({} - {})", mmio_start, mmio_end);
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PlattformClass {
    Client = 0,
    Server = 1,
}

/// As defined in
/// https://trustedcomputinggroup.org/wp-content/uploads/TCG_ACPIGeneralSpecification_v1.20_r8.pdf#%5B%7B%22num%22%3A71%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C69%2C530%2C0%5D
#[derive(Copy, Clone, Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u32)]
pub enum StartMethodType {
    ///  Uses the ACPI Start method.
    AcpiStartMethod = 2,
    /// Reserved for the Memory mapped I/O Interface (TIS 1.2+Cancel).
    MemMappedIO = 6,
    CrbInterface = 7,
    CrbInterfaceWithAcpiStartMethod = 8,
    ///  Uses the Command Response Buffer Interface with ARM Secure Monitor Call (SMC)
    CrbArmSmcHvc = 11,
    FifoI2c = 12,
    CrbAmdMailbox = 13,
    CrbArmFwFa = 15,
}

#[derive(Debug, PartialEq)]
pub enum TpmError {
    InvalidStartMethod,
    InvalidPlattformClass,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Tpm2Table {
    /// The contents of the HPET's 'General Capabilities and ID register'
    header: SdtHeader,

    /// len=2, offset=36 | 0 for client platforms. 1 for server platforms
    plattform_class: u16, // maps to PlattformClass

    ///  len=2, offset=38 | should always be 0
    reserved: u16,

    /// len=8, offset=40 | Physical address of the Control Area.
    ///
    /// The Control Area
    /// contains status registers and the location of the memory
    /// buffers for communicating with the device. The area may be
    /// in either TPM 2.0 device memory or in memory reserved by
    /// the system during boot. Interfaces that do not require the
    /// Control Area set this value to zero
    control_area_address: u64,

    /// len=4, offset=48 | Start Method
    ///
    /// The Start Method selector determines which mechanism the
    /// device driver uses to notify the TPM 2.0 device that a
    /// command is available for processing. This field may contain
    /// one of the values specified in Table 8.
    start_method: u32, // maps to StartMethodType

    /// len=4-12, offset=52
    pub start_method_params: u32,
}

/// ### Safety: Implementation properly represents a valid HPET table.
unsafe impl AcpiTable for Tpm2Table {
    const SIGNATURE: Signature = Signature::TPM2;

    fn header(&self) -> &SdtHeader {
        &self.header
    }
}

impl Tpm2Table {
    pub fn start_method(&self) -> Result<StartMethodType, TpmError> {
        StartMethodType::try_from(self.start_method).map_err(|_| TpmError::InvalidStartMethod)
    }

    pub fn plattform_class(&self) -> Result<PlattformClass, TpmError> {
        match self.plattform_class {
            0 => Ok(PlattformClass::Client),
            1 => Ok(PlattformClass::Server),
            _ => Err(InvalidPlattformClass),
        }
    }

    pub fn get_control_area_address(&self) -> u64 {
        self.control_area_address
    }
}

pub enum Tpm2Locality {
    Locality0 = 0x0000,
    Locality1 = 0x1000,
    Locality2 = 0x2000,
    Locality3 = 0x3000,
    Locality4 = 0x4000,
}
