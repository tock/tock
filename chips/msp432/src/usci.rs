//! (enhanced) Universal Serial Communication Interface (USCI)

use kernel::common::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::common::StaticRef;

pub const USCI_A0_BASE: StaticRef<UsciARegisters> =
    unsafe { StaticRef::new(0x4000_1000 as *const UsciARegisters) };
#[allow(dead_code)]
pub(crate) const USCI_A1_BASE: StaticRef<UsciARegisters> =
    unsafe { StaticRef::new(0x4000_1400 as *const UsciARegisters) };
#[allow(dead_code)]
pub(crate) const USCI_A2_BASE: StaticRef<UsciARegisters> =
    unsafe { StaticRef::new(0x4000_1800 as *const UsciARegisters) };
#[allow(dead_code)]
pub(crate) const USCI_A3_BASE: StaticRef<UsciARegisters> =
    unsafe { StaticRef::new(0x4000_1C00 as *const UsciARegisters) };
#[allow(dead_code)]
pub(crate) const USCI_B0_BASE: StaticRef<UsciBRegisters> =
    unsafe { StaticRef::new(0x4000_2000 as *const UsciBRegisters) };
#[allow(dead_code)]
pub(crate) const USCI_B1_BASE: StaticRef<UsciBRegisters> =
    unsafe { StaticRef::new(0x4000_2400 as *const UsciBRegisters) };
#[allow(dead_code)]
pub(crate) const USCI_B2_BASE: StaticRef<UsciBRegisters> =
    unsafe { StaticRef::new(0x4000_2800 as *const UsciBRegisters) };
#[allow(dead_code)]
pub(crate) const USCI_B3_BASE: StaticRef<UsciBRegisters> =
    unsafe { StaticRef::new(0x4000_2C00 as *const UsciBRegisters) };

register_structs! {
    /// EUSCI_Ax
    pub UsciARegisters {
        /// eUSCI_Ax Control Word Register 0
        (0x00 => pub(crate) ctlw0: ReadWrite<u16, UCAxCTLW0::Register>),
        /// eUSCI_Ax Control Word Register 1
        (0x02 => pub(crate) ctlw1: ReadWrite<u16>),
        (0x04 => _reserved0),
        /// eUSCI_Ax Baud Rate Control Word Register
        (0x06 => pub(crate) brw: ReadWrite<u16>),
        /// eUSCI_Ax Modulation Control Word Register
        (0x08 => pub(crate) mctlw: ReadWrite<u16, UCAxMCTLW::Register>),
        /// eUSCI_Ax Status Register
        (0x0A => pub(crate) statw: ReadWrite<u16, UCAxSTATW::Register>),
        /// eUSCI_Ax Receive Buffer Register
        (0x0C => pub(crate) rxbuf: ReadOnly<u16>),
        /// eUSCI_Ax Transmit Buffer Register
        (0x0E => pub(crate) txbuf: ReadWrite<u16>),
        /// eUSCI_Ax Auto Baud Rate Control Register
        (0x10 => pub(crate) abctl: ReadWrite<u16, UCAxABCTL::Register>),
        /// eUSCI_Ax IrDA Control Word Register
        (0x12 => pub(crate) irctl: ReadWrite<u16, UCAxIRCTL::Register>),
        (0x14 => _reserved1),
        /// eUSCI_Ax Interrupt Enable Register
        (0x1A => pub(crate) ie: ReadWrite<u16, UCAxIE::Register>),
        /// eUSCI_Ax Interrupt Flag Register
        (0x1C => pub(crate) ifg: ReadWrite<u16, UCAxIFG::Register>),
        /// eUSCI_Ax Interrupt Vector Register
        (0x1E => pub(crate) iv: ReadOnly<u16>),
        (0x20 => @END),
    },
    /// EUSCI_Bx
    pub(crate) UsciBRegisters {
        /// eUSCI_Bx Control Word Register 0
        (0x00 => pub(crate) ctlw0: ReadWrite<u16, UCBxCTLW0::Register>),
        /// eUSCI_Bx Control Word Register 1
        (0x02 => pub(crate) ctlw1: ReadWrite<u16, UCBxCTLW1::Register>),
        (0x04 => _reserved0),
        /// eUSCI_Bx Baud Rate Control Word Register
        (0x06 => pub(crate) brw: ReadWrite<u16>),
        /// eUSCI_Bx Status Register
        (0x08 => pub(crate) statw: ReadWrite<u16, UCBxSTATW::Register>),
        /// eUSCI_Bx Byte Counter Threshold Register
        (0x0A => pub(crate) tbcnt: ReadWrite<u16>),
        /// eUSCI_Bx Receive Buffer Register
        (0x0C => pub(crate) rxbuf: ReadOnly<u16>),
        /// eUSCI_Bx Transmit Buffer Register
        (0x0E => pub(crate) txbuf: ReadWrite<u16>),
        (0x10 => _reserved1),
        /// eUSCI_Bx I2C Own Address 0 Register
        (0x14 => pub(crate) i2coa0: ReadWrite<u16, UCBxI2COA0::Register>),
        /// eUSCI_Bx I2C Own Address 1 Register
        (0x16 => pub(crate) i2coa1: ReadWrite<u16, UCBxI2COA1::Register>),
        /// eUSCI_Bx I2C Own Address 2 Register
        (0x18 => pub(crate) i2coa2: ReadWrite<u16, UCBxI2COA2::Register>),
        /// eUSCI_Bx I2C Own Address 3 Register
        (0x1A => pub(crate) i2coa3: ReadWrite<u16, UCBxI2COA3::Register>),
        /// eUSCI_Bx I2C Received Address Register
        (0x1C => pub(crate) addrx: ReadOnly<u16>),
        /// eUSCI_Bx I2C Address Mask Register
        (0x1E => pub(crate) addmask: ReadWrite<u16>),
        /// eUSCI_Bx I2C Slave Address Register
        (0x20 => pub(crate) i2csa: ReadWrite<u16>),
        (0x22 => _reserved2),
        /// eUSCI_Bx Interrupt Enable Register
        (0x2A => pub(crate) ie: ReadWrite<u16, UCBxIE::Register>),
        /// eUSCI_Bx Interrupt Flag Register
        (0x2C => pub(crate) ifg: ReadWrite<u16, UCBxIFG::Register>),
        /// eUSCI_Bx Interrupt Vector Register
        (0x2E => pub(crate) iv: ReadOnly<u16>),
        (0x30 => @END),
    }
}

register_bitfields![u16,
    pub(crate) UCAxCTLW0 [
        /// Software reset enable
        UCSWRST OFFSET(0) NUMBITS(1) [
            /// Disabled. eUSCI_A reset released for operation
            DisabledEUSCI_AResetReleasedForOperation = 0,
            /// Enabled. eUSCI_A logic held in reset state
            EnabledEUSCI_ALogicHeldInResetState = 1
        ],
        /// Transmit break
        UCTXBRK OFFSET(1) NUMBITS(1) [
            /// Next frame transmitted is not a break
            NextFrameTransmittedIsNotABreak = 0,
            /// Next frame transmitted is a break or a break/synch
            NextFrameTransmittedIsABreakOrABreakSynch = 1
        ],
        /// Transmit address
        UCTXADDR OFFSET(2) NUMBITS(1) [
            /// Next frame transmitted is data
            NextFrameTransmittedIsData = 0,
            /// Next frame transmitted is an address
            NextFrameTransmittedIsAnAddress = 1
        ],
        /// Dormant
        UCDORM OFFSET(3) NUMBITS(1) [
            /// Not dormant. All received characters set UCRXIFG.
            NotDormantAllReceivedCharactersSetUCRXIFG = 0,
            /// Dormant. Only characters that are preceded by an idle-line or with address bit s
            UCDORM_1 = 1
        ],
        /// Receive break character interrupt enable
        UCBRKIE OFFSET(4) NUMBITS(1) [
            /// Received break characters do not set UCRXIFG
            ReceivedBreakCharactersDoNotSetUCRXIFG = 0,
            /// Received break characters set UCRXIFG
            ReceivedBreakCharactersSetUCRXIFG = 1
        ],
        /// Receive erroneous-character interrupt enable
        UCRXEIE OFFSET(5) NUMBITS(1) [
            /// Erroneous characters rejected and UCRXIFG is not set
            ErroneousCharactersRejectedAndUCRXIFGIsNotSet = 0,
            /// Erroneous characters received set UCRXIFG
            ErroneousCharactersReceivedSetUCRXIFG = 1
        ],
        /// eUSCI_A clock source select
        UCSSEL OFFSET(6) NUMBITS(2) [
            /// UCLK
            UCLK = 0,
            /// ACLK
            ACLK = 1,
            /// SMCLK
            SMCLK = 2
        ],
        /// Synchronous mode enable
        UCSYNC OFFSET(8) NUMBITS(1) [
            /// Asynchronous mode
            AsynchronousMode = 0,
            /// Synchronous mode
            SynchronousMode = 1
        ],
        /// eUSCI_A mode
        UCMODE OFFSET(9) NUMBITS(2) [
            /// UART mode
            UARTMode = 0,
            /// Idle-line multiprocessor mode
            IdleLineMultiprocessorMode = 1,
            /// Address-bit multiprocessor mode
            AddressBitMultiprocessorMode = 2,
            /// UART mode with automatic baud-rate detection
            UARTModeWithAutomaticBaudRateDetection = 3
        ],
        /// Stop bit select
        UCSPB OFFSET(11) NUMBITS(1) [
            /// One stop bit
            OneStopBit = 0,
            /// Two stop bits
            TwoStopBits = 1
        ],
        /// Character length
        UC7BIT OFFSET(12) NUMBITS(1) [
            /// 8-bit data
            _8BitData = 0,
            /// 7-bit data
            _7BitData = 1
        ],
        /// MSB first select
        UCMSB OFFSET(13) NUMBITS(1) [
            /// LSB first
            LSBFirst = 0,
            /// MSB first
            MSBFirst = 1
        ],
        /// Parity select
        UCPAR OFFSET(14) NUMBITS(1) [
            /// Odd parity
            OddParity = 0,
            /// Even parity
            EvenParity = 1
        ],
        /// Parity enable
        UCPEN OFFSET(15) NUMBITS(1) [
            /// Parity disabled
            ParityDisabled = 0,
            /// Parity enabled. Parity bit is generated (UCAxTXD) and expected (UCAxRXD). In add
            UCPEN_1 = 1
        ]
    ],

    pub(crate) UCAxCTLW1 [
        /// Deglitch time
        UCGLIT OFFSET(0) NUMBITS(2) [
            /// Approximately 5ns
            _5ns = 0,
            /// Approximately 20ns
            _20ns = 1,
            /// Approximately 30ns
            _30ns = 2,
            /// Approximately 50ns
            _50ns = 3
        ]
    ],
    pub(crate) UCAxMCTLW [
        /// Oversampling mode enabled
        UCOS16 OFFSET(0) NUMBITS(1) [
            /// Disabled
            Disabled = 0,
            /// Enabled
            Enabled = 1
        ],
        /// First modulation stage select
        UCBRF OFFSET(4) NUMBITS(4) [],
        /// Second modulation stage select
        UCBRS OFFSET(8) NUMBITS(8) []
    ],
    pub(crate) UCAxSTATW [
        /// eUSCI_A busy
        UCBUSY OFFSET(0) NUMBITS(1) [
            /// eUSCI_A inactive
            EUSCI_AInactive = 0,
            /// eUSCI_A transmitting or receiving
            EUSCI_ATransmittingOrReceiving = 1
        ],
        /// Address received / Idle line detected
        UCADDR_UCIDLE OFFSET(1) NUMBITS(1) [],
        /// Receive error flag
        UCRXERR OFFSET(2) NUMBITS(1) [
            /// No receive errors detected
            NoReceiveErrorsDetected = 0,
            /// Receive error detected
            ReceiveErrorDetected = 1
        ],
        /// Break detect flag
        UCBRK OFFSET(3) NUMBITS(1) [
            /// No break condition
            NoBreakCondition = 0,
            /// Break condition occurred
            BreakConditionOccurred = 1
        ],
        /// Parity error flag. When UCPEN = 0, UCPE is read as 0. UCPE is cleared when UCAxR
        UCPE OFFSET(4) NUMBITS(1) [
            /// No error
            NoError = 0,
            /// Character received with parity error
            CharacterReceivedWithParityError = 1
        ],
        /// Overrun error flag
        UCOE OFFSET(5) NUMBITS(1) [
            /// No error
            NoError = 0,
            /// Overrun error occurred
            OverrunErrorOccurred = 1
        ],
        /// Framing error flag
        UCFE OFFSET(6) NUMBITS(1) [
            /// No error
            NoError = 0,
            /// Character received with low stop bit
            CharacterReceivedWithLowStopBit = 1
        ],
        /// Listen enable
        UCLISTEN OFFSET(7) NUMBITS(1) [
            /// Disabled
            Disabled = 0,
            /// Enabled. UCAxTXD is internally fed back to the receiver
            EnabledUCAxTXDIsInternallyFedBackToTheReceiver = 1
        ]
    ],
    pub(crate) UCAxABCTL [
        /// Automatic baud-rate detect enable
        UCABDEN OFFSET(0) NUMBITS(1) [
            /// Baud-rate detection disabled. Length of break and synch field is not measured.
            BaudRateDetectionDisabledLengthOfBreakAndSynchFieldIsNotMeasured = 0,
            /// Baud-rate detection enabled. Length of break and synch field is measured and bau
            UCABDEN_1 = 1
        ],
        /// Break time out error
        UCBTOE OFFSET(2) NUMBITS(1) [
            /// No error
            NoError = 0,
            /// Length of break field exceeded 22 bit times
            LengthOfBreakFieldExceeded22BitTimes = 1
        ],
        /// Synch field time out error
        UCSTOE OFFSET(3) NUMBITS(1) [
            /// No error
            NoError = 0,
            /// Length of synch field exceeded measurable time
            LengthOfSynchFieldExceededMeasurableTime = 1
        ],
        /// Break/synch delimiter length
        UCDELIM OFFSET(4) NUMBITS(2) [
            /// 1 bit time
            _1BitTime = 0,
            /// 2 bit times
            _2BitTimes = 1,
            /// 3 bit times
            _3BitTimes = 2,
            /// 4 bit times
            _4BitTimes = 3
        ]
    ],
    pub(crate) UCAxIRCTL [
        /// IrDA encoder/decoder enable
        UCIREN OFFSET(0) NUMBITS(1) [
            /// IrDA encoder/decoder disabled
            IrDAEncoderDecoderDisabled = 0,
            /// IrDA encoder/decoder enabled
            IrDAEncoderDecoderEnabled = 1
        ],
        /// IrDA transmit pulse clock select
        UCIRTXCLK OFFSET(1) NUMBITS(1) [
            /// BRCLK
            BRCLK = 0,
            /// BITCLK16 when UCOS16 = 1. Otherwise, BRCLK.
            BITCLK16WhenUCOS161OtherwiseBRCLK = 1
        ],
        /// Transmit pulse length
        UCIRTXPL OFFSET(2) NUMBITS(6) [],
        /// IrDA receive filter enabled
        UCIRRXFE OFFSET(8) NUMBITS(1) [
            /// Receive filter disabled
            ReceiveFilterDisabled = 0,
            /// Receive filter enabled
            ReceiveFilterEnabled = 1
        ],
        /// IrDA receive input UCAxRXD polarity
        UCIRRXPL OFFSET(9) NUMBITS(1) [
            /// IrDA transceiver delivers a high pulse when a light pulse is seen
            IrDATransceiverDeliversAHighPulseWhenALightPulseIsSeen = 0,
            /// IrDA transceiver delivers a low pulse when a light pulse is seen
            IrDATransceiverDeliversALowPulseWhenALightPulseIsSeen = 1
        ],
        /// Receive filter length
        UCIRRXFL OFFSET(10) NUMBITS(4) []
    ],
    pub(crate) UCAxIE [
        /// Receive interrupt enable
        UCRXIE OFFSET(0) NUMBITS(1) [
            /// Interrupt disabled
            InterruptDisabled = 0,
            /// Interrupt enabled
            InterruptEnabled = 1
        ],
        /// Transmit interrupt enable
        UCTXIE OFFSET(1) NUMBITS(1) [
            /// Interrupt disabled
            InterruptDisabled = 0,
            /// Interrupt enabled
            InterruptEnabled = 1
        ],
        /// Start bit interrupt enable
        UCSTTIE OFFSET(2) NUMBITS(1) [
            /// Interrupt disabled
            InterruptDisabled = 0,
            /// Interrupt enabled
            InterruptEnabled = 1
        ],
        /// Transmit complete interrupt enable
        UCTXCPTIE OFFSET(3) NUMBITS(1) [
            /// Interrupt disabled
            InterruptDisabled = 0,
            /// Interrupt enabled
            InterruptEnabled = 1
        ]
    ],
    pub(crate) UCAxIFG [
        /// Receive interrupt flag
        UCRXIFG OFFSET(0) NUMBITS(1) [
            /// No interrupt pending
            NoInterruptPending = 0,
            /// Interrupt pending
            InterruptPending = 1
        ],
        /// Transmit interrupt flag
        UCTXIFG OFFSET(1) NUMBITS(1) [
            /// No interrupt pending
            NoInterruptPending = 0,
            /// Interrupt pending
            InterruptPending = 1
        ],
        /// Start bit interrupt flag
        UCSTTIFG OFFSET(2) NUMBITS(1) [
            /// No interrupt pending
            NoInterruptPending = 0,
            /// Interrupt pending
            InterruptPending = 1
        ],
        /// Transmit ready interrupt enable
        UCTXCPTIFG OFFSET(3) NUMBITS(1) [
            /// No interrupt pending
            NoInterruptPending = 0,
            /// Interrupt pending
            InterruptPending = 1
        ]
    ],
    pub(crate) UCAxIV [
        UCIV OFFSET(0) NUMBITS(16) [
            NoInterrupt = 0,
            ReceiveBufferFull = 2,
            TransmitBufferEmpty = 4,
            StartBitReceived = 6,
            TransmitComplete = 8
        ]
    ]
];

register_bitfields![u16,
    pub(crate) UCBxCTLW0 [
        /// Software reset enable
        UCSWRST OFFSET(0) NUMBITS(1) [
            /// Disabled. eUSCI_B reset released for operation
            DisabledEUSCI_BResetReleasedForOperation = 0,
            /// Enabled. eUSCI_B logic held in reset state
            EnabledEUSCI_BLogicHeldInResetState = 1
        ],
        /// Transmit START condition in master mode
        UCTXSTT OFFSET(1) NUMBITS(1) [
            /// Do not generate START condition
            DoNotGenerateSTARTCondition = 0,
            /// Generate START condition
            GenerateSTARTCondition = 1
        ],
        /// Transmit STOP condition in master mode
        UCTXSTP OFFSET(2) NUMBITS(1) [
            /// No STOP generated
            NoSTOPGenerated = 0,
            /// Generate STOP
            GenerateSTOP = 1
        ],
        /// Transmit a NACK
        UCTXNACK OFFSET(3) NUMBITS(1) [
            /// Acknowledge normally
            AcknowledgeNormally = 0,
            /// Generate NACK
            GenerateNACK = 1
        ],
        /// Transmitter/receiver
        UCTR OFFSET(4) NUMBITS(1) [
            /// Receiver
            Receiver = 0,
            /// Transmitter
            Transmitter = 1
        ],
        /// Transmit ACK condition in slave mode
        UCTXACK OFFSET(5) NUMBITS(1) [
            /// Do not acknowledge the slave address
            DoNotAcknowledgeTheSlaveAddress = 0,
            /// Acknowledge the slave address
            AcknowledgeTheSlaveAddress = 1
        ],
        /// eUSCI_B clock source select
        UCSSEL OFFSET(6) NUMBITS(2) [
            /// UCLKI
            UCLKI = 0,
            /// ACLK
            ACLK = 1,
            /// SMCLK
            SMCLK = 2
        ],
        /// Synchronous mode enable
        UCSYNC OFFSET(8) NUMBITS(1) [
            /// Asynchronous mode
            AsynchronousMode = 0,
            /// Synchronous mode
            SynchronousMode = 1
        ],
        /// eUSCI_B mode
        UCMODE OFFSET(9) NUMBITS(2) [
            /// 3-pin SPI
            _3PinSPI = 0,
            /// 4-pin SPI (master or slave enabled if STE = 1)
            _4PinSPIMasterOrSlaveEnabledIfSTE1 = 1,
            /// 4-pin SPI (master or slave enabled if STE = 0)
            _4PinSPIMasterOrSlaveEnabledIfSTE0 = 2,
            /// I2C mode
            I2CMode = 3
        ],
        /// Master mode select
        UCMST OFFSET(11) NUMBITS(1) [
            /// Slave mode
            SlaveMode = 0,
            /// Master mode
            MasterMode = 1
        ],
        /// Multi-master environment select
        UCMM OFFSET(13) NUMBITS(1) [
            /// Single master environment. There is no other master in the system. The address c
            UCMM_0 = 0,
            /// Multi-master environment
            MultiMasterEnvironment = 1
        ],
        /// Slave addressing mode select
        UCSLA10 OFFSET(14) NUMBITS(1) [
            /// Address slave with 7-bit address
            AddressSlaveWith7BitAddress = 0,
            /// Address slave with 10-bit address
            AddressSlaveWith10BitAddress = 1
        ],
        /// Own addressing mode select
        UCA10 OFFSET(15) NUMBITS(1) [
            /// Own address is a 7-bit address
            OwnAddressIsA7BitAddress = 0,
            /// Own address is a 10-bit address
            OwnAddressIsA10BitAddress = 1
        ]
    ],
    pub(crate) UCBxCTLW1 [
        /// Deglitch time
        UCGLIT OFFSET(0) NUMBITS(2) [
            /// 50 ns
            _50Ns = 0,
            /// 25 ns
            _25Ns = 1,
            /// 12.5 ns
            _125Ns = 2,
            /// 6.25 ns
            _625Ns = 3
        ],
        /// Automatic STOP condition generation
        UCASTP OFFSET(2) NUMBITS(2) [
            /// No automatic STOP generation. The STOP condition is generated after the user set
            UCASTP_0 = 0,
            /// UCBCNTIFG is set with the byte counter reaches the threshold defined in UCBxTBCN
            UCBCNTIFGIsSetWithTheByteCounterReachesTheThresholdDefinedInUCBxTBCNT = 1,
            /// A STOP condition is generated automatically after the byte counter value reached
            UCASTP_2 = 2
        ],
        /// SW or HW ACK control
        UCSWACK OFFSET(4) NUMBITS(1) [
            /// The address acknowledge of the slave is controlled by the eUSCI_B module
            TheAddressAcknowledgeOfTheSlaveIsControlledByTheEUSCI_BModule = 0,
            /// The user needs to trigger the sending of the address ACK by issuing UCTXACK
            TheUserNeedsToTriggerTheSendingOfTheAddressACKByIssuingUCTXACK = 1
        ],
        /// ACK all master bytes
        UCSTPNACK OFFSET(5) NUMBITS(1) [
            /// Send a non-acknowledge before the STOP condition as a master receiver (conform t
            SendANonAcknowledgeBeforeTheSTOPConditionAsAMasterReceiverConformToI2CStandard = 0,
            /// All bytes are acknowledged by the eUSCI_B when configured as master receiver
            AllBytesAreAcknowledgedByTheEUSCI_BWhenConfiguredAsMasterReceiver = 1
        ],
        /// Clock low timeout select
        UCCLTO OFFSET(6) NUMBITS(2) [
            /// Disable clock low timeout counter
            DisableClockLowTimeoutCounter = 0,
            /// 135 000 SYSCLK cycles (approximately 28 ms)
            _135000SYSCLKCyclesApproximately28Ms = 1,
            /// 150 000 SYSCLK cycles (approximately 31 ms)
            _150000SYSCLKCyclesApproximately31Ms = 2,
            /// 165 000 SYSCLK cycles (approximately 34 ms)
            _165000SYSCLKCyclesApproximately34Ms = 3
        ],
        /// Early UCTXIFG0
        UCETXINT OFFSET(8) NUMBITS(1) [
            /// UCTXIFGx is set after an address match with UCxI2COAx and the direction bit indi
            UCETXINT_0 = 0,
            /// UCTXIFG0 is set for each START condition
            UCTXIFG0IsSetForEachSTARTCondition = 1
        ]
    ],
    pub(crate) UCBxSTATW [
        /// Bus busy
        UCBBUSY OFFSET(4) NUMBITS(1) [
            /// Bus inactive
            BusInactive = 0,
            /// Bus busy
            BusBusy = 1
        ],
        /// General call address received
        UCGC OFFSET(5) NUMBITS(1) [
            /// No general call address received
            NoGeneralCallAddressReceived = 0,
            /// General call address received
            GeneralCallAddressReceived = 1
        ],
        /// SCL low
        UCSCLLOW OFFSET(6) NUMBITS(1) [
            /// SCL is not held low
            SCLIsNotHeldLow = 0,
            /// SCL is held low
            SCLIsHeldLow = 1
        ],
        /// Hardware byte counter value
        UCBCNT OFFSET(8) NUMBITS(8) []
    ],
    pub(crate) UCBxI2COA0 [
        /// I2C own address
        I2COA0 OFFSET(0) NUMBITS(10) [],
        /// Own Address enable register
        UCOAEN OFFSET(10) NUMBITS(1) [
            /// The slave address defined in I2COA0 is disabled
            TheSlaveAddressDefinedInI2COA0IsDisabled = 0,
            /// The slave address defined in I2COA0 is enabled
            TheSlaveAddressDefinedInI2COA0IsEnabled = 1
        ],
        /// General call response enable
        UCGCEN OFFSET(15) NUMBITS(1) [
            /// Do not respond to a general call
            DoNotRespondToAGeneralCall = 0,
            /// Respond to a general call
            RespondToAGeneralCall = 1
        ]
    ],
    pub(crate) UCBxI2COA1 [
        /// I2C own address
        I2COA1 OFFSET(0) NUMBITS(10) [],
        /// Own Address enable register
        UCOAEN OFFSET(10) NUMBITS(1) [
            /// The slave address defined in I2COA1 is disabled
            TheSlaveAddressDefinedInI2COA1IsDisabled = 0,
            /// The slave address defined in I2COA1 is enabled
            TheSlaveAddressDefinedInI2COA1IsEnabled = 1
        ]
    ],
    pub(crate) UCBxI2COA2 [
        /// I2C own address
        I2COA2 OFFSET(0) NUMBITS(10) [],
        /// Own Address enable register
        UCOAEN OFFSET(10) NUMBITS(1) [
            /// The slave address defined in I2COA2 is disabled
            TheSlaveAddressDefinedInI2COA2IsDisabled = 0,
            /// The slave address defined in I2COA2 is enabled
            TheSlaveAddressDefinedInI2COA2IsEnabled = 1
        ]
    ],
    pub(crate) UCBxI2COA3 [
        /// I2C own address
        I2COA3 OFFSET(0) NUMBITS(10) [],
        /// Own Address enable register
        UCOAEN OFFSET(10) NUMBITS(1) [
            /// The slave address defined in I2COA3 is disabled
            TheSlaveAddressDefinedInI2COA3IsDisabled = 0,
            /// The slave address defined in I2COA3 is enabled
            TheSlaveAddressDefinedInI2COA3IsEnabled = 1
        ]
    ],
    pub(crate) UCBxIE [
        /// Receive interrupt enable 0
        UCRXIE0 OFFSET(0) NUMBITS(1) [
            /// Interrupt disabled
            InterruptDisabled = 0,
            /// Interrupt enabled
            InterruptEnabled = 1
        ],
        /// Transmit interrupt enable 0
        UCTXIE0 OFFSET(1) NUMBITS(1) [
            /// Interrupt disabled
            InterruptDisabled = 0,
            /// Interrupt enabled
            InterruptEnabled = 1
        ],
        /// START condition interrupt enable
        UCSTTIE OFFSET(2) NUMBITS(1) [
            /// Interrupt disabled
            InterruptDisabled = 0,
            /// Interrupt enabled
            InterruptEnabled = 1
        ],
        /// STOP condition interrupt enable
        UCSTPIE OFFSET(3) NUMBITS(1) [
            /// Interrupt disabled
            InterruptDisabled = 0,
            /// Interrupt enabled
            InterruptEnabled = 1
        ],
        /// Arbitration lost interrupt enable
        UCALIE OFFSET(4) NUMBITS(1) [
            /// Interrupt disabled
            InterruptDisabled = 0,
            /// Interrupt enabled
            InterruptEnabled = 1
        ],
        /// Not-acknowledge interrupt enable
        UCNACKIE OFFSET(5) NUMBITS(1) [
            /// Interrupt disabled
            InterruptDisabled = 0,
            /// Interrupt enabled
            InterruptEnabled = 1
        ],
        /// Byte counter interrupt enable
        UCBCNTIE OFFSET(6) NUMBITS(1) [
            /// Interrupt disabled
            InterruptDisabled = 0,
            /// Interrupt enabled
            InterruptEnabled = 1
        ],
        /// Clock low timeout interrupt enable
        UCCLTOIE OFFSET(7) NUMBITS(1) [
            /// Interrupt disabled
            InterruptDisabled = 0,
            /// Interrupt enabled
            InterruptEnabled = 1
        ],
        /// Receive interrupt enable 1
        UCRXIE1 OFFSET(8) NUMBITS(1) [
            /// Interrupt disabled
            InterruptDisabled = 0,
            /// Interrupt enabled
            InterruptEnabled = 1
        ],
        /// Transmit interrupt enable 1
        UCTXIE1 OFFSET(9) NUMBITS(1) [
            /// Interrupt disabled
            InterruptDisabled = 0,
            /// Interrupt enabled
            InterruptEnabled = 1
        ],
        /// Receive interrupt enable 2
        UCRXIE2 OFFSET(10) NUMBITS(1) [
            /// Interrupt disabled
            InterruptDisabled = 0,
            /// Interrupt enabled
            InterruptEnabled = 1
        ],
        /// Transmit interrupt enable 2
        UCTXIE2 OFFSET(11) NUMBITS(1) [
            /// Interrupt disabled
            InterruptDisabled = 0,
            /// Interrupt enabled
            InterruptEnabled = 1
        ],
        /// Receive interrupt enable 3
        UCRXIE3 OFFSET(12) NUMBITS(1) [
            /// Interrupt disabled
            InterruptDisabled = 0,
            /// Interrupt enabled
            InterruptEnabled = 1
        ],
        /// Transmit interrupt enable 3
        UCTXIE3 OFFSET(13) NUMBITS(1) [
            /// Interrupt disabled
            InterruptDisabled = 0,
            /// Interrupt enabled
            InterruptEnabled = 1
        ],
        /// Bit position 9 interrupt enable
        UCBIT9IE OFFSET(14) NUMBITS(1) [
            /// Interrupt disabled
            InterruptDisabled = 0,
            /// Interrupt enabled
            InterruptEnabled = 1
        ]
    ],
    pub(crate) UCBxIFG [
        /// eUSCI_B receive interrupt flag 0
        UCRXIFG0 OFFSET(0) NUMBITS(1) [
            /// No interrupt pending
            NoInterruptPending = 0,
            /// Interrupt pending
            InterruptPending = 1
        ],
        /// eUSCI_B transmit interrupt flag 0
        UCTXIFG0 OFFSET(1) NUMBITS(1) [
            /// No interrupt pending
            NoInterruptPending = 0,
            /// Interrupt pending
            InterruptPending = 1
        ],
        /// START condition interrupt flag
        UCSTTIFG OFFSET(2) NUMBITS(1) [
            /// No interrupt pending
            NoInterruptPending = 0,
            /// Interrupt pending
            InterruptPending = 1
        ],
        /// STOP condition interrupt flag
        UCSTPIFG OFFSET(3) NUMBITS(1) [
            /// No interrupt pending
            NoInterruptPending = 0,
            /// Interrupt pending
            InterruptPending = 1
        ],
        /// Arbitration lost interrupt flag
        UCALIFG OFFSET(4) NUMBITS(1) [
            /// No interrupt pending
            NoInterruptPending = 0,
            /// Interrupt pending
            InterruptPending = 1
        ],
        /// Not-acknowledge received interrupt flag
        UCNACKIFG OFFSET(5) NUMBITS(1) [
            /// No interrupt pending
            NoInterruptPending = 0,
            /// Interrupt pending
            InterruptPending = 1
        ],
        /// Byte counter interrupt flag
        UCBCNTIFG OFFSET(6) NUMBITS(1) [
            /// No interrupt pending
            NoInterruptPending = 0,
            /// Interrupt pending
            InterruptPending = 1
        ],
        /// Clock low timeout interrupt flag
        UCCLTOIFG OFFSET(7) NUMBITS(1) [
            /// No interrupt pending
            NoInterruptPending = 0,
            /// Interrupt pending
            InterruptPending = 1
        ],
        /// eUSCI_B receive interrupt flag 1
        UCRXIFG1 OFFSET(8) NUMBITS(1) [
            /// No interrupt pending
            NoInterruptPending = 0,
            /// Interrupt pending
            InterruptPending = 1
        ],
        /// eUSCI_B transmit interrupt flag 1
        UCTXIFG1 OFFSET(9) NUMBITS(1) [
            /// No interrupt pending
            NoInterruptPending = 0,
            /// Interrupt pending
            InterruptPending = 1
        ],
        /// eUSCI_B receive interrupt flag 2
        UCRXIFG2 OFFSET(10) NUMBITS(1) [
            /// No interrupt pending
            NoInterruptPending = 0,
            /// Interrupt pending
            InterruptPending = 1
        ],
        /// eUSCI_B transmit interrupt flag 2
        UCTXIFG2 OFFSET(11) NUMBITS(1) [
            /// No interrupt pending
            NoInterruptPending = 0,
            /// Interrupt pending
            InterruptPending = 1
        ],
        /// eUSCI_B receive interrupt flag 3
        UCRXIFG3 OFFSET(12) NUMBITS(1) [
            /// No interrupt pending
            NoInterruptPending = 0,
            /// Interrupt pending
            InterruptPending = 1
        ],
        /// eUSCI_B transmit interrupt flag 3
        UCTXIFG3 OFFSET(13) NUMBITS(1) [
            /// No interrupt pending
            NoInterruptPending = 0,
            /// Interrupt pending
            InterruptPending = 1
        ],
        /// Bit position 9 interrupt flag
        UCBIT9IFG OFFSET(14) NUMBITS(1) [
            /// No interrupt pending
            NoInterruptPending = 0,
            /// Interrupt pending
            InterruptPending = 1
        ]
    ],
    pub(crate) UCBxIV [
        UCIV OFFSET(0) NUMBITS(16) [
            NoInterrupt = 0x00,
            ArbitrationLost = 0x02,
            NoAck = 0x04,
            ReceivedStartCondition = 0x06,
            ReceivedStopCondition = 0x08,
            Slave3DataReceived = 0x0A,
            Slave3TransmitBufferEmpty = 0x0C,
            Slave2DataReceived = 0x0E,
            Slave2TransmitBufferEmpty = 0x10,
            Slave1DataReceived = 0x12,
            Slave1TransmitBufferEmpty = 0x14,
            DataReceived = 0x16,
            TransmitBufferEmpty = 0x18,
            ByteCounterZero = 0x1A,
            ClockLowTimeout = 0x1C,
            NinethBitPosition = 0x1E
        ]
    ]
];
