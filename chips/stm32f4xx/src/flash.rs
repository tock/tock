use kernel::utilities::StaticRef;
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite, WriteOnly};
use kernel::utilities::registers::interfaces::{Readable, ReadWriteable};

register_structs! {
    /// FLASH
    FlashRegisters {
        /// Flash access control register
        (0x000 => acr: ReadWrite<u32, ACR::Register>),
        /// Flash key register
        (0x004 => keyr: WriteOnly<u32>),
        /// Flash option key register
        (0x008 => optkeyr: WriteOnly<u32>),
        /// Status register
        (0x00C => sr: ReadWrite<u32, SR::Register>),
        /// Control register
        (0x010 => cr: ReadWrite<u32, CR::Register>),
        /// Flash option control register
        (0x014 => optcr: ReadWrite<u32, OPTCR::Register>),
        /// Flash option control register
/// 1
        (0x018 => optcr1: ReadWrite<u32>),
        (0x01C => @END),
    }
}

register_bitfields![u32,
    ACR [
        /// Latency
        LATENCY OFFSET(0) NUMBITS(3) [],
        /// Prefetch enable
        PRFTEN OFFSET(8) NUMBITS(1) [],
        /// Instruction cache enable
        ICEN OFFSET(9) NUMBITS(1) [],
        /// Data cache enable
        DCEN OFFSET(10) NUMBITS(1) [],
        /// Instruction cache reset
        ICRST OFFSET(11) NUMBITS(1) [],
        /// Data cache reset
        DCRST OFFSET(12) NUMBITS(1) []
    ],
    KEYR [
        /// FPEC key
        KEY OFFSET(0) NUMBITS(32) []
    ],
    OPTKEYR [
        /// Option byte key
        OPTKEY OFFSET(0) NUMBITS(32) []
    ],
    SR [
        /// End of operation
        EOP OFFSET(0) NUMBITS(1) [],
        /// Operation error
        OPERR OFFSET(1) NUMBITS(1) [],
        /// Write protection error
        WRPERR OFFSET(4) NUMBITS(1) [],
        /// Programming alignment error
        PGAERR OFFSET(5) NUMBITS(1) [],
        /// Programming parallelism error
        PGPERR OFFSET(6) NUMBITS(1) [],
        /// Programming sequence error
        PGSERR OFFSET(7) NUMBITS(1) [],
        /// Busy
        BSY OFFSET(16) NUMBITS(1) []
    ],
    CR [
        /// Programming
        PG OFFSET(0) NUMBITS(1) [],
        /// Sector Erase
        SER OFFSET(1) NUMBITS(1) [],
        /// Mass Erase of sectors 0 to 11
        MER OFFSET(2) NUMBITS(1) [],
        /// Sector number
        SNB OFFSET(3) NUMBITS(5) [],
        /// Program size
        PSIZE OFFSET(8) NUMBITS(2) [],
        /// Mass Erase of sectors 12 to 23
        MER1 OFFSET(15) NUMBITS(1) [],
        /// Start
        STRT OFFSET(16) NUMBITS(1) [],
        /// End of operation interrupt enable
        EOPIE OFFSET(24) NUMBITS(1) [],
        /// Error interrupt enable
        ERRIE OFFSET(25) NUMBITS(1) [],
        /// Lock
        LOCK OFFSET(31) NUMBITS(1) []
    ],
    OPTCR [
        /// Option lock
        OPTLOCK OFFSET(0) NUMBITS(1) [],
        /// Option start
        OPTSTRT OFFSET(1) NUMBITS(1) [],
        /// BOR reset Level
        BOR_LEV OFFSET(2) NUMBITS(2) [],
        /// WDG_SW User option bytes
        WDG_SW OFFSET(5) NUMBITS(1) [],
        /// nRST_STOP User option bytes
        nRST_STOP OFFSET(6) NUMBITS(1) [],
        /// nRST_STDBY User option bytes
        nRST_STDBY OFFSET(7) NUMBITS(1) [],
        /// Read protect
        RDP OFFSET(8) NUMBITS(8) [],
        /// Not write protect
        nWRP OFFSET(16) NUMBITS(12) []
    ],
    OPTCR1 [
        /// Not write protect
        nWRP OFFSET(16) NUMBITS(12) []
    ]
];

const FLASH_BASE: StaticRef<FlashRegisters> =
    unsafe { StaticRef::new(0x40023C00 as *const FlashRegisters) };

pub struct Flash {
    registers: StaticRef<FlashRegisters>,
}

pub enum LatencyValue {
    Latency0,
    Latency1,
    Latency2,
    Latency3,
    Latency4,
    Latency5,
    Latency6,
    Latency7,
    // HELP: Some STM32F4xx allow only 3 bit values for the flash latency, while others allow for 4
    // bit values
    //Latency8,
    //Latency9,
    //Latency10,
    //Latency11,
    //Latency12,
    //Latency13,
    //Latency14,
    //Latency15,
}

impl TryFrom<usize> for LatencyValue {
    type Error = &'static str;

    fn try_from(item: usize) -> Result<Self, Self::Error> {
        match item {
            0 => Ok(LatencyValue::Latency0),
            1 => Ok(LatencyValue::Latency1),
            2 => Ok(LatencyValue::Latency2),
            3 => Ok(LatencyValue::Latency3),
            4 => Ok(LatencyValue::Latency4),
            5 => Ok(LatencyValue::Latency5),
            6 => Ok(LatencyValue::Latency6),
            7 => Ok(LatencyValue::Latency7),
            _ => Err("Error value for LatencyValue::try_from"),
        }
    }
}

impl Flash {
    pub fn new() -> Self {
        Self {
            registers: FLASH_BASE,
        }
    }

    pub fn get_latency(&self) -> LatencyValue {
        // Can't fail because the hardware will always contain a valid value
        TryFrom::try_from(self.registers.acr.read(ACR::LATENCY) as usize).unwrap()
    }

    pub fn set_latency(&self, value: LatencyValue) {
        self.registers.acr.modify(ACR::LATENCY.val(value as u32));
    }
}
