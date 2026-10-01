use tock_registers::registers::{ReadOnly, ReadWrite};
use tock_registers::{register_bitfields, register_structs};

register_structs! {

    /// As defined in "Table 36 — Address Allocation for CRB TPM Access"
    pub CrbRegisters {
        /// Used to determine current state of locality of a TPM. This register is aliased across all localities. Read-only.
        (0x0000 => pub loc_state: ReadOnly<u32, LocalityState::Register>),

        /// Reserved
        (0x0004 => _reserved0),

        /// Used to gain control of a TPM by this locality. This register SHALL NOT be aliased.
        (0x0008 => pub loc_ctrl: ReadWrite<u32, LocalityControl::Register>),

        /// Used to determine whether locality has been granted or seized. Read-only. This register SHALL NOT be aliased.
        (0x000C => pub loc_sts: ReadOnly<u32, LocalityStatus::Register>),

        /// Register to enable checksum computation on command and response
        (0x0010 => pub data_csum_enable: ReadWrite<u32, DataChecksumEnable::Register>),

        /// Register to read the checksum computed
        (0x0014 => pub data_csum: ReadOnly<u32, DataChecksum::Register>),

        /// Reserved
        (0x0018 => _reserved1),

        /// Used to identify the Interface types supported by a TPM as well as the Vendor ID, Device ID and Revision ID
        (0x0030 => pub crb_intf_id: ReadOnly<u64, CrbInterfaceIdentifier::Register>),

        /// Reserved
        (0x0038 => _reserved12),

        /// Register used to initiate transactions for the CRB interface. This register MAY be aliased across localities.
        (0x0040 => pub crb_ctrl_req: ReadWrite<u32, CrbControlAreaRequest::Register>),

        /// Register used by a TPM to provide status of the CRB interface. This register MAY be aliased across localities
        (0x0044 => pub crb_ctrl_sts: ReadOnly<u32, CrbControlAreaStatus::Register>),

        /// Register used by Software to cancel command processing. This register MAY be aliased across localities.
        (0x0048 => pub crb_ctrl_cancel: ReadWrite<u32, CrbControlCancel::Register>),

        /// Register used to indicate presence of command or response data in the CRB buffer. This register MAY be aliased across localities
        (0x004C => pub crb_ctrl_start: ReadWrite<u32, CrbControlStart::Register>),

        /// Register used to configure interrupts. This register MAY be aliased across localities
        (0x0050 => pub crb_int_enable: ReadWrite<u32, CrbInterruptControl::Register>),

        /// Register used to respond to interrupts. This register MAY be aliased across localities
        (0x0054 => pub crb_int_sts: ReadWrite<u32, CrbInterruptStatus::Register>),

        /// Size of the Command buffer. This register MAY be aliased across localities.
        (0x0058 => pub crb_ctrl_cmd_size: ReadOnly<u32>),

        /// Lower 32bits of the Command buffer start address for Locality 0. This register MAY be aliased across localities.
        (0x005C => pub crb_ctrl_cmd_laddr: ReadOnly<u32>),

        /// Upper 32bits of the Command buffer start address for Locality 0. This register MAY be aliased across localities.
        (0x0060 => pub crb_ctrl_cmd_haddr: ReadOnly<u32>),

        /// Size of the Response buffer. Note: If command and response buffers are implemented as a single buffer, this field SHALL be identical to the value in the TPM_CRB_CTRL_CMD_SIZE_x buffer. This register MAY be aliased across localities.
        (0x0064 => pub crb_ctrl_rsp_size: ReadOnly<u32>),

        /// Address of the start of the Response buffer. Note: If command and response buffers are implemented as a single buffer, this field SHALL contain the same address contained in TPM_CRB_CTRL_CMD_HADDR_x and TPM_CRB_CMD_LADDR_x. This register MAY be aliased across localities.
        (0x0068 => pub crb_ctrl_rsp_addr: ReadOnly<u64>),

        /// Reserved
        (0x0070 => _reserved2),

        /// Command/Response Data may be defined as large as 3968. This is implementation-specific. However, the full address space has been reserved. This buffer MAY be aliased across localities. This field accepts data transfers from 1B up to the size indicated by TPM_CRB_INTF_ID_x.CapDataXferSizeSupport (see section 6.4.2.2 CRB Interface Identifier Register).
        (0x0080 => pub crb_data_buffer: [ReadWrite<u8>; 3712]),

        /// Reserved for maximum size of Command/Response Buffer
        (0x0F00 => _reserved3),

        (0x1000 => @END), // "[...]pointing to the offset immediately past the list of registers."
    }
}

register_bitfields![u32,
    /// Used to determine status of the locality controls of a TPM.
    ///
    /// See also "Table 37 — TPM_LOC_STATE Definition"
    LocalityState [
        /// A TPM clears this bit to 0 upon receipt of D-RTM HASH_END.\
        /// A TPM sets this bit to a 1 when the TPM_LOC_CTRL_x.resetEstablishment field is set to 1.
        tpmEstablished              OFFSET(0)   NUMBITS(1) [],

        /// A 0 indicates to the host that no locality is assigned,
        /// a 1 indicates a locality has been assigned.
        locAssigned                 OFFSET(1)   NUMBITS(1) [],

        /// This bit field informs host Software which locality currently has access to a TPM.
        activeLocality              OFFSET(2)   NUMBITS(3) [
            Locality0 = 0,
            Locality1 = 1,
            Locality2 = 2,
            Locality3 = 3,
            Locality4 = 4,
        ],

        /// This bit indicates that all other bits of this register contain valid values, when it is a 1.
        tpmRegValidSts              OFFSET(7)   NUMBITS(1) [],
    ],


    /// Used to gain control of a TPM
    ///
    /// See also "Table 38 — TPM_LOC_CTRL_x Register Definition"
    LocalityControl [
        /// Reads always return 0.\
        /// Writes (0): Ignored.\
        /// Writes (1): Interrupt a TPM and execute a locality arbitration algorithm.
        ///
        /// **Note:** This field corresponds to the TPM_ACCESS_x.requestUse field in the FIFO implementation
        requestAccess               OFFSET(0)   NUMBITS(1) [],

        /// Reads always return 0.\
        /// Writes (0): Ignored.\
        /// Writes (1): The active locality is done with the TPM.
        Relinquish                  OFFSET(1)   NUMBITS(1) [],

        /// Reads always return 0.\
        /// Writes (0): Ignored.\
        /// Writes (1): A TPM gives control of the TPM to the locality
        /// setting this bit if it is the higher priority locality.
        Seize                       OFFSET(2)   NUMBITS(1) [],

        /// Reads always return 0.\
        /// Writes (0): Ignored.\
        /// Writes (1): Reset TPM_LOC_STATE_x.tpmEstablished bit if the write occurs from Locality 3 or 4.\
        /// Valid indicator: NA
        resetEstablishmentBit       OFFSET(3)   NUMBITS(1) [],
    ],


    /// Used to perform actions of D-RTM Sequence
    ///
    /// (Why do you do this to me?)\
    /// See also "Table 39 — TPM_LOC_CTRL_4 Register Definition"
    LocalityControlLocality4 [
        /// Reads return 0\
        /// Writes (0): Ignored\
        /// Writes (1): Initiates the HASH_START TPM actions (see Section 5.3 Locality-Controlled Functions)
        TPM_HASH_START              OFFSET(1)   NUMBITS(1) [],

        /// Reads return 0\
        /// Writes (0): Ignored\
        /// Writes (1): Initiates the HASH_DATA TPM actions (see Section 5.3 Locality-Controlled Functions)
        TPM_HASH_DATA               OFFSET(1)   NUMBITS(1) [],

        /// Reads return 0\
        /// Writes (0): Ignored\
        /// Writes (1): Initiates the HASH_END TPM actions (see Section 5.3 Locality-Controlled Functions)
        TPM_HASH_END                OFFSET(1)   NUMBITS(1) [],

        /// Reads always return 0.\
        /// Writes (0): Ignored.\
        /// Writes (1): Reset TPM_LOC_STATE_x.tpmEstablished bit if the write occurs from Locality 3 or 4.\
        /// Valid indicator: NA
        resetEstablishmentBit       OFFSET(3)   NUMBITS(1) [],
    ],


    /// If a locality is the active locality, Software can use this field
    /// to determine whether the active locality has been taken
    /// away (i.e., seized) by another, higher priority locality and
    /// therefore the seized locality needs to abort an entire task
    /// and restart it after it has obtained the active locality again.
    ///
    /// This register is unique for each locality.
    ///
    /// See also "Table 40 —TPM_LOC_STS"
    LocalityStatus [
        /// 0: Locality has not been granted access to the TPM.\
        /// 1: Locality has been granted access to the TPM
        Granted                     OFFSET(0)   NUMBITS(1) [],

        /// 0: A higher locality has not initiated a Seize arbitration process.\
        /// 1: A higher locality has Seized the TPM from this locality.
        beenSeized                  OFFSET(0)   NUMBITS(1) [],
    ],


    /// See also "Table 28 — TPM_DATA_CSUM_ENABLE Definition"
    DataChecksumEnable [
        /// Implicit Data Checksum enabled
        dataCSumEnable              OFFSET(0)   NUMBITS(1) [
            Enabled = 1,
            Disabled = 0,
        ],

        /// In Command Reception state
        dataCSumRequest             OFFSET(1)   NUMBITS(1) [
            RequestDataChecksumCalculation = 1,
            TpmCompletedDataChecksumCalculation = 0,
        ],
    ],


    /// See also "Table 29 — TPM_DATA_CSUM Definition"
    DataChecksum [
        /// Read returns the checksum of the entire command data at the end of the command transmission
        /// or the checksum of the entire response data at the end of the response transmission.
        ///
        /// Default: 0x00
        dataChecksum                OFFSET(0)   NUMBITS(16) []
    ],

];

register_bitfields![
    u64,

    /// CRB Interface Identifier Register
    ///
    /// See also "Table 24 — CRB Interface Identifier Register"
    CrbInterfaceIdentifier [
        /// `0000` – FIFO interface as defined in PTP for TPM 2.0 is active.\
        /// `0010` – RAM CRB interface is active.\
        /// `0001` – CRB interface is active.\
        /// `1111` – FIFO interface as defined in TIS1.3 is active (all other fields of this register are don’t care).
        InterfaceType               OFFSET(0)   NUMBITS(4) [],

        // ....
    ],
];

register_bitfields![u32,

    /// The Control Area Request register is used to manage TPM states as defined in Section 6.5.3.10 Interface Controls.
    ///
    /// See also "Table 41 — TPM CRB Control Area Request"
    CrbControlAreaRequest [
        /// Used by Software to request a TPM to transition to the Ready State.
        ///
        /// 1: Set to 1 by Software to indicate the TPM needs to be ready to receive a command.\
        /// 0: Cleared to 0 by the TPM to acknowledge completion of the state change request
        /// to the Ready state with the resulting state reflected in the TPM_CRB_STS_x.tpmIdle field.
        /// The TPM SHALL complete this transition within TIMEOUT_C.
        ///
        /// Writes of 0 are ignored.
        cmdReady                    OFFSET(0)   NUMBITS(1) [],

        /// Used by Software to indicate transition of a TPM to and from the Idle state.
        ///
        /// 1: Set by Software to indicate response has been read from the response buffer and the TPM can transition to Idle.\
        /// 0: Cleared to 0 by the TPM to acknowledge completion of the state change request
        /// to the Idle state with the resulting state reflected in the TPM_CRB_STS_x.tpmIdle field.
        /// The TPM SHALL complete this transition within TIMEOUT_C.
        ///
        /// Writes of 0 are ignored.
        goIdle                      OFFSET(1)   NUMBITS(1) [],
    ],


    /// This register is used to indicate the current state and status of a TPM.
    ///
    /// See also "Table 42 — TPM CRB Control Area Status"
    CrbControlAreaStatus [


        /// Used by the TPM to indicate current status.\
        /// 1: Set by the TPM to indicate a FATAL Error\
        /// 0: Indicates the TPM is operational
        tpmSts                      OFFSET(0)   NUMBITS(1) [],

        /// Used by the TPM to indicate it is in the Idle State.\
        /// 1: Set by the TPM when in the Idle State\
        /// 0: Cleared by the TPM on receipt of `TPM_CRB_CTRL_REQ_x.cmdReady` when TPM
        /// transitions to the Ready State. SHALL be cleared by `TIMEOUT_C`
        tpmIdle                     OFFSET(1)   NUMBITS(1) [],


        /// Used by a TPM to indicate the command checksum is available to be read by the host,
        /// if the checksum is supported.
        ///
        /// 1: Set by a TPM when the command CSUM is available \
        /// 0: Indicates no CSUM is available
        ///
        /// See Section 6.5.1.8.2 TPM_DATA_CSUM
        cSUMAvailable               OFFSET(2)   NUMBITS(1) [],
    ],


    /// Cancel may be used by Software to request a TPM to terminate processing the current command.
    ///
    /// See also "Table 43 — TPM CRB Control Cancel"
    CrbControlCancel [
        Cancel                      OFFSET(0)   NUMBITS(32) [
            CancelACommand = 0,
            ClearFieldWhenCommandHasBeenCanceled = 1,
        ],
    ],


    /// See also "Table 44 — TPM CRB Control Start"
    CrbControlStart [
        /// When set by software, indicates a command is ready for processing.
        /// When cleared by a TPM, the TPM has transitioned to Command Completion.
        Start                       OFFSET(0)   NUMBITS(1) [],

        /// Command Completion:
        ///
        /// Set to 1 by software to request the TPM to write the first chunk
        /// in the CRB Data Buffer again, e.g., a Response Retry.\
        /// Cleared to 0 by the TPM when the first chunk of the response
        /// is available in the CRB Data Buffer on a retry.
        crbRspRetry                 OFFSET(1)   NUMBITS(1) [],

        /// TODO
        nextChunk                   OFFSET(2)   NUMBITS(1) [],
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
    ],


];
