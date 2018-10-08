//! RF Core
//!
//! Provides communication with the core module of the radio.
//!
//! The radio is managed by an external Cortex-M0 running prioprietary code in order to manage
//! and set everything up. The external MCU implements all network stacks, and the main MCU
//! communicates over the radio by interfacing with it.
//!
//!
//! In order to communicate, we send a set of commands to the cortex-m0 through an interface called
//! the radio doorbell.
//!
//! The radio doorbell is a communication mechanism between the system and radio MCUs which contains
//! a set of dedicated registers, shared access to MCU RAMs, and a set of interrupts to both the
//! radio CPU and to the system CPU. Parameters and payloads are transferred through the system RAM
//! or the radio RAM. During operation, the radio CPU updates parameters and payload in RAM and raises
//! interrupts. The system CPU can mask out interrupts so that it remains in idle or power-down mode
//! until the entire radio operation finishes. Because the system CPU and the radio CPU share a common
//! RAM area, software must ensure that there is no contention or race conditions. If any parameters or
//! payload are in the system RAM, the system CPU must remain powered. Otherwise, if everything is in the
//! radio RAM, the system CPU may go into power-down mode to save current.
//!
use radio::commands as cmd;
// use cortexm4::{self, nvic};
use self::test_commands::{CommandCommon, CommandRadioSetup, CommandSyncRat};
use core::cell::Cell;
use kernel::common::registers::{ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use prcm;
use rtc;

// This section defines the register offsets of
// RFC_DBELL component

#[repr(C)]
pub struct RfcDBellRegisters {
    // Doorbell Command Register
    cmdr: ReadWrite<u32>,
    // RFC Command Status register
    cmdsta: ReadOnly<u32>,
    // Interrupt Flags From RF HW Modules
    _rfhw_ifg: ReadWrite<u32>,
    // Interrupt Flags For RF HW Modules
    _rfhw_ien: ReadWrite<u32>,
    // Interrupt Flags For CPE Generated Interrupts
    rfcpe_ifg: ReadWrite<u32, CPEInterrupts::Register>,
    // Interrupt Enable For CPE Generated Interrupts
    rfcpe_ien: ReadWrite<u32, CPEInterrupts::Register>,
    // Interrupt Vector Selection for CPE
    rfcpe_isl: ReadWrite<u32, CPEInterrupts::Register>,
    // Doorbell Command Acknowledgement Interrupt Flags
    rfack_ifg: ReadWrite<u32, DBellCmdAck::Register>,
    // RF Core General Purpose Output Control
    _sysgpoctl: ReadWrite<u32>,
}

register_bitfields! {
    u32,
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
    ]
}

// This section defines the register offsets of
// RFC_PWC component

#[repr(C)]
pub struct RfcPWCRegisters {
    pub pwmclken: ReadWrite<u32, RFCPWE::Register>,
}

register_bitfields! {
    u32,
    RFCPWE [
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

#[derive(PartialEq, Clone, Copy)]
pub enum RfcMode {
    Common = 0x00,
    BLE = 0x01,
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

type RadioReturnCode = Result<(), u32>;

const RFC_PWC_BASE: StaticRef<RfcPWCRegisters> =
    unsafe { StaticRef::new(0x4004_0000 as *const RfcPWCRegisters) };

const RFC_DBELL_BASE: StaticRef<RfcDBellRegisters> =
    unsafe { StaticRef::new(0x4004_1000 as *const RfcDBellRegisters) };

pub const DRIVER_NUM: usize = 0xCC1312;

#[derive(Debug, Clone, Copy)]
pub enum RfcInterrupt {
    Cpe0,
    Cpe1,
    CmdAck,
    Hardware,
}

pub struct RFCore {
    dbell_regs: StaticRef<RfcDBellRegisters>,
    pwc_regs: StaticRef<RfcPWCRegisters>,
    client: Cell<Option<&'static RFCoreClient>>,
    pub mode: Cell<Option<RfcMode>>,
    pub rat: Cell<u32>,
    pub status: Cell<u32>,
}

impl RFCore {
    pub const fn new() -> RFCore {
        RFCore {
            dbell_regs: RFC_DBELL_BASE,
            pwc_regs: RFC_PWC_BASE,
            client: Cell::new(None),
            mode: Cell::new(None),
            rat: Cell::new(0),
            status: Cell::new(0),
        }
    }

    pub fn set_client(&self, client: &'static RFCoreClient) {
        self.client.set(Some(client));
    }

    // Check if RFCore params are enabled
    pub fn is_enabled(&self) -> bool {
        prcm::Power::is_enabled(prcm::PowerDomain::RFC)
    }

    // Enable RFCore
    pub fn enable(&self) {
        // Make sure RFC power is enabled
        prcm::Power::enable_domain(prcm::PowerDomain::RFC);
        prcm::Clock::enable_rfc();

        unsafe {
            rtc::RTC.set_upd_en(true);
        }

        while !prcm::Power::is_enabled(prcm::PowerDomain::RFC) {}

        // Set power and clock regs for RFC
        let pwc_regs = &*self.pwc_regs;
        pwc_regs.pwmclken.modify(
            RFCPWE::RFC::SET
                + RFCPWE::CPE::SET
                + RFCPWE::CPERAM::SET
                + RFCPWE::MDM::SET
                + RFCPWE::MDMRAM::SET
                + RFCPWE::RFE::SET
                + RFCPWE::RFERAM::SET
                + RFCPWE::RAT::SET
                + RFCPWE::PHA::SET
                + RFCPWE::FSCA::SET,
        );

        let dbell_regs = self.dbell_regs;

        // Clear ack flag
        dbell_regs.rfack_ifg.set(0);

        // Enable interrupts and clear flags
        dbell_regs
            .rfcpe_isl
            .write(CPEInterrupts::INTERNAL_ERROR::SET);
        dbell_regs.rfcpe_ien.write(
            CPEInterrupts::INTERNAL_ERROR::SET
                + CPEInterrupts::COMMAND_DONE::SET
                + CPEInterrupts::TX_DONE::SET
                + CPEInterrupts::BOOT_DONE::SET,
        );
        dbell_regs.rfcpe_ifg.set(0x0000);
        // self.enable_cpe_interrupts();
        // self.enable_hw_interrupts();

        // dbell_regs.rfcpeifg.set(0x7FFFFFFF);

        // Initialize radio module
        let cmd_init = cmd::DirectCommand::new(cmd::RFC_CMD0, 0x10 | 0x40);
        self.send_direct(&cmd_init)
            .ok()
            .expect("Could not initialize radio module");

        // TESTING clear ack flag register
        dbell_regs.rfack_ifg.set(0);

        // Request bus
        let cmd_bus_req = cmd::DirectCommand::new(cmd::RFC_BUS_REQUEST, 1);
        self.send_direct(&cmd_bus_req)
            .ok()
            .expect("Could not request bus on radio module");

        // TESTING clear ack flag register
        dbell_regs.rfack_ifg.set(0);

        // Ping radio module
        let cmd_ping = cmd::DirectCommand::new(cmd::RFC_PING, 0);
        self.send_direct(&cmd_ping)
            .ok()
            .expect("Could not ping radio module");

        // TESTING clear ack flag register
        dbell_regs.rfack_ifg.set(0);
    }

    pub fn check_enabled(&self) -> bool {
        // Ping radio module
        let cmd_ping = cmd::DirectCommand::new(cmd::RFC_PING, 0);
        self.send_direct(&cmd_ping).is_ok()
    }

    // Disable RFCore
    pub fn disable(&self) {
        let dbell_regs = &*self.dbell_regs;
        self.send_direct(&cmd::DirectCommand::new(cmd::RFC_STOP, 0))
            .ok()
            .expect("Could not stop RFC with direct command");

        dbell_regs.rfcpe_ien.set(0x00);
        dbell_regs.rfcpe_ifg.set(0x00);
        dbell_regs.rfcpe_isl.set(0x00);

        dbell_regs.rfack_ifg.set(0);

        let p_next_op = 0; // MAKE THIS POINTER TO NEXT CMD IN STACK FUTURE
        let start_time = 0; // CMD STARTS IMMEDIATELY
        let start_trigger = 0; // TRIGGER FOR NOW
        let condition = {
            let mut cond = cmd::RfcCondition(0);
            cond.set_rule(0x01);
            cond
        };
        let common =
            cmd::CmdCommon::new(0x080D, 0, p_next_op, start_time, start_trigger, condition);
        let mut fs_powerdown: cmd::CmdFSPowerdown = cmd::CmdFSPowerdown::new(common);
        fs_powerdown = cmd::RadioCommand::pack(&fs_powerdown, common);
        self.send(&fs_powerdown)
            .ok()
            .expect("Could not send power down command");

        self.stop_rat();

        // Add disable power domain and clocks

        self.mode.set(None);
    }

    // Call commands to setup RFCore with optional register overrides and power output
    pub fn setup(&self, reg_override: u32, tx_power: u16) {
        let mode = self.mode.get().expect("No RF mode selected, cannot setup");

        let p_next_op = 0; // MAKE THIS POINTER TO NEXT CMD IN STACK FUTURE
        let start_time = 0; // CMD STARTS IMMEDIATELY
        let start_trigger = 0; // TRIGGER FOR NOW
        let condition = {
            let mut cond = cmd::RfcCondition(0);
            cond.set_rule(0x01);
            cond
        };

        let common =
            cmd::CmdCommon::new(0x0802, 0, p_next_op, start_time, start_trigger, condition);

        let mut radio_setup =
            cmd::CmdRadioSetup::new(common, 0, reg_override, mode as u8, tx_power);
        radio_setup = cmd::RadioCommand::pack(&radio_setup, common);
        self.send(&radio_setup)
            .and_then(|_| self.wait(&radio_setup))
            .ok()
            .expect("Radio setup command returned Err");
    }

    pub fn setup_test(&self, reg_override: u32, tx_power: u16) {
        let mode = self.mode.get().expect("No RF mode selected, cannot setup");

        let dbell_regs = &*self.dbell_regs;
        let cmd = CommandRadioSetup {
            command_no: 0x0802,
            status: 0,
            p_nextop: 0,
            ratmr: 0,
            start_trigger: 0,
            condition: {
                let mut cond = cmd::RfcCondition(0);
                cond.set_rule(0x01); // COND_NEVER
                cond
            },
            mode: 0xFF,
            // mode: mode as u8,
            lo_divider: 0,
            config: {
                let mut cfg = cmd::RfcSetupConfig(0);
                cfg.set_frontend_mode(0); // Differential mode
                cfg.set_bias_mode(false); // Internal bias
                cfg
            },
            tx_power: tx_power,
            reg_override: 0,
        };

        self.send_test(&cmd).and_then(|_| self.wait_test(&cmd)).ok();
        // .expect("Radio setup command returned Err");
        dbell_regs.rfack_ifg.set(0);
    }

    pub fn start_rat(&self) {
        let p_next_op = 0; // MAKE THIS POINTER TO NEXT CMD IN STACK FUTURE
        let start_time = 0; // CMD STARTS IMMEDIATELY
        let start_trigger = 0; // TRIGGER FOR NOW
        let condition = {
            let mut cond = cmd::RfcCondition(0);
            cond.set_rule(0x01);
            cond
        };
        let common =
            cmd::CmdCommon::new(0x080A, 0, p_next_op, start_time, start_trigger, condition);

        let mut rf_command = cmd::CmdSyncStartRat::new(common, self.rat.get());
        rf_command = cmd::RadioCommand::pack(&rf_command, common);

        self.send(&rf_command)
            .and_then(|_| self.wait(&rf_command))
            .ok()
            .expect("Start RAT command returned Err");
    }

    pub fn start_rat_test(&self) {
        let dbell_regs = &*self.dbell_regs;

        let rf_command_test = CommandSyncRat {
            command_no: 0x080A,
            status: 0,
            next_op: 0,
            start_time: 0,
            start_trigger: 0,
            condition: {
                let mut cond = cmd::RfcCondition(0);
                cond.set_rule(0x01); // COND_NEVER
                cond
            },
            _reserved: 0,
            rat0: self.rat.get(),
        };

        self.send_test(&rf_command_test)
            .and_then(|_| self.wait_test(&rf_command_test))
            .ok()
            .expect("Start RAT command erturned Err");

        // REMOVE AFTER TESTING DONE
        dbell_regs.rfack_ifg.set(0);
        dbell_regs.rfcpe_ifg.set(0);
    }

    pub fn stop_rat(&self) {
        let p_next_op = 0; // MAKE THIS POINTER TO NEXT CMD IN STACK FUTURE
        let start_time = 0; // CMD STARTS IMMEDIATELY
        let start_trigger = 0; // TRIGGER FOR NOW
        let condition = {
            let mut cond = cmd::RfcCondition(0);
            cond.set_rule(0x01);
            cond
        };
        let common =
            cmd::CmdCommon::new(0x0809, 0, p_next_op, start_time, start_trigger, condition);

        let mut rf_command = cmd::CmdSyncStopRat::new(common, self.rat.get());
        rf_command = cmd::RadioCommand::pack(&rf_command, common);

        self.send(&rf_command)
            .and_then(|_| self.wait(&rf_command))
            .ok()
            .expect("Stop RAT command returned Err");
    }

    // Get current mode of RFCore
    pub fn current_mode(&self) -> Option<RfcMode> {
        self.mode.get()
    }

    pub fn status_ready(&self) -> bool {
        let status = self.status.get();
        match status {
            0x0400 => true,
            _ => false,
        }
    }

    // Set mode of RFCore
    pub fn set_mode(&self, mode: RfcMode) {
        let rf_mode = match mode {
            RfcMode::Common => 0x00,
            RfcMode::BLE => 0x01,
            _ => panic!("Only HAL mode supported"),
        };

        prcm::rf_mode_sel(rf_mode);

        self.mode.set(Some(mode))
    }

    // Post command pointer to CMDR register
    fn post_cmdr(&self, rf_command: u32) -> RadioReturnCode {
        let dbell_regs: &RfcDBellRegisters = &*self.dbell_regs;
        if !prcm::Power::is_enabled(prcm::PowerDomain::RFC) {
            panic!("RFC power domain is off");
        }

        dbell_regs.cmdr.set(rf_command);

        let mut status = 0;
        let mut timeout: u32 = 0;
        const MAX_TIMEOUT: u32 = 0x2FFFFFF;
        while timeout < MAX_TIMEOUT {
            status = dbell_regs.cmdsta.get();
            if (status & 0xFF) == 0x01 {
                return Ok(());
            }

            timeout += 1;
        }
        Err(status)
    }

    // Get status from active radio command
    pub fn wait_cmdr(&self, rf_command: u32) -> RadioReturnCode {
        let command_op: &CommandCommon = unsafe { &*(rf_command as *const CommandCommon) };
        let mut status = 0;
        let mut timeout: u32 = 0;
        const MAX_TIMEOUT: u32 = 0x2FFFFFF;
        while timeout < MAX_TIMEOUT {
            status = command_op.status.get();
            self.status.set(status.into());
            if status == 0x0400 {
                return Ok(());
            }
            timeout += 1;
        }
        Err(status as u32)
    }

    // Get status from CMDSTA register after ACK Interrupt flag has been thrown, then handle ACK
    // flag
    // Return CMDSTA register value
    pub fn cmdsta(&self) -> RadioReturnCode {
        let dbell_regs = &*self.dbell_regs;
        let status: u32 = dbell_regs.cmdsta.get();
        match status & 0xFF {
            0x00 => Err(status),
            0x01 => Ok(()),
            _ => Err(status),
        }
    }

    pub fn send<T: cmd::RadioCommand>(&self, rf_command: &T) -> RadioReturnCode {
        let command = { (rf_command as *const T) as u32 };

        self.post_cmdr(command)
    }

    pub fn send_test<T>(&self, rf_command: &T) -> RadioReturnCode {
        let command = (rf_command as *const T) as u32;

        self.post_cmdr(command)
    }

    pub fn send_direct(&self, dir_command: &cmd::DirectCommand) -> RadioReturnCode {
        let command = {
            let cmd = dir_command.command_no as u32;
            let par = dir_command.params as u32;
            (cmd << 16) | (par & 0xFFFC) | 1
        };

        self.post_cmdr(command)
    }

    pub fn wait<T: cmd::RadioCommand>(&self, rf_command: &T) -> RadioReturnCode {
        let command = { (rf_command as *const T) as u32 };

        return self.wait_cmdr(command);
    }

    pub fn wait_test<T>(&self, rf_command: &T) -> RadioReturnCode {
        let command = { (rf_command as *const T) as u32 };

        return self.wait_cmdr(command);
    }

    pub fn handle_interrupt(&self, int: RfcInterrupt) {
        let dbell_regs = &*self.dbell_regs;
        match int {
            // Hardware interrupt handler unimplemented
            RfcInterrupt::CmdAck => {
                // Clear the interrupt
                dbell_regs.rfack_ifg.set(0);
            }
            RfcInterrupt::Cpe0 => {
                let command_done = dbell_regs.rfcpe_ifg.is_set(CPEInterrupts::COMMAND_DONE);
                let last_command_done = dbell_regs
                    .rfcpe_ifg
                    .is_set(CPEInterrupts::LAST_COMMAND_DONE);
                let tx_done = dbell_regs.rfcpe_ifg.is_set(CPEInterrupts::TX_DONE);
                let rx_ok = dbell_regs.rfcpe_ifg.is_set(CPEInterrupts::RX_OK);
                dbell_regs.rfcpe_ifg.set(0);
                //self.cpe0_nvic.enable();

                if command_done || last_command_done {
                    self.client.get().map(|client| client.command_done());
                }
                if tx_done {
                    self.client.get().map(|client| client.tx_done());
                }
                if rx_ok {
                    self.client.get().map(|client| client.rx_ok());
                }
            }
            RfcInterrupt::Cpe1 => {
                dbell_regs.rfcpe_ifg.set(0x7FFFFFFF);
                panic!("Internal occurred during radio command!\r");
            }
            _ => panic!("Unhandled RFC interrupt: {}\r", int as u8),
        }
    }
}

pub trait RFCoreClient {
    fn command_done(&self);
    fn tx_done(&self);
    fn rx_ok(&self);
}

mod test_commands {
    use kernel::common::registers::ReadOnly;
    use radio::commands as cmd;

    #[allow(unused)]
    #[repr(C)]
    pub struct CommandCommon {
        pub command_no: ReadOnly<u16>,
        pub status: ReadOnly<u16>,
        pub p_nextop: ReadOnly<u32>,
        pub ratmr: ReadOnly<u32>,
        pub start_trigger: ReadOnly<u8>,
        pub condition: cmd::RfcCondition,
    }

    #[repr(C)]
    pub struct CommandSyncRat {
        pub command_no: u16,
        pub status: u16,
        pub next_op: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: cmd::RfcCondition,
        pub _reserved: u16,
        pub rat0: u32,
    }

    /* In order to properly setup the radio mode (e.g IEEE) */
    #[repr(C)]
    pub struct CommandRadioSetup {
        pub command_no: u16,
        pub status: u16,
        pub p_nextop: u32,
        pub ratmr: u32,
        pub start_trigger: u8,
        pub condition: cmd::RfcCondition,
        pub mode: u8,
        pub lo_divider: u8,
        pub config: cmd::RfcSetupConfig,
        pub tx_power: u16,
        pub reg_override: u32,
    }
}
