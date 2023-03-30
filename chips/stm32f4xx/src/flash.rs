use kernel::utilities::StaticRef;
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite, WriteOnly};
use kernel::utilities::registers::interfaces::{Readable, ReadWriteable};
use kernel::ErrorCode;
use kernel::debug;

// TODO: Make sure it is possible to create one common superset flash structure
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

// TODO: Make sure it is possible to create one common superset flash bitfields
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

// All this hassle is caused by the fact that the following 4 chip models support 3 bit latency
// values, while the other chips support 4 bit values
#[cfg(not(any(
    feature = "stm32f405",
    feature = "stm32f415",
    feature = "stm32f407",
    feature = "stm32f417"
)))]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum FlashLatency {
    Latency0,
    Latency1,
    Latency2,
    Latency3,
    Latency4,
    Latency5,
    Latency6,
    Latency7,
    Latency8,
    Latency9,
    Latency10,
    Latency11,
    Latency12,
    Latency13,
    Latency14,
    Latency15,
}

#[cfg(any(
    feature = "stm32f405",
    feature = "stm32f415",
    feature = "stm32f407",
    feature = "stm32f417"
))]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum FlashLatency {
    Latency0,
    Latency1,
    Latency2,
    Latency3,
    Latency4,
    Latency5,
    Latency6,
    Latency7,
}

impl TryFrom<usize> for FlashLatency {
    type Error = &'static str;

    #[cfg(not(any(
        feature = "stm32f405",
        feature = "stm32f415",
        feature = "stm32f407",
        feature = "stm32f417"
    )))]
    fn try_from(item: usize) -> Result<Self, Self::Error> {
        match item {
            0 => Ok(FlashLatency::Latency0),
            1 => Ok(FlashLatency::Latency1),
            2 => Ok(FlashLatency::Latency2),
            3 => Ok(FlashLatency::Latency3),
            4 => Ok(FlashLatency::Latency4),
            5 => Ok(FlashLatency::Latency5),
            6 => Ok(FlashLatency::Latency6),
            7 => Ok(FlashLatency::Latency7),
            8 => Ok(FlashLatency::Latency8),
            9 => Ok(FlashLatency::Latency9),
            10 => Ok(FlashLatency::Latency10),
            11 => Ok(FlashLatency::Latency11),
            12 => Ok(FlashLatency::Latency12),
            13 => Ok(FlashLatency::Latency13),
            14 => Ok(FlashLatency::Latency14),
            15 => Ok(FlashLatency::Latency15),
            _ => Err("Error value for FlashLatency::try_from"),
        }
    }

    #[cfg(any(
        feature = "stm32f405",
        feature = "stm32f415",
        feature = "stm32f407",
        feature = "stm32f417"
    ))]
    fn try_from(item: usize) -> Result<Self, Self::Error> {
        match item {
            0 => Ok(FlashLatency::Latency0),
            1 => Ok(FlashLatency::Latency1),
            2 => Ok(FlashLatency::Latency2),
            3 => Ok(FlashLatency::Latency3),
            4 => Ok(FlashLatency::Latency4),
            5 => Ok(FlashLatency::Latency5),
            6 => Ok(FlashLatency::Latency6),
            7 => Ok(FlashLatency::Latency7),
            _ => Err("Error value for FlashLatency::try_from"),
        }
    }
}

impl Flash {
    pub fn new() -> Self {
        Self {
            registers: FLASH_BASE,
        }
    }

    // TODO: Take into account the power supply
    #[cfg(not(any(
        feature = "stm32f410",
        feature = "stm32f411",
        feature = "stm32f412",
        feature = "stm32f413",
        feature = "stm32f423"
    )))]
    fn get_number_wait_cycles_based_on_frequency(&self, frequency_mhz: usize) -> FlashLatency {
        if frequency_mhz <= 30 {
            FlashLatency::Latency0
        } else if frequency_mhz <= 60 {
            FlashLatency::Latency1
        } else if frequency_mhz <= 90 {
            FlashLatency::Latency2
        } else if frequency_mhz <= 120 {
            FlashLatency::Latency3
        } else if frequency_mhz <= 150 {
            FlashLatency::Latency4
        } else {
            FlashLatency::Latency5
        }
    }

    #[cfg(any(
        feature = "stm32f410",
        feature = "stm32f411",
        feature = "stm32f412",
    ))]
    fn get_number_wait_cycles_based_on_frequency(&self, frequency_mhz: usize) -> FlashLatency {
        if frequency_mhz <= 30 {
            FlashLatency::Latency0
        } else if frequency_mhz <= 64 {
            FlashLatency::Latency1
        } else if frequency_mhz <= 90 {
            FlashLatency::Latency2
        } else {
            FlashLatency::Latency3
        }
    }

    #[cfg(any(
        feature = "stm32f413",
        feature = "stm32f423",
    ))]
    fn get_number_wait_cycles_based_on_frequency(&self, frequency_mhz: usize) -> FlashLatency {
        if frequency_mhz <= 25 {
            FlashLatency::Latency0
        } else if frequency_mhz <= 50 {
            FlashLatency::Latency1
        } else if frequency_mhz <= 75 {
            FlashLatency::Latency2
        } else {
            FlashLatency::Latency3
        }
    }

    pub fn get_latency(&self) -> FlashLatency {
        // Can't panic because the hardware will always contain a valid value
        TryFrom::try_from(self.registers.acr.read(ACR::LATENCY) as usize).unwrap()
    }

    // TODO: Take into the account the power supply
    // This method has been made public(crate) because flash latency depends on the system
    // clock frequency.
    pub(crate) fn set_latency(&self, sys_clock_frequency: usize) -> Result<(), ErrorCode> {
        let flash_latency = self.get_number_wait_cycles_based_on_frequency(sys_clock_frequency);
        self.registers.acr.modify(ACR::LATENCY.val(flash_latency as u32));

        for _ in 0..16 {
            if self.get_latency() == flash_latency {
                return Ok(());
            }
        }

        Err(ErrorCode::BUSY)
    }
}

pub mod tests {
    use super::*;

    const HSI_FREQUENCY_MHZ: usize = 16;

    #[cfg(not(any(
        feature = "stm32f413",
        feature = "stm32f423"
    )))]
    pub fn test_get_number_wait_cycles_based_on_frequency(flash: &Flash) {
        debug!("");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("Testing number of wait cycles based on the system frequency...");

        // HSI frequency
        assert_eq!(FlashLatency::Latency0, flash.get_number_wait_cycles_based_on_frequency(HSI_FREQUENCY_MHZ));

        // AHB Ethernet minimal frequency
        assert_eq!(FlashLatency::Latency0, flash.get_number_wait_cycles_based_on_frequency(25));

        // Maximum APB1 frequency depending on the chip
        assert_eq!(FlashLatency::Latency1, flash.get_number_wait_cycles_based_on_frequency(42));
        assert_eq!(FlashLatency::Latency1, flash.get_number_wait_cycles_based_on_frequency(45));
        assert_eq!(FlashLatency::Latency1, flash.get_number_wait_cycles_based_on_frequency(50));

        // Maximum APB2 frequency depending on the chip
        assert_eq!(FlashLatency::Latency2, flash.get_number_wait_cycles_based_on_frequency(84));

        // STM32F401 maximum clock frequency is 84MHz
        #[cfg(not(
            feature = "stm32f401"
        ))]
        {
            // Maximum APB2 frequency depending on the chip
            assert_eq!(FlashLatency::Latency2, flash.get_number_wait_cycles_based_on_frequency(90));
            assert_eq!(FlashLatency::Latency3, flash.get_number_wait_cycles_based_on_frequency(100));

            // Default PLL frequency
            assert_eq!(FlashLatency::Latency3, flash.get_number_wait_cycles_based_on_frequency(96));

            // Maximum CPU frequency without overdrive
            assert_eq!(FlashLatency::Latency5, flash.get_number_wait_cycles_based_on_frequency(168));

            // Maximum CPU frequency with overdrive (for some chips)
            assert_eq!(FlashLatency::Latency5, flash.get_number_wait_cycles_based_on_frequency(180));
        }

        debug!("Finished testing number of wait cycles based on the system clock frequency. Everything is alright!");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("");
    }

    #[cfg(any(
        feature = "stm32f413",
        feature = "stm32f423"
    ))]
    pub fn test_get_number_wait_cycles_based_on_frequency(flash: &Flash) {
        debug!("");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("Testing number of wait cycles based on the system frequency...");

        // HSI frequency
        assert_eq!(FlashLatency::Latency0, flash.get_number_wait_cycles_based_on_frequency(HSI_FREQUENCY_MHZ));

        // AHB Ethernet minimal frequency
        assert_eq!(FlashLatency::Latency0, flash.get_number_wait_cycles_based_on_frequency(25));

        // Maximum APB1 frequency depending on the chip
        assert_eq!(FlashLatency::Latency1, flash.get_number_wait_cycles_based_on_frequency(42));
        assert_eq!(FlashLatency::Latency1, flash.get_number_wait_cycles_based_on_frequency(45));
        assert_eq!(FlashLatency::Latency1, flash.get_number_wait_cycles_based_on_frequency(50));

        // Maximum APB2 frequency depending on the chip
        assert_eq!(FlashLatency::Latency3, flash.get_number_wait_cycles_based_on_frequency(84));
        assert_eq!(FlashLatency::Latency3, flash.get_number_wait_cycles_based_on_frequency(90));
        assert_eq!(FlashLatency::Latency3, flash.get_number_wait_cycles_based_on_frequency(100));

        // Default PLL frequency
        assert_eq!(FlashLatency::Latency3, flash.get_number_wait_cycles_based_on_frequency(96));

        debug!("Finished testing number of wait cycles based on the system clock frequency. Everything is alright!");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("");
    }

    pub fn test_set_flash_latency(flash: &Flash) {
        debug!("");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("Testing setting flash latency...");

        // HSI frequency
        assert_eq!(Ok(()), flash.set_latency(HSI_FREQUENCY_MHZ));

        // Minimal Ethernet frequency
        assert_eq!(Ok(()), flash.set_latency(25));

        // Maximum APB1 frequency depending on the chip
        assert_eq!(Ok(()), flash.set_latency(42));
        assert_eq!(Ok(()), flash.set_latency(45));
        assert_eq!(Ok(()), flash.set_latency(50));

        // Maximum APB2 frequency depending on the chip
        assert_eq!(Ok(()), flash.set_latency(84));

        // STM32F401 maximum system clock frequency is 84MHz
        #[cfg(not(feature = "stm32f401"))]
        {
            // Maximum APB2 frequency depending on the chip
            assert_eq!(Ok(()), flash.set_latency(90));
            assert_eq!(Ok(()), flash.set_latency(100));

            // Default PLL frequency
            assert_eq!(Ok(()), flash.set_latency(96));
        }

        // Low entries STM32F4 chips don't support frequencies higher than 100 MHz,
        // but the foundation and advanced ones support system clock frequencies up to
        // 180MHz
        #[cfg(not(any(
            feature = "stm32f401",
            feature = "stm32f410",
            feature = "stm32f411",
            feature = "stm32f412",
            feature = "stm32f413",
            feature = "stm32f423",
        )))]
        {
            // Maximum CPU frequency without overdrive
            assert_eq!(Ok(()), flash.set_latency(168));

            // Maximum CPU frequency with overdrive (for some chips)
            assert_eq!(Ok(()), flash.set_latency(180));

        }

        // Revert to default settings
        assert_eq!(Ok(()), flash.set_latency(HSI_FREQUENCY_MHZ));

        debug!("Finished testing setting flash latency. Everything is alright!");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("");
    }

    pub fn run_all(flash: &Flash) {
        debug!("");
        debug!("===============================================");
        debug!("Testing setting flash latency...");

        test_get_number_wait_cycles_based_on_frequency(flash);
        test_set_flash_latency(flash);

        debug!("Finished testing flash. Everything is alright!");
        debug!("===============================================");
        debug!("");
    }
}
