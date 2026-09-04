use core::ptr::{write_volatile, read_volatile};

use acpi::{
    AcpiTable,
    sdt::{SdtHeader, Signature},
};
use log::info;
use tpm2::{Command, commands::GetRandom};
use x86_64::structures::paging::PageTableFlags;

use crate::{acpi_tables, device::tpm2::TpmError::InvalidPlattformClass, memory::vma::VmaType, process_manager};

pub fn init_tpm2() {
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

    // Use the Command Response Buffer (CRB) to send a get_random command to the TPM2:

    // <AISlop> ------------------------------------

    // Build a minimal TPM2 GetRandom command in TPM wire format.
    const TPM_ST_NO_SESSIONS: u16 = 0x8001;
    const TPM2_CC_GET_RANDOM: u32 = 0x0000_017b;
    const REQUESTED_BYTES: u16 = 16;

    let mut cmd = [0u8; 12];
    let cmd_len = cmd.len() as u32;
    cmd[0..2].copy_from_slice(&TPM_ST_NO_SESSIONS.to_be_bytes());
    cmd[2..6].copy_from_slice(&(cmd_len).to_be_bytes());
    cmd[6..10].copy_from_slice(&TPM2_CC_GET_RANDOM.to_be_bytes());
    cmd[10..12].copy_from_slice(&REQUESTED_BYTES.to_be_bytes());

    // On the common QEMU CRB setup, the command buffer is placed in the shared region
    // following the control registers.
    let crb_cmd_buf = (mmio_start as usize + 0x1000) as *mut u8;
    for (i, byte) in cmd.iter().copied().enumerate() {
        unsafe {
            write_volatile(crb_cmd_buf.add(i), byte);
        }
    }

    // Notify the TPM that a command is ready.
    let ctrl_req = mmio_start as *mut u32;
    unsafe {
        write_volatile(ctrl_req, 0x1);
    }

    info!("TPM2 GetRandom request queued via CRB ({} bytes)", REQUESTED_BYTES);
    // Poll for response: The CRB command/response buffer uses the same buffer.
    // The response size is at offset 2..6 (big-endian u32). Poll until non-zero or timeout.
    let crb_resp_buf = crb_cmd_buf;
    let mut resp_len: u32 = 0;
    for _ in 0..1_000_000 {
        unsafe {
            let b0 = read_volatile(crb_resp_buf.add(2)) as u32;
            let b1 = read_volatile(crb_resp_buf.add(3)) as u32;
            let b2 = read_volatile(crb_resp_buf.add(4)) as u32;
            let b3 = read_volatile(crb_resp_buf.add(5)) as u32;
            resp_len = (b0 << 24) | (b1 << 16) | (b2 << 8) | b3;
        }
        if resp_len != 0 {
            break;
        }
    }

    if resp_len == 0 {
        info!("TPM2 response timed out");
    } else {
        let read_len = core::cmp::min(resp_len as usize, 1024);
        let mut resp = alloc::vec::Vec::with_capacity(read_len);
        unsafe {
            for i in 0..read_len {
                resp.push(read_volatile(crb_resp_buf.add(i)));
            }
        }
        info!("TPM2 response (len={}): {:x?}", resp_len, resp);
    }

    // </AISlop> ------------------------------------
}

#[derive(Copy, Clone, Debug)]
enum PlattformClass {
    Client = 0,
    Server = 1,
}

/// As defined in
/// https://trustedcomputinggroup.org/wp-content/uploads/TCG_ACPIGeneralSpecification_v1.20_r8.pdf#%5B%7B%22num%22%3A71%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C69%2C530%2C0%5D
#[derive(Copy, Clone, Debug)]
enum StartMethodType {
    ///  Uses the ACPI Start method.
    AcpiStartMethod = 2,
    ///  Reserved for the Memory mapped I/O Interface (TIS 1.2+Cancel).
    MemMappedIO = 6,
    ///  Uses the Command Response Buffer Interface with the ACPI Start Method.
    CommandResponseBufferInterfaceWithAcpiStartMethod = 8,
    ///  Uses the Command Response Buffer Interface with ARM Secure Monitor Call (SMC)
    CommandResponseBufferInterfaceWithArmSMC = 11,
}

enum TpmError {
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
    start_method_params: u32,
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
        match self.start_method {
            2 => Ok(StartMethodType::AcpiStartMethod),
            6 => Ok(StartMethodType::MemMappedIO),
            8 => Ok(StartMethodType::CommandResponseBufferInterfaceWithAcpiStartMethod),
            11 => Ok(StartMethodType::CommandResponseBufferInterfaceWithArmSMC),
            _ => Err(InvalidPlattformClass),
        }
    }

    pub fn plattform_class(&self) -> Result<PlattformClass, TpmError> {
        match self.plattform_class {
            0 => Ok(PlattformClass::Client),
            1 => Ok(PlattformClass::Server),
            _ => Err(InvalidPlattformClass),
        }
    }
}
