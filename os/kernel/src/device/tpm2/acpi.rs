use acpi::{
    AcpiTable,
    sdt::{SdtHeader, Signature},
};
use log::info;
use num_enum::TryFromPrimitive;

use crate::{
    acpi_tables,
    device::tpm2::tpm2::TpmError::{self, InvalidPlattformClass},
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PlattformClass {
    Client = 0,
    Server = 1,
}

/// As defined in the TCG ACPI spec
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

pub fn try_get_crb_tpm2_via_acpi() -> Option<u64> {
    let Ok(tpm2_table_mapping) = acpi_tables().lock().find_table::<Tpm2Table>() else {
        info!("No {:#?} found via ACPI", core::any::type_name::<Tpm2Table>());
        return None;
    };
    let tpm2_table = tpm2_table_mapping.get();

    let has_tpm2_header = tpm2_table.header().signature == acpi::sdt::Signature::TPM2;
    if !has_tpm2_header {
        info!(
            "TPM2 ACPI table found, but header signature is {:#?}, instead of expected TPM2 sig {:#?}",
            tpm2_table.header().signature,
            acpi::sdt::Signature::TPM2
        );
    }

    // The TPM exposed from swtpm via QEMU (like the Lenovo-TPM in my ThinkPad) uses a revision of 4,
    // from the 2014 version of the spec (TCG_ACPIGeneralSpecification_1-10_0-37-Published.pdf)
    //
    // The current (2026, TCG-ACPI-General-Specification_v1.50_pub.pdf) version of the TPM ACPI spec
    // has a revision of 6, which adds two more fields to the Table (LAML and LASA).
    // We ignore that for now and go ahead with the revision 4 that is supported.
    // TODO: Find out how to `find_table::<T>()` for different revisions, as to support multiple revisions
    //       if it turns out that a hardware TPM 2.0 supports these
    let is_correct_revision = tpm2_table.header().revision == 4; 
    if !is_correct_revision {
        info!(
            "TPM2 table header contained revision {:#?}, instead of the only supported value of 6",
            tpm2_table.header().revision
        );
    }

    let is_client = tpm2_table.plattform_class() == Ok(PlattformClass::Client);
    if !is_client {
        info!(
            "TPM2 plattform class is {:#?}, instead of the only supported value of {:#?}",
            tpm2_table.plattform_class(),
            PlattformClass::Client
        );
    }

    let has_crb_interface = tpm2_table.start_method() == Ok(super::acpi::StartMethodType::CrbInterface);
    if !has_crb_interface {
        info!(
            "TPM2 start method is {:#?} instead of the only supported value of {:#?}",
            tpm2_table.start_method(),
            super::acpi::StartMethodType::CrbInterface
        );
    }

    if has_tpm2_header && is_correct_revision && is_client && has_crb_interface {
        let buffer_address = tpm2_table.get_control_area_address();
        let crb_registers_start = buffer_address - 0x40;

        return Some(crb_registers_start);
    } else {
        info!("Full TPM2 ACPI table:\n{:#?}", tpm2_table);
        
        return None;
    }
}
