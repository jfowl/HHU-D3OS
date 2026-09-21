use core::ops::Deref;
use core::ptr::NonNull;
use tock_registers::interfaces::Readable;
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};
use tock_registers::{register_bitfields, register_structs};

register_structs! {

    /// As defined in "Table 36 — Address Allocation for CRB TPM Access"
    pub CrbRegisters {
        /// Used to determine current state of locality of a TPM. This register is aliased across all localities. Read-only.
        (0x0000 => pub loc_state: ReadOnly<u32, TpmLocState::Register>),

        /// Reserved
        (0x0004 => _reserved0x0004),

        /// Used to gain control of a TPM by this locality. This register SHALL NOT be aliased.
        (0x0008 => pub loc_ctrl: ReadOnly<u32, LocalityControl::Register>),

        /// Used to determine whether locality has been granted or seized. Read-only. This register SHALL NOT be aliased.
        (0x000C => pub loc_sts: ReadOnly<u32, LocalityStatus::Register>),

        /// Register to enable checksum computation on command and response
        (0x0010 => pub data_csum_enable: ReadOnly<u32, DataChecksumEnable::Register>),

        /// Register to read the checksum computed
        (0x0014 => pub data_csum: ReadOnly<u32, DataChecksum::Register>),

        /// Reserved
        (0x0018 => _reserved0018),

        /// Used to identify the Interface types supported by a TPM as well as the Vendor ID, Device ID and Revision ID
        (0x0030 => pub crb_intf_id: ReadOnly<u32, CrbInterfaceIdentifier::Register>),

        /// Reserved
        (0x0038 => _reserved0038),

        /// Register used to initiate transactions for the CRB interface. This register MAY be aliased across localities.
        (0x0040 => pub crb_ctrl_req: ReadOnly<u32, CrbControlAreaRequest::Register>),

        /// Register used by a TPM to provide status of the CRB interface. This register MAY be aliased across localities
        (0x0044 => pub crb_ctrl_sts: ReadOnly<u32, CrbControlAreaStatus::Register>),

        /// Register used by Software to cancel command processing. This register MAY be aliased across localities.
        (0x0048 => pub crb_ctrl_cancel: ReadOnly<u32, CrbControlCancel::Register>),

        /// Register used to indicate presence of command or response data in the CRB buffer. This register MAY be aliased across localities
        (0x004C => pub crb_ctrl_start: ReadOnly<u32, CrbControlStart::Register>),

        /// Register used to configure interrupts. This register MAY be aliased across localities
        (0x0050 => pub crb_int_enable: ReadOnly<u32, CrbInterruptControl::Register>),

        /// Register used to respond to interrupts. This register MAY be aliased across localities
        (0x0054 => pub crb_int_sts: ReadOnly<u32, CrbInterruptStatus::Register>),

        /// Size of the Command buffer. This register MAY be aliased across localities.
        (0x0058 => pub crb_ctrl_cmd_size: ReadOnly<u32>),

        /// Lower 32bits of the Command buffer start address for Locality 0. This register MAY be aliased across localities.
        (0x005C => pub crb_ctrl_cmd_laddr: ReadOnly<u32>),

        /// Upper 32bits of the Command buffer start address for Locality 0. This register MAY be aliased across localities.
        (0x0060 => pub crb_ctrl_cmd_haddr: ReadOnly<u32>),

        /// Size of the Response buffer. Note: If command and response buffers are implemented as a single buffer, this field SHALL be identical to the value in the TPM_CRB_CTRL_CMD_SIZE_x buffer. This register MAY be aliased across localities.
        (0x0064 => pub crb_ctrl_rsp_size: ReadOnly<u32>),

        /// Address of the start of the Response buffer. Note: If command and response buffers are implemented as a single buffer, this field SHALL contain the same address contained in TPM_CRB_CTRL_CMD_HADDR_x and TPM_CRB_CMD_LADDR_x. This register MAY be aliased across localities.
        (0x0068 => pub crb_ctrl_rsp_addr: ReadOnly<u32>),

        /// Reserved
        (0x0070 => _reserved0070),

        /// Command/Response Data may be defined as large as 3968. This is implementation-specific. However, the full address space has been reserved. This buffer MAY be aliased across localities. This field accepts data transfers from 1B up to the size indicated by TPM_CRB_INTF_ID_x.CapDataXferSizeSupport (see section 6.4.2.2 CRB Interface Identifier Register).
        (0x0080 => pub crb_data_buffer: ReadOnly<[u8; 496], TpmCrbDataBuffer::Register>),

        /// Reserved for maximum size of Command/Response Buffer
        (0x0F00 => _reserved0f00),

        (0x1000 => @END), // "[...]pointing to the offset immediately past the list of registers."
    }
}

register_bitfields![u32,
    TpmLocState [
        tpmEstablished  OFFSET(0)  NUMBITS(1) [],
        locAssigned     OFFSET(1)  NUMBITS(1) [],
        RESET   OFFSET(3)  NUMBITS(1) []
    ],

    /// Used to Control CRB Interrupts
    /// 
    /// See also "Table 48 — CRB Interrupt Control"
    CrbInterruptControl [
        startIntEnable              OFFSET(0)   NUMBITS(1) [],
        cmdReadyIntEnable           OFFSET(1)   NUMBITS(1) [],
        establishmentClearIntEnable OFFSET(2)   NUMBITS(1) [],
        localityChangeIntEnable     OFFSET(3)   NUMBITS(1) [],
        nextChunkClearedIntEnable   OFFSET(4)   NUMBITS(1) [],

        /// 0 = All interrupts are disabled.
        /// 1 = Interrupt enable is controlled by the individual bits in this register
        globalInterruptEnable       OFFSET(31)  NUMBITS(1) [],
    ],

    /// Shows which interrupt has occurred.
    /// 
    /// See also "Table 49 — Interrupt Status"
    CrbInterruptStatus [
        /// A 1 indicates that a TPM has executed a
        /// command as requested by TPM_CTRL_Start_x
        /// = 0001 and the corresponding response is
        /// available for read-out (i.e. Start field has been
        /// cleared). This interrupt will also be triggered if
        /// the currently executed command will be
        /// cancelled via a Command Cancel.
        /// 
        /// Writing a 1 to this field clears the interrupt.
        /// Writing a 0 to this field has no effect.
        startInt                    OFFSET(0)   NUMBITS(1) [],

        /// A 1 indicates that after a write of 1 to
        /// TPM_CTRL_REQ.cmdReady the TPM has
        /// successfully finished the transition to the Ready
        /// state (i.e. TPM_CRB_CTRL_STS_x.tpmIdle ==
        /// 0)
        /// 
        /// Writing a 1 to this field clears the interrupt.
        /// Writing a 0 to this field has no effect.        
        cmdReadyInt                 OFFSET(1)   NUMBITS(1) [],

        /// A 1 indicates that the reset of the
        /// TPM_LOC_STATE_x.tpmEstablished field has
        /// been successfully executed after the
        /// corresponding request
        /// TPM_LOC_CTRL_x.resetEstablishment
        /// 
        /// Writing a 1 to this field clears the interrupt.
        /// Writing a 0 to this field has no effect.        
        establishmentClearInt       OFFSET(2)   NUMBITS(1) [],

        /// A 1 indicates that a locality change has
        /// occurred.
        /// This interrupt is caused whenever the value of
        /// bits 4:2 of the TPM_LOC_STATE register
        /// changes.
        /// 
        /// **Note:**
        /// If a TPM has
        /// TPM_LOC_STATE_x.locAssigned == 0 before
        /// Request Use is set there will be no Interrupt
        /// because the TPM will make the transition
        /// immediately to the requesting locality
        /// 
        /// Writing a 1 to this field clears the interrupt.
        /// Writing a 0 to this field has no effect.        
        localityChangeInt           OFFSET(3)   NUMBITS(1) [],

        /// A 1 indicates that the nextChunk field has been
        /// cleared and the interface is ready for the next
        /// chunk of the command / response to be written /
        /// read to / from the
        /// TPM_CRB_DATA_BUFFER_x.
        /// 
        /// Writing a 1 to this field clears the interrupt.
        /// Writing a 0 to this field has no effect.
        nextChunkClearedInt         OFFSET(4)   NUMBITS(1) [],
    ]
];
