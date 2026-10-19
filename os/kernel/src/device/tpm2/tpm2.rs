use core::ops::Range;

use alloc::vec::{self, Vec};
use log::info;
use tock_registers::interfaces::{ReadWriteable, Readable};

use crate::device::tpm2::{
    acpi::try_get_crb_tpm2_via_acpi,
    crb::{CrbRegisters, LocalityControl, LocalityStatus, get_locality_0_regs},
    mmio::MMIO,
    util::mmio_map,
};

pub fn init_tpm2() {
    // QEMU: The CRB interface makes a memory mapped IO region
    // in the area 0xfed40000-0xfed40fff (1 locality)
    // available to the guest operating system.
    //
    // But since QEMU also makes this device discoverable via a static ACPI table,
    // we use that instead to find the address.

    let Some(cbr_addr) = try_get_crb_tpm2_via_acpi() else {
        info!("Unable to find a TPM2 device via ACPI");
        panic!();
    };
    info!("Found TPM2 device via static ACPI table at address 0x{:x}", cbr_addr);
    let mmio_range = Range {
        start: cbr_addr,
        end: cbr_addr + 0x1000,
    };

    mmio_map(mmio_range.clone(), "tpm2");

    let loc0_regs = get_locality_0_regs(mmio_range.start);

    info!("LOC_STATE: {:#?}", loc0_regs.loc_state.extract().debug());
    info!("LOC_STATUS: {:#?}", loc0_regs.loc_sts.extract().debug());
    info!("INTF_ID: {:#?}", loc0_regs.crb_intf_id.extract().debug());
    info!("CRB_STATUS: {:#?}", loc0_regs.crb_ctrl_sts.extract().debug());

    loc0_regs.loc_ctrl.modify(LocalityControl::requestAccess::SET + LocalityControl::Seize::SET);

    info!("Asked TPM2 to give access to Locality 0");

    info!("Polling TPM regs until access to locality...");
    for _ in 0..10_000_000 {
        if loc0_regs.loc_sts.extract().is_set(LocalityStatus::Granted) {
            info!("Locality granted!");
            break;
        }
    }
    if !loc0_regs.loc_sts.extract().is_set(LocalityStatus::Granted) {
        panic!("TPM locality was never granted");
    }

    info!("LOC_STATE: {:#?}", loc0_regs.loc_state.extract().debug());
    info!("LOC_STATUS: {:#?}", loc0_regs.loc_sts.extract().debug());
    info!("INTF_ID: {:#?}", loc0_regs.crb_intf_id.extract().debug());
    info!("CRB_STATUS: {:#?}", loc0_regs.crb_ctrl_sts.extract().debug());
    let raw_crb_status: u32 = loc0_regs.crb_ctrl_sts.extract().into();
    info!("CRB_STATUS raw: {:#?}", raw_crb_status);

    panic!("Time to shut down!");
}

pub enum Tpm2Locality {
    Locality0 = 0x0000,
    Locality1 = 0x1000,
    Locality2 = 0x2000,
    Locality3 = 0x3000,
    Locality4 = 0x4000,
}

#[derive(Debug, PartialEq)]
pub enum TpmError {
    InvalidStartMethod,
    InvalidPlattformClass,
}

pub struct Tpm2Device {
    locality0_regs: MMIO<CrbRegisters>,
}

impl Tpm2Device {
    pub unsafe fn new(base_addr: u64) -> Self {
        Self {
            locality0_regs: get_locality_0_regs(base_addr),
        }
    }

    pub fn run_command<C: Tpm2Cmd, R: Tpm2Rsp + AnswerTo<C>>(&mut self, cmd: C) -> Result<R, TpmError> {
        unimplemented!();
    }

    pub fn get_random(&mut self, cmd: GetRandomCmd) -> Result<GetRandomRsp, TpmError> {
        self.run_command(cmd)
    }
}

trait AnswerTo<T: Tpm2Cmd> {} // Marker trait
trait Tpm2Cmd {
    fn to_bytes() -> Vec<u8>;
}
trait Tpm2Rsp {
    fn from_bytes(bytes: Vec<u8>);
}

pub struct GetRandomCmd {}
pub struct GetRandomRsp {}
impl Tpm2Cmd for GetRandomCmd {
    fn to_bytes() -> Vec<u8> {
        unimplemented!()
    }
}
impl Tpm2Rsp for GetRandomRsp {
    fn from_bytes(bytes: Vec<u8>) {
        unimplemented!()
    }
}
impl AnswerTo<GetRandomCmd> for GetRandomRsp {}

pub fn test(tpm: &mut Tpm2Device) {
    tpm.run_command::<GetRandomCmd, GetRandomRsp>(GetRandomCmd {});
}


