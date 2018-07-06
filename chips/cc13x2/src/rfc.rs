//! RF Core
//!
//! Provides communication with the core module of the radio.
//!
//! The radio is managed by an external Cortex-M0 running prioprietary code in order to manage
//! and set everything up. All stacks is implemented on this external MCU, and interaction
//! with it enables the radio for communication in Sub-GHz bands.
//!
//! In order to communicate, we send commands to the Cortex-M0 through something called
//! "Radio Doorbell".
//!
//!
use commands as cmd;
use core::cell::Cell;
use kernel::common::regs::{ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use prcm;

//*****************************************************************************
//
// This section defines the register offsets of
// RFC_DBELL component
//
//*****************************************************************************

#[repr(C)]
pub struct RfcDBellRegisters {
    // Doorbell Command Register
    pub cmdr: ReadWrite<u32>,
    // RFC Command Status register
    pub cmdsta: ReadOnly<u32>,
    // Interrupt Flags From RF HW Modules
    _rfhwifg: ReadWrite<u32, RFHWInterrupts::Register>,
    // Interrupt Flags For RF HW Modules
    _rfhwien: ReadWrite<u32, RFHWInterrupts::Register>,
    // Interrupt Flags For CPE Generated Interrupts
    pub rfcpeifg: ReadWrite<u32, CPEIntFlags::Register>,
    // Interrupt Enable For CPE Generated Interrupts
    pub rfcpeien: ReadWrite<u32, CPEInterrupts::Register>,
    // Interrupt Vector Selection for CPE
    pub rfcpeisl: ReadWrite<u32, CPEVectorSelect::Register>,
    // Doorbell Command Acknowledgement Interrupt Flags
    pub rfackifg: ReadWrite<u32, DBellCmdAck::Register>,
    // RF Core General Purpose Output Control
    pub sysgpoctl: ReadWrite<u32, RFCoreGPO::Register>,
}

register_bitfields! {
    u32,
    Status [
        RESULT   OFFSET(0) NUMBITS(8) [
            Pending = 0x00,
            Done = 0x01,
            IllegalPointer = 0x81,
            UnknownCommand = 0x82,
            UnknownDirCommand = 0x83,
            ContextError = 0x85,
            SchedulingError = 0x86,
            ParError = 0x87,
            QueueError = 0x88,
            QueueBusy = 0x89
        ],
        RETBYTE1 OFFSET(8) NUMBITS(8) [],
        RETBYTE2 OFFSET(16) NUMBITS(8) [],
        RETBYTE3 OFFSET(16) NUMBITS(8) []
    ],
    RFHWInterrupts [
        ALL_INTERRUPTS OFFSET(1) NUMBITS(19) []
    ],
    RFHWIntFlags [
        FSCA     OFFSET(1) NUMBITS(1) [],                     // Frequency synthesizer calibration accelerator interrupt flag/enable
        MDMDONE  OFFSET(2) NUMBITS(1) [],                     // Modem command done interrupt flag/enable
        MDMIN    OFFSET(3) NUMBITS(1) [],                     // Modem FIFO input interupt flag/enable
        MDMOUT   OFFSET(4) NUMBITS(1) [],                     // Modem FIFO output interrupt flag/enable
        MDMSOFT  OFFSET(5) NUMBITS(1) [],                     // Modem software defined interrupt flag/enable
        TRCTK    OFFSET(6) NUMBITS(1) [],                     // Debug tracer systick interrupt flag/enable
        RFEDONE  OFFSET(8) NUMBITS(1) [],                     // RF engine command done interrupt flag/enable
        RFESOFT0 OFFSET(9) NUMBITS(1) [],                     // RF engine software defined interrupt 0 flag/enable
        RFESOFT1 OFFSET(10) NUMBITS(1) [],                    // RF engine software defined interrupt 1 flag/enable
        RFESOFT2 OFFSET(11) NUMBITS(1) [],                    // RF engine software defined interrupt 2 flag/enable
        RATCH0   OFFSET(12) NUMBITS(1) [],                    // Radio timer channel 0 interrupt flag/enable
        RATCH1   OFFSET(13) NUMBITS(1) [],                    // Radio timer channel 1 interrupt flag/enable
        RATCH2   OFFSET(14) NUMBITS(1) [],                    // Radio timer channel 2 interrupt flag/enable
        RATCH3   OFFSET(15) NUMBITS(1) [],                    // Radio timer channel 3 interrupt flag/enable
        RATCH4   OFFSET(16) NUMBITS(1) [],                    // Radio timer channel 4 interrupt flag/enable
        RATCH5   OFFSET(17) NUMBITS(1) [],                    // Radio timer channel 5 interrupt flag/enable
        RATCH6   OFFSET(18) NUMBITS(1) [],                    // Radio timer channel 6 interrupt flag/enable
        RATCH7   OFFSET(19) NUMBITS(1) []                     // Radio timer channel 7 interrupt flag/enable
    ],
    CPEInterrupts [
        ALL_INTERRUPTS OFFSET(0) NUMBITS(32) []
    ],
    CPEIntFlags [
        COMMAND_DONE         OFFSET(0) NUMBITS(1) [],         // A radio operation command in chain has finished
        LAST_COMMAND_DONE    OFFSET(1) NUMBITS(1) [],         // Last radio operation command in chain has finished
        FG_COMMAND_DONE      OFFSET(2) NUMBITS(1) [],         // IEEE 802.15.4 mode only. Foreground radio operation command finished
        LAST_FG_COMMAND_DONE OFFSET(3) NUMBITS(1) [],         // IEEE 802.15.4 mode only. Last foreground radio operation command finished
        TX_DONE              OFFSET(4) NUMBITS(1) [],         // Packet transmitted
        TX_ACK               OFFSET(5) NUMBITS(1) [],         // Transmitted automantic ACK frame
        TX_CTRL              OFFSET(6) NUMBITS(1) [],         // BLE Mode: Transmitted LL control packet
        TX_CTRL_ACK          OFFSET(7) NUMBITS(1) [],         // BLE Mode: ACK received on transmitted LL control packet
        TX_CTRL_ACK_ACK      OFFSET(8) NUMBITS(1) [],         // BLE Mode: ACK received on ACK for transmitted LL control packet
        TX_RETRANS           OFFSET(9) NUMBITS(1) [],         // BLE Mode only: Packet retransmitted
        TX_ENTRY_DONE        OFFSET(10) NUMBITS(1) [],        // Tx queue data entry state chaned to finished
        TX_BUFFER_CHANGED    OFFSET(11) NUMBITS(1) [],        // BLE Mode only: Buffer change is complete after CMD_BLE_ADV_PAYLOAD
        BG_COMMAND_SUSPENDED OFFSET(12) NUMBITS(1) [],        // IEEE 802.15.4 only: Background level radio operation command has been suspended
        IRQ13                OFFSET(13) NUMBITS(1) [],        // Int flag 13
        IRQ14                OFFSET(14) NUMBITS(1) [],        // Int flag 14
        IRQ15                OFFSET(15) NUMBITS(1) [],        // Int flag 15
        RX_OK                OFFSET(16) NUMBITS(1) [],        // Packet received correctly
        RX_NOK               OFFSET(17) NUMBITS(1) [],        // Packet received crc error
        RX_IGNORED           OFFSET(18) NUMBITS(1) [],        // Packet received but can be ignored
        RX_EMPTY             OFFSET(19) NUMBITS(1) [],        // BLE Mode only: Packet received correctly but cannot be ignored
        RX_CTRL              OFFSET(20) NUMBITS(1) [],        // BLE Mode only: LL control packet received iwth CRC OK
        RX_CTRL_ACK          OFFSET(21) NUMBITS(1) [],        // BLE Mode only: ACK sent after RX_CTRL true
        RX_BUF_FULL          OFFSET(22) NUMBITS(1) [],        // Packet received that did not fit in Rx queue
        RX_ENTRY_DONE        OFFSET(23) NUMBITS(1) [],        // RX queue data entry changing state to finished
        RX_DATA_WRITTEN      OFFSET(24) NUMBITS(1) [],        // Data written to partial read Rx buffer
        RX_N_DATA_Written    OFFSET(25) NUMBITS(1) [],        // Specified number of bytes written to partial read Rx buffer
        RX_ABORTED           OFFSET(26) NUMBITS(1) [],        // Packet reception has stopped
        IRQ27                OFFSET(27) NUMBITS(1) [],        // Int flag 27
        SYNTH_NO_LOCK        OFFSET(28) NUMBITS(1) [],        // PLL in frequency synth has reported loss of lock
        MODULES_UNLOCKED     OFFSET(29) NUMBITS(1) [],        // CPE has access to RF Core modules and memories as part of boot
        BOOT_DONE            OFFSET(30) NUMBITS(1) [],        // CPE boot is finished
        INTERNAL_ERROR       OFFSET(31) NUMBITS(1) []          // CPE has observed an unexpected error. CPE reset is needed.
    ],
    CPEVectorSelect [
        ALL OFFSET(0) NUMBITS(32) []
    ],
    DBellCmdAck [
        CMDACK OFFSET(0) NUMBITS(1) []
    ],
    RFCoreGPO [
        GPOCTL0 OFFSET(0) NUMBITS(4) [                        // Selects which signal to output on RF Core GPO 0
            GPOCTL0_CPEGPO0 = 0x0,
            GPOCTL0_CPEGPO1 = 0x1,
            GPOCTL0_CPEGPO2 = 0x2,
            GPOCTL0_CPEGPO3 = 0x3,
            GPOCTL0_MCEGPO0 = 0x4,
            GPOCTL0_MCEGPO1 = 0x5,
            GPOCTL0_MCEGPO2 = 0x6,
            GPOCTL0_MCEGPO3 = 0x7,
            GPOCTL0_RFEGPO0 = 0x8,
            GPOCTL0_RFEGPO1 = 0x9,
            GPOCTL0_RFEGPO2 = 0xA,
            GPOCTL0_RFEGPO3 = 0xB,
            GPOCTL0_RATGPO0 = 0xC,
            GPOCTL0_RATGPO1 = 0xD,
            GPOCTL0_RATGPO2 = 0xE,
            GPOCTL0_RATGPO3 = 0xF
        ],
        GPOCTL1 OFFSET(4) NUMBITS(4) [                        // Selects which signal to output on RF Core GPO 1
            GPOCTL1_CPEGPO0 = 0x0,
            GPOCTL1_CPEGPO1 = 0x1,
            GPOCTL1_CPEGPO2 = 0x2,
            GPOCTL1_CPEGPO3 = 0x3,
            GPOCTL1_MCEGPO0 = 0x4,
            GPOCTL1_MCEGPO1 = 0x5,
            GPOCTL1_MCEGPO2 = 0x6,
            GPOCTL1_MCEGPO3 = 0x7,
            GPOCTL1_RFEGPO0 = 0x8,
            GPOCTL1_RFEGPO1 = 0x9,
            GPOCTL1_RFEGPO2 = 0xA,
            GPOCTL1_RFEGPO3 = 0xB,
            GPOCTL1_RATGPO0 = 0xC,
            GPOCTL1_RATGPO1 = 0xD,
            GPOCTL1_RATGPO2 = 0xE,
            GPOCTL1_RATGPO3 = 0xF
        ],
        GPOCTL2 OFFSET(8) NUMBITS(4) [                        // Selects which signal to output on RF Core GPO 2
            GPOCTL2_CPEGPO0 = 0x0,
            GPOCTL2_CPEGPO1 = 0x1,
            GPOCTL2_CPEGPO2 = 0x2,
            GPOCTL2_CPEGPO3 = 0x3,
            GPOCTL2_MCEGPO0 = 0x4,
            GPOCTL2_MCEGPO1 = 0x5,
            GPOCTL2_MCEGPO2 = 0x6,
            GPOCTL2_MCEGPO3 = 0x7,
            GPOCTL2_RFEGPO0 = 0x8,
            GPOCTL2_RFEGPO1 = 0x9,
            GPOCTL2_RFEGPO2 = 0xA,
            GPOCTL2_RFEGPO3 = 0xB,
            GPOCTL2_RATGPO0 = 0xC,
            GPOCTL2_RATGPO1 = 0xD,
            GPOCTL2_RATGPO2 = 0xE,
            GPOCTL2_RATGPO3 = 0xF
        ],
        GPOCTL3 OFFSET(12) NUMBITS(4) [                       // Selects which signal to output on RF Core GPO 3
            GPOCTL3_CPEGPO0 = 0x0,
            GPOCTL3_CPEGPO1 = 0x1,
            GPOCTL3_CPEGPO2 = 0x2,
            GPOCTL3_CPEGPO3 = 0x3,
            GPOCTL3_MCEGPO0 = 0x4,
            GPOCTL3_MCEGPO1 = 0x5,
            GPOCTL3_MCEGPO2 = 0x6,
            GPOCTL3_MCEGPO3 = 0x7,
            GPOCTL3_RFEGPO0 = 0x8,
            GPOCTL3_RFEGPO1 = 0x9,
            GPOCTL3_RFEGPO2 = 0xA,
            GPOCTL3_RFEGPO3 = 0xB,
            GPOCTL3_RATGPO0 = 0xC,
            GPOCTL3_RATGPO1 = 0xD,
            GPOCTL3_RATGPO2 = 0xE,
            GPOCTL3_RATGPO3 = 0xF
        ]
    ]
}

//*****************************************************************************
//
// This section defines the register offsets of
// RFC_PWR component
//
//*****************************************************************************
#[repr(C)]
pub struct RfcPWCRegisters {
    pub pwmclken: ReadWrite<u32, RFCorePWMEnable::Register>,
}

register_bitfields! {
    u32,
    RFCorePWMEnable [
        RFC    OFFSET(0) NUMBITS(1) [],                       // Enable essential clocks for RF Core interface
        CPE    OFFSET(1) NUMBITS(1) [],                       // Enable processor clock (hclk) to CPE. Set this bit with CPERAM to enable CPE boot
        CPERAM OFFSET(2) NUMBITS(1) [],                       // Enable clock to CPE RAM module
        MDM    OFFSET(3) NUMBITS(1) [],                       // Enable clock to Modem module
        MDMRAM OFFSET(4) NUMBITS(1) [],                       // Enable clock to Modem RAM module
        RFE    OFFSET(5) NUMBITS(1) [],                       // Enable clock to RF engine module
        RFERAM OFFSET(6) NUMBITS(1) [],                       // Enable clock to RF engine ram module
        RAT    OFFSET(7) NUMBITS(1) [],                       // Enable clock to Radio Timer
        PHA    OFFSET(8) NUMBITS(1) [],                       // Enable clock to packet handling accelerator module
        FSCA   OFFSET(9) NUMBITS(1) [],                       // Enable clock to frequency synthesizer calibration accelerator module
        RFCTRC OFFSET(10) NUMBITS(1) []                       // Enable clock to the RF Core Tracer module
    ]
}

const RFC_PWC_BASE: StaticRef<RfcPWCRegisters> =
    unsafe { StaticRef::new(0x4004_000 as *const RfcPWCRegisters) };
const RFC_DBELL_BASE: StaticRef<RfcDBellRegisters> =
    unsafe { StaticRef::new(0x4004_1000 as *const RfcDBellRegisters) };

pub const RFC_RAM_BASE: usize = 0x2100_0000;
pub const RFC_ULLRAM_BASE: usize = 0x2100_4000;

pub static mut RFCORE: RFCore = RFCore::new();

type RfcResult = Result<(), u32>;

#[derive(Clone, Copy)]
pub enum RfcInterrupt {
    Cpe0,
    Cpe1,
    CmdAck,
    Hardware,
}

#[derive(Clone, Copy)]
pub enum RfcMode {
    NONPROP = 0x00,
    IEEE802144 = 0x01,
    BLE = 0x02,
    PROPRF = 0x03,
    Unchanged = 0xFF,
}

#[derive(Clone, Copy)]
pub enum RfcCMDSTA {
    Pending = 0x00,
    Done = 0x01,
    IllegalPointer = 0x81,
    UnknownCommand = 0x82,
    UnknownDirCommand = 0x83,
    ContextError = 0x85,
    SchedulingError = 0x86,
    ParError = 0x87,
    QueueError = 0x88,
    QueueBusy = 0x89,
}

pub struct RFCore {
    // dbell_regs: StaticRef<RfcDBellRegisters>,
    // pwc_regs: StaticRef<RfcPWCRegisters>,
    client: Cell<Option<&'static RFCoreClient>>,
    mode: Cell<Option<RfcMode>>,
    rat: Cell<u32>,
}

pub trait RFCoreClient {
    fn command_done(&self);
    fn tx_done(&self);
}

impl RFCore {
    pub const fn new() -> RFCore {
        RFCore {
            // dbell_regs: RFC_DBELL_BASE,
            // pwc_regs: RFC_PWC_BASE,
            client: Cell::new(None),
            mode: Cell::new(None),
            rat: Cell::new(0),
        }
    }

    pub fn is_enabled(&self) -> bool {
        prcm::Power::is_enabled(prcm::PowerDomain::RFC)
    }

    pub fn enable(&self) {
        // Make sure RFC power is enabled
        if !prcm::Power::is_enabled(prcm::PowerDomain::RFC) {
            prcm::Power::enable_domain(prcm::PowerDomain::RFC);

            while !prcm::Power::is_enabled(prcm::PowerDomain::RFC) {}
        }

        // Set power and clock regs for RFC
        let pwc_regs = RFC_PWC_BASE;

        pwc_regs.pwmclken.set(0x7FF);

        // Enable interrupts and clear flags
        self.enable_cpe_interrupts();
        self.enable_hw_interrupts();

        self.handle_cpe_interrupts();
        self.handle_hw_interrupts();

        // Initialize radio module
        self.send_direct(&cmd::DirectCommand::new(cmd::RFC_CMD0, 0x10 | 0x40))
            .ok()
            .expect("Could not initialize radio module");

        // Request bus
        self.send_direct(&cmd::DirectCommand::new(cmd::RFC_BUS_REQUEST, 1))
            .ok()
            .expect("Could not request bus for radio module");

        // Ping radio module
        self.send_direct(&cmd::DirectCommand::new(cmd::RFC_PING, 0))
            .ok()
            .expect("Coudl not ping radio module");
    }

    pub fn disable(&self) {
        self.send_direct(&cmd::DirectCommand::new(cmd::RFC_STOP, 0))
            .ok()
            .expect("Could not send stop cmd to radio module");

        self.disable_cpe_interrupts();
        self.disable_hw_interrupts();

        let fs_powerdown: cmd::CmdFSPowerdown = cmd::CmdFSPowerdown::new();

        self.send(&fs_powerdown)
            .and_then(|_| self.wait(&fs_powerdown))
            .ok()
            .expect("Could not power down frequency synthesizer for radio module");

        self.stop_rat();

        // Add disable power domain and clocks

        self.mode.set(None);
    }

    pub fn setup(&self, reg_override: u32, tx_power: u16) {
        let mode = self.mode.get().expect("No RF mode selected, cannot setup");
        let radio_setup = cmd::CmdRadioSetup::new(reg_override, mode as u8, tx_power);

        self.send(&radio_setup)
            .and_then(|_| self.wait(&radio_setup))
            .ok()
            .expect("Could not enable NonProp mode in radio module");
    }

    pub fn current_mode(&self) -> Option<RfcMode> {
        self.mode.get()
    }

    pub fn set_mode(&self, mode: RfcMode) {
        let rf_mode = match mode {
            RfcMode::NONPROP => 0x00,
            _ => panic!("Only HAL mode supported"),
        };

        prcm::rf_mode_sel(rf_mode);

        self.mode.set(Some(mode))
    }

    fn post_cmdr(&self, rf_command: u32) -> RfcResult {
        let dbell_regs = RFC_DBELL_BASE;

        if !prcm::Power::is_enabled(prcm::PowerDomain::RFC) {
            panic!("RFC power domain is off");
        }

        dbell_regs.cmdr.set(rf_command);

        let mut timeout: u32 = 0;
        let mut status = 0;

        const MAX_TIMEOUT: u32 = 0x2FFFFFF;
        while timeout < MAX_TIMEOUT {
            status = self.cmdsta();
            if (status & 0xFF) == 0x01 {
                return Ok(());
            }

            timeout += 1;
        }

        return Err(status);
    }

    fn wait_cmdr(&self, rf_command: u32) -> RfcResult {
        let command_op: &cmd::CmdCommon = unsafe { &*(rf_command as *const cmd::CmdCommon) };

        let mut timeout: u32 = 0;
        let mut status = 0;
        const MAX_TIMEOUT: u32 = 0x2FFFFFF;
        while timeout < MAX_TIMEOUT {
            status = command_op.status.get();
            if status == 0x0400 {
                return Ok(());
            }

            timeout += 1;
        }

        return Err(status as u32);
    }

    fn cmdsta(&self) -> u32 {
        let dbell_regs = RFC_DBELL_BASE;
        let ret: u32 = dbell_regs.cmdsta.get();

        return ret;
    }

    fn enable_hw_interrupts(&self) {
        let dbell_regs = RFC_DBELL_BASE;
        // Enable all interrupts
        dbell_regs
            ._rfhwien
            .modify(RFHWInterrupts::ALL_INTERRUPTS::SET);
    }

    pub fn handle_hw_interrupts(&self) {
        let dbell_regs = RFC_DBELL_BASE;
        // Clear all RFHW interrupts
        dbell_regs
            ._rfhwifg
            .write(RFHWInterrupts::ALL_INTERRUPTS::SET);
    }

    fn disable_hw_interrupts(&self) {
        let dbell_regs = RFC_DBELL_BASE;
        // Disable all RFHW interrupts
        dbell_regs
            ._rfhwien
            .modify(RFHWInterrupts::ALL_INTERRUPTS::CLEAR);
        // Clear all RFHW interrupts
        dbell_regs
            ._rfhwifg
            .write(RFHWInterrupts::ALL_INTERRUPTS::SET);
    }

    fn enable_cpe_interrupts(&self) {
        let dbell_regs = RFC_DBELL_BASE;
        // Enable CPE interrupts
        dbell_regs
            .rfcpeien
            .modify(CPEInterrupts::ALL_INTERRUPTS::SET);
    }

    pub fn handle_cpe_interrupts(&self) {
        let dbell_regs = RFC_DBELL_BASE;
        // Clear all CPE interrupts
        dbell_regs.rfcpeifg.set(0x7FFFFFFF);
    }

    fn disable_cpe_interrupts(&self) {
        let dbell_regs = RFC_DBELL_BASE;
        // Disable all CPE interrupts
        dbell_regs
            .rfcpeien
            .modify(CPEInterrupts::ALL_INTERRUPTS::CLEAR);
        // Clear all CPE interrupts
        dbell_regs.rfcpeifg.set(0x7FFFFFFF);
    }

    pub fn cpe_vec_select(&self, cpe: bool) {
        let dbell_regs = RFC_DBELL_BASE;
        // Select CPE0 or CPE1
        if !cpe {
            dbell_regs.rfcpeisl.modify(CPEVectorSelect::ALL::CLEAR);
        } else {
            dbell_regs.rfcpeisl.modify(CPEVectorSelect::ALL::SET);
        }
    }

    pub fn handle_ack_interrupt(&self) {
        let dbell_regs = RFC_DBELL_BASE;
        // Reset flag for Command ACK
        dbell_regs.rfackifg.write(DBellCmdAck::CMDACK::SET);
    }

    pub fn send<T>(&self, rf_command: &T) -> RfcResult {
        let command = { (rf_command as *const T) as u32 };

        return self.post_cmdr(command);
    }

    fn send_direct(&self, dir_command: &cmd::DirectCommand) -> RfcResult {
        let command = {
            let cmd = dir_command.command_no as u32;
            let par = dir_command.command_no as u32;
            (cmd << 16) | (par & 0xFFFC) | 1
        };

        return self.post_cmdr(command);
    }

    pub fn wait<T>(&self, rf_command: &T) -> RfcResult {
        let command = { (rf_command as *const T) as u32 };

        return self.wait_cmdr(command);
    }

    pub fn start_rat(&self) {
        let rf_command = cmd::CmdSyncStartRat::new(self.rat.get());

        self.send(&rf_command)
            .and_then(|_| self.wait(&rf_command))
            .ok()
            .expect("Could not start radio timer.");
    }

    pub fn stop_rat(&self) {
        let rf_command = cmd::CmdSyncStopRat::new(self.rat.get());

        self.send(&rf_command)
            .and_then(|_| self.wait(&rf_command))
            .ok()
            .expect("Could not start radio timer");
    }
    pub fn handle_interrupts(&self, int: RfcInterrupt) {
        let dbell_regs = RFC_DBELL_BASE;

        match int {
            RfcInterrupt::CmdAck => {
                dbell_regs.rfackifg.set(0);
            }
            RfcInterrupt::Cpe0 => {
                let command_done = dbell_regs.rfcpeifg.is_set(CPEIntFlags::COMMAND_DONE);
                let tx_done = dbell_regs.rfcpeifg.is_set(CPEIntFlags::TX_DONE);

                dbell_regs.rfcpeifg.set(0);

                if command_done {
                    self.client.get().map(|client| client.command_done());
                }

                if tx_done {
                    self.client.get().map(|client| client.tx_done());
                }
            }
            RfcInterrupt::Cpe1 => {
                dbell_regs.rfcpeifg.set(0x7FFFFFFF);
                panic!("Internal error occurred during rad command")
            }
            _ => panic!("Unhandled RFC interrupt: {}\r", int as u8),
        }
    }

    pub fn set_client(&self, client: &'static RFCoreClient) {
        self.client.set(Some(client));
    }
}
