// Basic RCC definitions for STM32U5 series microcontrollers.

use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};
use kernel::utilities::registers::{register_bitfields, ReadWrite};
use kernel::utilities::StaticRef;

#[repr(C)]
struct RccRegisters {
    /// RCC clock control register
    cr: ReadWrite<u32, CR::Register>,
    _reserved0: [u8; 0x004],
    /// RCC internal clock sources calibration register 1
    icscr1: ReadWrite<u32, ICSCR1::Register>,
    /// RCC internal clock sources calibration register 2
    icscr2: ReadWrite<u32, ICSCR2::Register>,
    /// RCC internal clock sources calibration register 3
    icscr3: ReadWrite<u32, ICSCR3::Register>,
    /// RCC clock recovery RC register
    crrcr: ReadWrite<u32>,
    _reserved1: [u8; 0x004],
    /// RCC clock configuration register 1
    cfgr1: ReadWrite<u32, CFGR1::Register>,
    /// RCC clock configuration register 2
    cfgr2: ReadWrite<u32, CFGR2::Register>,
    /// RCC clock configuration register 3
    cfgr3: ReadWrite<u32, CFGR3::Register>,
    /// RCC PLL1 configuration register
    pll1cfgr: ReadWrite<u32, PLL1CFGR::Register>,
    /// RCC PLL2 configuration register
    pll2cfgr: ReadWrite<u32, PLL2CFGR::Register>,
    /// RCC PLL3 configuration register
    pll3cfgr: ReadWrite<u32, PLL3CFGR::Register>,
    /// RCC PLL1 dividers register
    pll1divr: ReadWrite<u32, PLL1DIVR::Register>,
    /// RCC PLL1 fractional divider register
    pll1fracr: ReadWrite<u32>,
    /// RCC PLL2 dividers configuration register
    pll2divr: ReadWrite<u32, PLL2DIVR::Register>,
    /// RCC PLL2 fractional divider register
    pll2fracr: ReadWrite<u32>,
    /// RCC PLL3 dividers configuration register
    pll3divr: ReadWrite<u32, PLL3DIVR::Register>,
    /// RCC PLL3 fractional divider register
    pll3fracr: ReadWrite<u32>,
    _reserved2: [u8; 0x004],
    /// RCC clock interrupt enable register
    cier: ReadWrite<u32, CIER::Register>,
    /// RCC clock interrupt flag register
    cifr: ReadWrite<u32, CIFR::Register>,
    /// RCC clock interrupt clear register
    cicr: ReadWrite<u32, CICR::Register>,
    _reserved3: [u8; 0x004],
    /// RCC AHB1 peripheral reset register
    ahb1rstr: ReadWrite<u32, AHB1RSTR::Register>,
    /// RCC AHB2 peripheral reset register 1
    ahb2rstr1: ReadWrite<u32, AHB2RSTR1::Register>,
    /// RCC AHB2 peripheral reset register 2
    ahb2rstr2: ReadWrite<u32, AHB2RSTR2::Register>,
    /// RCC AHB3 peripheral reset register
    ahb3rstr: ReadWrite<u32, AHB3RSTR::Register>,
    _reserved4: [u8; 0x004],
    /// RCC APB1 peripheral reset register 1
    apb1rstr1: ReadWrite<u32, APB1RSTR1::Register>,
    /// RCC APB1 peripheral reset register 2
    apb1rstr2: ReadWrite<u32, APB1RSTR2::Register>,
    /// RCC APB2 peripheral reset register
    apb2rstr: ReadWrite<u32, APB2RSTR::Register>,
    /// RCC APB3 peripheral reset register
    apb3rstr: ReadWrite<u32, APB3RSTR::Register>,
    _reserved5: [u8; 0x004],
    /// RCC AHB1 peripheral clock enable register
    ahb1enr: ReadWrite<u32, AHB1ENR::Register>,
    /// RCC AHB2 peripheral clock enable register 1
    ahb2enr1: ReadWrite<u32, AHB2ENR1::Register>,
    /// RCC AHB2 peripheral clock enable register 2
    ahb2enr2: ReadWrite<u32, AHB2ENR2::Register>,
    /// RCC AHB3 peripheral clock enable register
    ahb3enr: ReadWrite<u32, AHB3ENR::Register>,
    _reserved6: [u8; 0x004],
    /// RCC APB1 peripheral clock enable register 1
    apb1enr1: ReadWrite<u32, APB1ENR1::Register>,
    /// RCC APB1 peripheral clock enable register 2
    apb1enr2: ReadWrite<u32, APB1ENR2::Register>,
    /// RCC APB2 peripheral clock enable register
    apb2enr: ReadWrite<u32, APB2ENR::Register>,
    /// RCC APB3 peripheral clock enable register
    apb3enr: ReadWrite<u32, APB3ENR::Register>,
    _reserved7: [u8; 0x004],
    /// RCC AHB1 peripheral clock enable in Sleep and Stop modes register
    ahb1smenr: ReadWrite<u32, AHB1SMENR::Register>,
    /// RCC AHB2 peripheral clock enable in Sleep and	Stop modes register 1
    ahb2smenr1: ReadWrite<u32, AHB2SMENR1::Register>,
    /// RCC AHB2 peripheral clock enable in Sleep and	Stop modes register 2
    ahb2smenr2: ReadWrite<u32, AHB2SMENR2::Register>,
    /// RCC AHB3 peripheral clock enable in Sleep and Stop modes register
    ahb3smenr: ReadWrite<u32, AHB3SMENR::Register>,
    _reserved8: [u8; 0x004],
    /// RCC APB1 peripheral clock enable in Sleep and Stop modes	register 1
    apb1smenr1: ReadWrite<u32, APB1SMENR1::Register>,
    /// RCC APB1 peripheral clocks enable in Sleep and	Stop modes register 2
    apb1smenr2: ReadWrite<u32, APB1SMENR2::Register>,
    /// RCC APB2 peripheral clocks enable in Sleep and Stop modes register
    apb2smenr: ReadWrite<u32, APB2SMENR::Register>,
    /// RCC APB3 peripheral clock enable in Sleep and Stop modes register
    apb3smenr: ReadWrite<u32, APB3SMENR::Register>,
    _reserved9: [u8; 0x0C],
    /// RCC peripherals independent clock configuration register 1
    ccipr1: ReadWrite<u32, CCIPR1::Register>,
}

// Bitfields for STM32U5 RCC_CR / RCC_CFGR1 / RCC_CFGR2.
//
// Check against the STM32U5 reference manual if you change them.
register_bitfields![u32,
    CR [
    /// MSIS clock enable
/// This bit is set and cleared by software. It is cleared by hardware to stop the MSIS oscillator when entering Stop, Standby or Shutdown mode. This bit is set by hardware to force the�MSIS oscillator on when exiting Standby or Shutdown mode. It is set by hardware to force the MSIS oscillator ON when STOPWUCK = 0 when exiting Stop modes, or in case of a failure of the HSE oscillator.
/// Set by hardware when used directly or indirectly as system clock.
    MSISON OFFSET(0) NUMBITS(1) [
        /// MSIS (MSI system) oscillator off
        MSISMSISystemOscillatorOff = 0,
        /// MSIS (MSI system) oscillator on
        MSISMSISystemOscillatorOn = 1
    ],
    /// MSI enable for some peripheral kernels
/// This bit is set and cleared by software to force MSI ON even in Stop modes. Keeping the MSI on in Stop mode allows the communication speed not to be reduced by the MSI startup time. This bit has no effect on MSISON and MSIKON values (see Section�11.4.24 for more details). This bit must be configured at 0 before entering Stop 3 mode.
    MSIKERON OFFSET(1) NUMBITS(1) [
        /// No effect on MSI oscillator
        NoEffectOnMSIOscillator = 0,
        /// MSI oscillator forced ON even in Stop mode
        MSIOscillatorForcedONEvenInStopMode = 1
    ],
    /// MSIS clock ready flag
/// This bit is set by hardware to indicate that the MSIS oscillator is stable. It is set only when MSIS is enabled by software (by setting MSISON).
/// Note: Once the MSISON bit is cleared, MSISRDY goes low after six MSIS clock cycles.
    MSISRDY OFFSET(2) NUMBITS(1) [
        /// MSIS (MSI system) oscillator not ready
        MSISMSISystemOscillatorNotReady = 0,
        /// MSIS (MSI system) oscillator ready
        MSISMSISystemOscillatorReady = 1
    ],
    /// MSI clock PLL-mode enable
/// This bit is set and cleared by software to enable/disable the PLL part of the MSI clock source.
/// MSIPLLEN must be enabled after LSE is enabled (LSEON enabled) and ready (LSERDY set by hardware). A hardware protection prevents from enabling MSIPLLEN if LSE is not ready. This bit is cleared by hardware when LSE is disabled (LSEON = 0) or when the CSS on LSE detects a LSE failure (see RCC_CSR).
    MSIPLLEN OFFSET(3) NUMBITS(1) [
        /// MSI PLL-mode OFF
        MSIPLLModeOFF = 0,
        /// MSI PLL-mode ON
        MSIPLLModeON = 1
    ],
    /// MSIK clock enable
/// This bit is set and cleared by software. It is cleared by hardware to stop the MSIK when entering Stop, Standby, or Shutdown mode. This bit is set by hardware to force the MSIK oscillator ON when exiting Standby or Shutdown mode. It is set by hardware to force the MSIK oscillator on when STOPWUCK = 0 or STOPKERWUCK�=�0 when exiting Stop modes, or in case of a failure of the HSE oscillator.
    MSIKON OFFSET(4) NUMBITS(1) [
        /// MSIK (MSI kernel) oscillator disabled
        MSIKMSIKernelOscillatorDisabled = 0,
        /// MSIK (MSI kernel) oscillator enabled
        MSIKMSIKernelOscillatorEnabled = 1
    ],
    /// MSIK clock ready flag
/// This bit is set by hardware to indicate that the MSIK is stable. It is set only when MSI kernel oscillator is enabled by software by setting MSIKON.
/// Note: Once MSIKON bit is cleared, MSIKRDY goes low after six MSIK oscillator clock cycles.
    MSIKRDY OFFSET(5) NUMBITS(1) [
        /// MSIK (MSI kernel) oscillator not ready
        MSIKMSIKernelOscillatorNotReady = 0,
        /// MSIK (MSI kernel) oscillator ready
        MSIKMSIKernelOscillatorReady = 1
    ],
    /// MSI clock with PLL mode selection
/// This bit is set and cleared by software to select which MSI output clock uses the PLL mode. It�can be written only when the MSI PLL mode is disabled (MSIPLLEN = 0).
/// Note: If the MSI kernel clock output uses the same oscillator source than the MSI system clock output, then the PLL mode is applied to both clock outputs.
    MSIPLLSEL OFFSET(6) NUMBITS(1) [
        /// PLL mode applied to MSIK (MSI kernel) clock output
        PLLModeAppliedToMSIKMSIKernelClockOutput = 0,
        /// PLL mode applied to MSIS (MSI system) clock output
        PLLModeAppliedToMSISMSISystemClockOutput = 1
    ],
    /// MSI PLL mode fast startup
/// This bit is set and reset by software to enable/disable the fast PLL mode start-up of the MSI clock source. This bit is used only if PLL mode is selected (MSIPLLEN = 1).
/// The fast start-up feature is not active the first time the PLL mode is selected. The�fast start-up is active when the MSI in PLL mode returns from switch off.
    MSIPLLFAST OFFSET(7) NUMBITS(1) [
        /// MSI PLL normal start-up
        MSIPLLNormalStartUp = 0,
        /// MSI PLL fast start-up
        MSIPLLFastStartUp = 1
    ],
    /// HSI16 clock enable
/// This bit is set and cleared by software. It is cleared by hardware to stop the HSI16 oscillator when entering Stop, Standby, or Shutdown mode. This bit is set by hardware to force the�HSI16 oscillator on when STOPWUCK = 1 when leaving Stop modes, or in case of failure of the HSE crystal oscillator. This bit is set by hardware if the HSI16 is used directly or indirectly as system clock.
    HSION OFFSET(8) NUMBITS(1) [
        /// HSI16 oscillator off
        HSI16OscillatorOff = 0,
        /// HSI16 oscillator on
        HSI16OscillatorOn = 1
    ],
    /// HSI16 enable for some peripheral kernels
/// This bit is set and cleared by software to force HSI16 ON even in Stop modes. Keeping HSI16 on in Stop mode allows the communication speed not to be reduced by the HSI16 startup time. This bit has no effect on HSION value. Refer to Section�11.4.24 for more details.
/// This bit must be configured at 0 before entering Stop 3 mode.
    HSIKERON OFFSET(9) NUMBITS(1) [
        /// No effect on HSI16 oscillator
        NoEffectOnHSI16Oscillator = 0,
        /// HSI16 oscillator forced on even in Stop mode
        HSI16OscillatorForcedOnEvenInStopMode = 1
    ],
    /// HSI16 clock ready flag
/// This bit is set by hardware to indicate that HSI16 oscillator is stable. It is set only when HSI16 is enabled by software (by setting HSION).
/// Note: Once the HSION bit is cleared, HSIRDY goes low after six HSI16 clock cycles.
    HSIRDY OFFSET(10) NUMBITS(1) [
        /// HSI16 oscillator not ready
        HSI16OscillatorNotReady = 0,
        /// HSI16 oscillator ready
        HSI16OscillatorReady = 1
    ],
    /// HSI48 clock enable
/// This bit is set and cleared by software. It is cleared by hardware to stop the HSI48 when entering in Stop, Standby, or Shutdown modes.
    HSI48ON OFFSET(12) NUMBITS(1) [
        /// HSI48 oscillator off
        HSI48OscillatorOff = 0,
        /// HSI48 oscillator on
        HSI48OscillatorOn = 1
    ],
    /// HSI48 clock ready flag
/// This bit is set by hardware to indicate that HSI48 oscillator is stable. Itis set only when HSI48 is enabled by software (by setting HSI48ON).
    HSI48RDY OFFSET(13) NUMBITS(1) [
        /// HSI48 oscillator not ready
        HSI48OscillatorNotReady = 0,
        /// HSI48 oscillator ready
        HSI48OscillatorReady = 1
    ],
    /// SHSI clock enable
/// This bit is set and cleared by software. It is cleared by hardware to stop the SHSI when entering in Stop, Standby, or Shutdown modes.
    SHSION OFFSET(14) NUMBITS(1) [
        /// SHSI oscillator off
        SHSIOscillatorOff = 0,
        /// SHSI oscillator on
        SHSIOscillatorOn = 1
    ],
    /// SHSI clock ready flag
/// This bit is set by hardware to indicate that the SHSI oscillator is stable. It is set only when SHSI is enabled by software (by setting SHSION).
/// Note: Once the SHSION bit is cleared, SHSIRDY goes low after six SHSI clock cycles.
    SHSIRDY OFFSET(15) NUMBITS(1) [
        /// SHSI oscillator not ready
        SHSIOscillatorNotReady = 0,
        /// SHSI oscillator ready
        SHSIOscillatorReady = 1
    ],
    /// HSE clock enable
/// This bit is set and cleared by software. It is cleared by hardware to stop the HSE oscillator when entering Stop, Standby, or Shutdown mode. This bit cannot be reset if the HSE oscillator is used directly or indirectly as the system clock.
    HSEON OFFSET(16) NUMBITS(1) [
        /// HSE oscillator off
        HSEOscillatorOff = 0,
        /// HSE oscillator on
        HSEOscillatorOn = 1
    ],
    /// HSE clock ready flag
/// This bit is set by hardware to indicate that the HSE oscillator is stable.
/// Note: Once the HSEON bit is cleared, HSERDY goes low after six HSE clock cycles.
    HSERDY OFFSET(17) NUMBITS(1) [
        /// HSE oscillator not ready
        HSEOscillatorNotReady = 0,
        /// HSE oscillator ready
        HSEOscillatorReady = 1
    ],
    /// HSE crystal oscillator bypass
/// This bit is set and cleared by software to bypass the oscillator with an external clock. The�external clock must be enabled with the HSEON bit set, to be used by the device. This�bit can be written only if the HSE oscillator is disabled.
    HSEBYP OFFSET(18) NUMBITS(1) [
        /// HSE crystal oscillator not bypassed
        HSECrystalOscillatorNotBypassed = 0,
        /// HSE crystal oscillator bypassed with external clock
        HSECrystalOscillatorBypassedWithExternalClock = 1
    ],
    /// Clock security system enable
/// This bit is set by software to enable the clock security system. When CSSON is set, the clock detector is enabled by hardware when the HSE oscillator is ready, and disabled by hardware if a HSE clock failure is detected. This bit is set only and is cleared by reset.
    CSSON OFFSET(19) NUMBITS(1) [
        /// clock security system OFF (clock detector OFF)
        ClockSecuritySystemOFFClockDetectorOFF = 0,
        /// clock security system ON (clock detector ON if the HSE oscillator is stable, OFF if not).
        ClockSecuritySystemONClockDetectorONIfTheHSEOscillatorIsStableOFFIfNot = 1
    ],
    /// HSE external clock bypass mode
/// This bit is set and reset by software to select the external clock mode in bypass mode. External clock mode must be configured with HSEON bit to be used by the device. This bit can be written only if the HSE oscillator is disabled. This bit is active only if the HSE bypass mode is enabled.
    HSEEXT OFFSET(20) NUMBITS(1) [
        /// external HSE clock analog mode
        ExternalHSEClockAnalogMode = 0,
        /// external HSE clock digital mode (through I/O Schmitt trigger)
        ExternalHSEClockDigitalModeThroughIOSchmittTrigger = 1
    ],
    /// PLL1 enable
/// This bit is set and cleared by software to enable the main PLL. It is cleared by hardware when entering Stop, Standby, or Shutdown mode. This bit cannot be reset if the PLL1 clock is used as the system clock.
    PLL1ON OFFSET(24) NUMBITS(1) [
        /// PLL1 OFF
        PLL1OFF = 0,
        /// PLL1 ON
        PLL1ON = 1
    ],
    /// PLL1 clock ready flag
/// This bit is set by hardware to indicate that the PLL1 is locked.
    PLL1RDY OFFSET(25) NUMBITS(1) [
        /// PLL1 unlocked
        PLL1Unlocked = 0,
        /// PLL1 locked
        PLL1Locked = 1
    ],
    /// PLL2 enable
/// This bit is set and cleared by software to enable PLL2. It is cleared by hardware when entering Stop, Standby, or Shutdown mode.
    PLL2ON OFFSET(26) NUMBITS(1) [
        /// PLL2 OFF
        PLL2OFF = 0,
        /// PLL2 ON
        PLL2ON = 1
    ],
    /// PLL2 clock ready flag
/// This bit is set by hardware to indicate that the PLL2 is locked.
    PLL2RDY OFFSET(27) NUMBITS(1) [
        /// PLL2 unlocked
        PLL2Unlocked = 0,
        /// PLL2 locked
        PLL2Locked = 1
    ],
    /// PLL3 enable
/// This bit is set and cleared by software to enable PLL3. It is cleared by hardware when entering Stop, Standby, or Shutdown mode.
    PLL3ON OFFSET(28) NUMBITS(1) [
        /// PLL3 OFF
        PLL3OFF = 0,
        /// PLL3 ON
        PLL3ON = 1
    ],
    /// PLL3 clock ready flag
/// This bit is set by hardware to indicate that the PLL3 is locked.
    PLL3RDY OFFSET(29) NUMBITS(1) [
        /// PLL3 unlocked
        PLL3Unlocked = 0,
        /// PLL3 locked
        PLL3Locked = 1
    ]
],
ICSCR1 [
    /// MSIRC3 clock calibration for MSI ranges 12 to 15
/// These bits are initialized at startup with the factory-programmed MSIRC3 calibration trim value for ranges 12 to 15. When MSITRIM3 is written, MSICAL3 is updated with the sum of MSITRIM3[4:0] and the factory calibration trim value MSIRC2[4:0].
/// There is no hardware protection to limit a potential overflow due to the addition of MSITRIM bitfield and factory program bitfield for this calibration value. Control must be managed by software at user level.
    MSICAL3 OFFSET(0) NUMBITS(5) [],
    /// MSIRC2 clock calibration for MSI ranges 8 to 11
/// These bits are initialized at startup with the factory-programmed MSIRC2 calibration trim value for ranges 8 to 11. When MSITRIM2 is written, MSICAL2 is updated with the sum of MSITRIM2[4:0] and the factory calibration trim value MSIRC2[4:0].
/// There is no hardware protection to limit a potential overflow due to the addition of MSITRIM bitfield and factory program bitfield for this calibration value. Control must be managed by software at user level.
    MSICAL2 OFFSET(5) NUMBITS(5) [],
    /// MSIRC1 clock calibration for MSI ranges 4 to 7
/// These bits are initialized at startup with the factory-programmed MSIRC1 calibration trim value for ranges 4 to 7. When MSITRIM1 is written, MSICAL1 is updated with the sum of MSITRIM1[4:0] and the factory calibration trim value MSIRC1[4:0].
/// There is no hardware protection to limit a potential overflow due to the addition of MSITRIM bitfield and factory program bitfield for this calibration value. Control must be managed by software at user level.
    MSICAL1 OFFSET(10) NUMBITS(5) [],
    /// MSIRC0 clock calibration for MSI ranges 0 to 3
/// These bits are initialized at startup with the factory-programmed MSIRC0 calibration trim value for ranges 0 to 3. When MSITRIM0 is written, MSICAL0 is updated with the sum of MSITRIM0[4:0] and the factory-programmed calibration trim value MSIRC0[4:0].
/// There is no hardware protection to limit a potential overflow due to the addition of MSITRIM bitfield and factory program bitfield for this calibration value. Control must be managed by software at user level.
    MSICAL0 OFFSET(15) NUMBITS(5) [],
    /// MSI bias mode selection
/// This bit is set by software to select the MSI bias mode. By default, the MSI bias is in�continuous mode in order to maintain the output clocks accuracy. Setting this bit reduces the MSI consumption when the regulator is in range 4, or when the device is in Stop 1 or Stop�2 mode, but it�decreases the MSI accuracy
    MSIBIAS OFFSET(22) NUMBITS(1) [
        /// MSI bias continuous mode (clock accuracy fast settling time)
        MSIBiasContinuousModeClockAccuracyFastSettlingTime = 0,
        /// MSI bias sampling mode when the regulator is in range 4, or when the device is in�Stop�1�or Stop 2 (ultra-low-power mode)
        B_0x1 = 1
    ],
    /// MSI clock range selection
/// This bit is set by software to select the MSIS and MSIK clocks range with MSISRANGE[3:0] and MSIKRANGE[3:0]. Write 0 has no effect.
/// After exiting Standby or Shutdown mode, or after a reset, this bit is at 0 and the MSIS and MSIK ranges are provided by MSISSRANGE[3:0] and MSIKSRANGE[3:0] in RCC_CSR.
    MSIRGSEL OFFSET(23) NUMBITS(1) [
        /// MSIS/MSIK ranges provided by MSISSRANGE[3:0] and MSIKSRANGE[3:0] in RCC_CSR
        MSISMSIKRangesProvidedByMSISSRANGE30AndMSIKSRANGE30InRCC_CSR = 0,
        /// MSIS/MSIK ranges provided by MSISRANGE[3:0] and MSIKRANGE[3:0] in�RCC_ICSCR1
        MSISMSIKRangesProvidedByMSISRANGE30AndMSIKRANGE30InRCC_ICSCR1 = 1
    ],
    /// MSIK clock ranges
/// These bits are configured by software to choose the frequency range of MSIK oscillator when MSIRGSEL is set. 16 frequency ranges are available:
/// Note: MSIKRANGE can be modified when MSIK is off (MSISON = 0) or when MSIK is ready (MSIKRDY�=�1). MSIKRANGE must NOT be modified when MSIK is on and NOT ready (MSIKON = 1 and MSIKRDY = 0)
/// Note: MSIKRANGE is kept when the device wakes up from Stop mode, except when the�MSIK range is above 24 MHz. In this case MSIKRANGE is changed by hardware into�range 2 (24 MHz).
    MSIKRANGE OFFSET(24) NUMBITS(4) [
        /// range 0 around 48�MHz
        Range0Around48MHz = 0,
        /// range 1 around 24�MHz
        Range1Around24MHz = 1,
        /// range 2 around 16�MHz
        Range2Around16MHz = 2,
        /// range 3 around 12�MHz
        Range3Around12MHz = 3,
        /// range 4 around 4�MHz (reset value)
        Range4Around4MHzResetValue = 4,
        /// range 5 around 2�MHz
        Range5Around2MHz = 5,
        /// range 6 around 1.33�MHz
        Range6Around133MHz = 6,
        /// range 7 around 1�MHz
        Range7Around1MHz = 7,
        /// range 8 around 3.072�MHz
        Range8Around3072MHz = 8,
        /// range 9 around 1.536�MHz
        Range9Around1536MHz = 9,
        /// range 10 around 1.024�MHz
        Range10Around1024MHz = 10,
        /// range 11 around 768�kHz
        Range11Around768KHz = 11,
        /// range 12 around 400�kHz
        Range12Around400KHz = 12,
        /// range 13 around 200�kHz
        Range13Around200KHz = 13,
        /// range 14 around 133 kHz
        Range14Around133KHz = 14,
        /// range 15 around 100�kHz
        Range15Around100KHz = 15
    ],
    /// MSIS clock ranges
/// These bits are configured by software to choose the frequency range of MSIS oscillator when MSIRGSEL is set. 16 frequency ranges are available:
/// Note: MSISRANGE can be modified when MSIS is off (MSISON = 0) or when MSIS is ready (MSISRDY�=�1). MSISRANGE must NOT be modified when MSIS is on and NOT ready (MSISON�=�1 and MSISRDY�=�0)
/// Note: MSISRANGE is kept when the device wakes up from Stop mode, except when the�MSIS range is above 24 MHz. In this case MSISRANGE is changed by hardware into range 2 (24 MHz).
    MSISRANGE OFFSET(28) NUMBITS(4) [
        /// range 0 around 48�MHz
        Range0Around48MHz = 0,
        /// range 1 around 24�MHz
        Range1Around24MHz = 1,
        /// range 2 around 16�MHz
        Range2Around16MHz = 2,
        /// range 3 around 12�MHz
        Range3Around12MHz = 3,
        /// range 4 around 4�MHz (reset value)
        Range4Around4MHzResetValue = 4,
        /// range 5 around 2�MHz
        Range5Around2MHz = 5,
        /// range 6 around 1.33�MHz
        Range6Around133MHz = 6,
        /// range 7 around 1�MHz
        Range7Around1MHz = 7,
        /// range 8 around 3.072�MHz
        Range8Around3072MHz = 8,
        /// range 9 around 1.536�MHz
        Range9Around1536MHz = 9,
        /// range 10 around 1.024�MHz
        Range10Around1024MHz = 10,
        /// range 11 around 768�kHz
        Range11Around768KHz = 11,
        /// range 12 around 400�kHz
        Range12Around400KHz = 12,
        /// range 13 around 200�kHz
        Range13Around200KHz = 13,
        /// range 14 around 133 kHz
        Range14Around133KHz = 14,
        /// range 15 around 100�kHz
        Range15Around100KHz = 15
    ]
],
ICSCR2 [
    /// MSI clock trimming for ranges 12 to 15
/// These bits provide an additional user-programmable trimming value that is added to the factory-programmed calibration trim value MSIRC3[4:0] bits. It can be programmed to adjust to voltage and temperature variations that influence the frequency of the MSI.
    MSITRIM3 OFFSET(0) NUMBITS(5) [],
    /// MSI clock trimming for ranges 8 to 11
/// These bits provide an additional user-programmable trimming value that is added to the factory-programmed calibration trim value MSIRC2[4:0] bits. It can be programmed to adjust to voltage and temperature variations that influence the frequency of the MSI.
    MSITRIM2 OFFSET(5) NUMBITS(5) [],
    /// MSI clock trimming for ranges 4 to 7
/// These bits provide an additional user-programmable trimming value that is added to the factory-programmed calibration trim value MSIRC1[4:0] bits. It can be programmed to adjust to voltage and temperature variations that influence the frequency of the MSI.
    MSITRIM1 OFFSET(10) NUMBITS(5) [],
    /// MSI clock trimming for ranges 0 to 3
/// These bits provide an additional user-programmable trimming value that is added to the factory-programmed calibration trim value MSIRC0[4:0] bits. It can be programmed to adjust to voltage and temperature variations that influence the frequency of the MSI.
    MSITRIM0 OFFSET(15) NUMBITS(5) []
],
ICSCR3 [
    /// HSI clock calibration
/// These bits are initialized at startup with the factory-programmed HSI calibration trim value. When HSITRIM is written, HSICAL is updated with the sum of HSITRIM and the factory trim value.
    HSICAL OFFSET(0) NUMBITS(12) [],
    /// HSI clock trimming
/// These bits provide an additional user-programmable trimming value that is added to HSICAL[11:0] bits. It can be programmed to adjust to voltage and temperature variations that influence the frequency of the HSI.
    HSITRIM OFFSET(16) NUMBITS(5) []
],
    CFGR1 [
    /// system clock switch
/// This bitfield is set and cleared by software to select system clock source (SYSCLK). It is configured by hardware to force MSIS oscillator selection when exiting Standby or Shutdown mode. This bitfield is configured by hardware to force MSIS or HSI16 oscillator selection when exiting Stop mode or in case of HSE oscillator failure, depending on STOPWUCK.
    SW OFFSET(0) NUMBITS(2) [
        /// MSIS selected as system clock
        MSISSelectedAsSystemClock = 0,
        /// HSI16 selected as system clock
        HSI16SelectedAsSystemClock = 1,
        /// HSE selected as system clock
        HSESelectedAsSystemClock = 2,
        /// PLL pll1_r_ck selected as system clock
        PLLPll1_r_ckSelectedAsSystemClock = 3
    ],
    /// system clock switch status
/// This bitfield is set and cleared by hardware to indicate which clock source is used as system clock.
    SWS OFFSET(2) NUMBITS(2) [
        /// MSIS oscillator used as system clock
        MSISOscillatorUsedAsSystemClock = 0,
        /// HSI16 oscillator used as system clock
        HSI16OscillatorUsedAsSystemClock = 1,
        /// HSE used as system clock
        HSEUsedAsSystemClock = 2,
        /// PLL pll1_r_ck used as system clock
        PLLPll1_r_ckUsedAsSystemClock = 3
    ],
    /// wake-up from Stop and CSS backup clock selection
/// This bit is set and cleared by software to select the system clock used when exiting Stop mode. The selected clock is also used as emergency clock for the clock security system on�HSE.
/// STOPWUCK must not be modified when the CSS is enabled by HSECSSON in�RCC_CR, and the system clock is HSE (SWS = 10) or a switch on HSE is�requested (SW�=�10).
    STOPWUCK OFFSET(4) NUMBITS(1) [
        /// MSIS oscillator selected as wake-up from stop clock and CSS backup clock
        MSISOscillatorSelectedAsWakeUpFromStopClockAndCSSBackupClock = 0,
        /// HSI16 oscillator selected as wake-up from stop clock and CSS backup clock
        HSI16OscillatorSelectedAsWakeUpFromStopClockAndCSSBackupClock = 1
    ],
    /// wake-up from Stop kernel clock automatic enable selection
/// This bit is set and cleared by software to enable automatically another oscillator when exiting Stop mode. This oscillator can be used as independent kernel clock by peripherals.
    STOPKERWUCK OFFSET(5) NUMBITS(1) [
        /// MSIK oscillator automatically enabled when exiting Stop mode
        MSIKOscillatorAutomaticallyEnabledWhenExitingStopMode = 0,
        /// HSI16 oscillator automatically enabled when exiting Stop mode
        HSI16OscillatorAutomaticallyEnabledWhenExitingStopMode = 1
    ],
    /// microcontroller clock output
/// This bitfield is set and cleared by software.
/// Others: reserved
/// Note: This clock output may have some truncated cycles at startup or during MCO clock source switching.
    MCOSEL OFFSET(24) NUMBITS(4) [
        /// MCO output disabled, no clock on MCO
        MCOOutputDisabledNoClockOnMCO = 0,
        /// SYSCLK system clock selected
        SYSCLKSystemClockSelected = 1,
        /// MSIS clock selected
        MSISClockSelected = 2,
        /// HSI16 clock selected
        HSI16ClockSelected = 3,
        /// HSE clock selected
        HSEClockSelected = 4,
        /// Main PLL clock pll1_r_ck selected
        MainPLLClockPll1_r_ckSelected = 5,
        /// LSI clock selected
        LSIClockSelected = 6,
        /// LSE clock selected
        LSEClockSelected = 7,
        /// Internal HSI48 clock selected
        InternalHSI48ClockSelected = 8,
        /// MSIK clock selected
        MSIKClockSelected = 9
    ],
    /// microcontroller clock output prescaler
/// This bitfield is set and cleared by software. It is highly recommended to change this prescaler before MCO output is enabled.
/// Others: not allowed
    MCOPRE OFFSET(28) NUMBITS(3) [
        /// MCO divided by 1
        MCODividedBy1 = 0,
        /// MCO divided by 2
        MCODividedBy2 = 1,
        /// MCO divided by 4
        MCODividedBy4 = 2,
        /// MCO divided by 8
        MCODividedBy8 = 3,
        /// MCO divided by 16
        MCODividedBy16 = 4
    ]
],

    CFGR2 [
    /// AHB prescaler
/// This bitfiled is set and cleared by software to control the division factor of the AHB clock (HCLK).
/// Depending on the device voltage range, the software must set these bits correctly to ensure that the system frequency does not exceed the maximum allowed frequency (for more details, refer to Table�118). After a write operation to these bits and before decreasing the voltage range, this register must be read to be sure that the new value is taken into account.
/// 0xxx: SYSCLK not divided
    HPRE OFFSET(0) NUMBITS(4) [
        /// SYSCLK divided by 2
        SYSCLKDividedBy2 = 8,
        /// SYSCLK divided by 4
        SYSCLKDividedBy4 = 9,
        /// SYSCLK divided by 8
        SYSCLKDividedBy8 = 10,
        /// SYSCLK divided by 16
        SYSCLKDividedBy16 = 11,
        /// SYSCLK divided by 64
        SYSCLKDividedBy64 = 12,
        /// SYSCLK divided by 128
        SYSCLKDividedBy128 = 13,
        /// SYSCLK divided by 256
        SYSCLKDividedBy256 = 14,
        /// SYSCLK divided by 512
        SYSCLKDividedBy512 = 15
    ],
    /// APB1 prescaler
/// This bitfiled is set and cleared by software to control the division factor of APB1 clock (PCLK1).
/// 0xx: PCLK1 not divided
    PPRE1 OFFSET(4) NUMBITS(3) [
        /// PCLK1 divided by 2
        PCLK1DividedBy2 = 4,
        /// PCLK1 divided by 4
        PCLK1DividedBy4 = 5,
        /// PCLK1 divided by 8
        PCLK1DividedBy8 = 6,
        /// PCLK1 divided by 16
        PCLK1DividedBy16 = 7
    ],
    /// APB2 prescaler
/// This bitfiled is set and cleared by software to control the division factor of APB2 clock (PCLK2).
/// 0xx: PCLK2 not divided
    PPRE2 OFFSET(8) NUMBITS(3) [
        /// PCLK2 divided by 2
        PCLK2DividedBy2 = 4,
        /// PCLK2 divided by 4
        PCLK2DividedBy4 = 5,
        /// PCLK2 divided by 8
        PCLK2DividedBy8 = 6,
        /// PCLK2 divided by 16
        PCLK2DividedBy16 = 7
    ],
    /// DSI PHY prescaler
/// This bitfiled is set and cleared by software to control the division factor of DSI PHY bus clock (DCLK).
/// 0xx: DCLK not divided
/// Note: This bitfield is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bitfield as reserved and keep it at reset value.
    DPRE OFFSET(12) NUMBITS(3) [
        /// DCLK divided by 2
        DCLKDividedBy2 = 4,
        /// DCLK divided by 4
        DCLKDividedBy4 = 5,
        /// DCLK divided by 8
        DCLKDividedBy8 = 6,
        /// DCLK divided by 16
        DCLKDividedBy16 = 7
    ],
    /// AHB1 clock disable
/// This bit can be set in order to further reduce power consumption, when none of the AHB1 peripherals (except those listed hereafter) are used and when their clocks are disabled in RCC_AHB1ENR. When this bit is set, all the AHB1 peripherals clocks are off, except for FLASH, BKPSRAM, ICACHE, DCACHE1 and SRAM1.
    AHB1DIS OFFSET(16) NUMBITS(1) [
        /// AHB1 clock enabled, distributed to peripherals according to their dedicated clock enable control bits
        B_0x0 = 0,
        /// AHB1 clock disabled
        AHB1ClockDisabled = 1
    ],
    /// AHB2_1 clock disable
/// This bit can be set in order to further reduce power consumption, when none of the AHB2 peripherals from RCC_AHB2ENR1 (except SRAM2 and SRAM3) are used and when their clocks are disabled in RCC_AHB2ENR1. When this bit is set, all the AHB2 peripherals clocks from RCC_AHB2ENR1 are off, except for SRAM2 and SRAM3.
    AHB2DIS1 OFFSET(17) NUMBITS(1) [
        /// AHB2_1 clock enabled, distributed to peripherals according to their dedicated clock enable control bits
        B_0x0 = 0,
        /// AHB2_1 clock disabled
        AHB2_1ClockDisabled = 1
    ],
    /// AHB2_2 clock disable
/// This bit can be set in order to further reduce power consumption, when none of the AHB2 peripherals from RCC_AHB2ENR2 are used and when their clocks are disabled in RCC_AHB2ENR2. When this bit is set, all the AHB2 peripherals clocks from RCC_AHB2ENR2 are off.
    AHB2DIS2 OFFSET(18) NUMBITS(1) [
        /// AHB2_2 clock enabled, distributed to peripherals according to their dedicated clock enable control bits
        B_0x0 = 0,
        /// AHB2_2 clock disabled
        AHB2_2ClockDisabled = 1
    ],
    /// APB1 clock disable
/// This bit can be set in order to further reduce power consumption, when none of the APB1 peripherals (except IWDG) are used and when their clocks are disabled in RCC_APB1ENR. When this bit is set, all the APB1 peripherals clocks are off, except for IWDG.
    APB1DIS OFFSET(19) NUMBITS(1) [
        /// APB1 clock enabled, distributed to peripherals according to their dedicated clock enable control bits
        B_0x0 = 0,
        /// APB1 clock disabled
        APB1ClockDisabled = 1
    ],
    /// APB2 clock disable
/// This bit can be set in order to further reduce power consumption, when none of the APB2 peripherals are used and when their clocks are disabled in RCC_APB2ENR. When this bit is set, all APB2 peripherals clocks are off.
    APB2DIS OFFSET(20) NUMBITS(1) [
        /// APB2 clock enabled, distributed to peripherals according to their dedicated clock enable control bits
        B_0x0 = 0,
        /// APB2 clock disabled
        APB2ClockDisabled = 1
    ]
],
    CFGR3 [
    /// APB3 prescaler
/// This bitfield is set and cleared by software to control the division factor of the APB3 clock (PCLK3).
/// 0xx: HCLK not divided
    PPRE3 OFFSET(4) NUMBITS(3) [
        /// HCLK divided by 2
        HCLKDividedBy2 = 4,
        /// HCLK divided by 4
        HCLKDividedBy4 = 5,
        /// HCLK divided by 8
        HCLKDividedBy8 = 6,
        /// HCLK divided by 16
        HCLKDividedBy16 = 7
    ],
    /// AHB3 clock disable
/// This bit can be set in order to further reduce power consumption, when none of the AHB3 peripherals (except SRAM4) are used and when their clocks are disabled in RCC_AHB3ENR. When this bit is set, all the AHB3 peripherals clocks are off, except for SRAM4.
    AHB3DIS OFFSET(16) NUMBITS(1) [
        /// AHB3 clock enabled, distributed to peripherals according to their dedicated clock enable control bits
        B_0x0 = 0,
        /// AHB3 clock disabled
        AHB3ClockDisabled = 1
    ],
    /// APB3 clock disable
/// This bit can be set in order to further reduce power consumption, when none of the APB3 peripherals from RCC_APB3ENR are used and when their clocks are disabled in RCC_APB3ENR. When this bit is set, all the APB3 peripherals clocks are off.
    APB3DIS OFFSET(17) NUMBITS(1) [
        /// APB3 clock enabled, distributed to peripherals according to their dedicated clock enable control bits
        B_0x0 = 0,
        /// APB3 clock disabled
        APB3ClockDisabled = 1
    ]
],
PLL1CFGR [
    /// PLL1 entry clock source
/// This bitfield is set and cleared by software to select PLL1 clock source. It can be written only when the PLL1 is disabled. In order to save power, when no PLL1 is used, this bitfield value must be zero.
    PLL1SRC OFFSET(0) NUMBITS(2) [
        /// No clock sent to PLL1
        NoClockSentToPLL1 = 0,
        /// MSIS clock selected as PLL1 clock entry
        MSISClockSelectedAsPLL1ClockEntry = 1,
        /// HSI16 clock selected as PLL1 clock entry
        HSI16ClockSelectedAsPLL1ClockEntry = 2,
        /// HSE clock selected as PLL1 clock entry
        HSEClockSelectedAsPLL1ClockEntry = 3
    ],
    /// PLL1 input frequency range
/// This bit is set and reset by software to select the proper reference frequency range used for PLL1. It must be written before enabling the PLL1.
/// 00-01-10: PLL1 input (ref1_ck) clock range frequency between 4 and 8 MHz
    PLL1RGE OFFSET(2) NUMBITS(2) [
        /// PLL1 input (ref1_ck) clock range frequency between 8 and 16 MHz
        PLL1InputRef1_ckClockRangeFrequencyBetween8And16MHz = 3
    ],
    /// PLL1 fractional latch enable
/// This bit is set and reset by software to latch the content of PLL1FRACN in the ΣΔ modulator. In order to latch the PLL1FRACN value into the ΣΔ modulator, PLL1FRACEN must be set to 0, then set to 1: the transition 0 to 1 transfers the content of PLL1FRACN into the modulator (see PLL initialization phase for details).
    PLL1FRACEN OFFSET(4) NUMBITS(1) [],
    /// Prescaler for PLL1
/// This bitfield is set and cleared by software to configure the prescaler of the PLL1. The VCO1 input frequency is PLL1 input clock frequency/PLL1M.
/// This bit can be written only when the PLL1 is disabled (PLL1ON = 0 and PLL1RDY = 0).
/// ...
    PLL1M OFFSET(8) NUMBITS(4) [
        /// division by 1 (bypass)
        DivisionBy1Bypass = 0,
        /// division by 2
        DivisionBy2 = 1,
        /// division by 3
        DivisionBy3 = 2,
        /// division by 16
        DivisionBy16 = 15
    ],
    /// Prescaler for EPOD booster input clock
/// This bitfield is set and cleared by software to configure the prescaler of the PLL1, used for the EPOD booster. The EPOD booster input frequency is PLL1�input�clock�frequency/PLL1MBOOST.
/// This bit can be written only when the PLL1 is disabled (PLL1ON = 0 and PLL1RDY = 0) and EPODboost mode is disabled (see Section�10: Power control (PWR)).
/// others: reserved
    PLL1MBOOST OFFSET(12) NUMBITS(4) [
        /// division by 1 (bypass)
        DivisionBy1Bypass = 0,
        /// division by 2
        DivisionBy2 = 1,
        /// division by 4
        DivisionBy4 = 2,
        /// division by 6
        DivisionBy6 = 3,
        /// division by 8
        DivisionBy8 = 4,
        /// division by 10
        DivisionBy10 = 5,
        /// division by 12
        DivisionBy12 = 6,
        /// division by 14
        DivisionBy14 = 7,
        /// division by 16
        DivisionBy16 = 8
    ],
    /// PLL1 DIVP divider output enable
/// This bit is set and reset by software to enable the pll1_p_ck output of the PLL1. To save power, PLL1PEN and PLL1P bits must be set to 0 when pll1_p_ck is not used.
    PLL1PEN OFFSET(16) NUMBITS(1) [
        /// pll1_p_ck output disabled
        Pll1_p_ckOutputDisabled = 0,
        /// pll1_p_ck output enabled
        Pll1_p_ckOutputEnabled = 1
    ],
    /// PLL1 DIVQ divider output enable
/// This bit is set and reset by software to enable the pll1_q_ck output of the PLL1. To save power, PLL1QEN and PLL1Q bits must be set to 0 when pll1_q_ck is not used.
    PLL1QEN OFFSET(17) NUMBITS(1) [
        /// pll1_q_ck output disabled
        Pll1_q_ckOutputDisabled = 0,
        /// pll1_q_ck output enabled
        Pll1_q_ckOutputEnabled = 1
    ],
    /// PLL1 DIVR divider output enable
/// This bit is set and reset by software to enable the pll1_r_ck output of the PLL1. To save power, PLL1RENPLL2REN and PLL1R bits must be set to 0 when pll1_r_ck is not used. This bit can be cleared only when the PLL1 is not used as SYSCLK.
    PLL1REN OFFSET(18) NUMBITS(1) [
        /// pll1_r_ck output disabled
        Pll1_r_ckOutputDisabled = 0,
        /// pll1_r_ck output enabled
        Pll1_r_ckOutputEnabled = 1
    ]
],
PLL2CFGR [
    /// PLL2 entry clock source
/// This bitfield is set and cleared by software to select PLL2 clock source. It can be written only when the PLL2 is disabled. To save power, when no PLL2 is used, this bitfield value must be�zero.
    PLL2SRC OFFSET(0) NUMBITS(2) [
        /// No clock sent to PLL2
        NoClockSentToPLL2 = 0,
        /// MSIS clock selected as PLL2 clock entry
        MSISClockSelectedAsPLL2ClockEntry = 1,
        /// HSI16 clock selected as PLL2 clock entry
        HSI16ClockSelectedAsPLL2ClockEntry = 2,
        /// HSE clock selected as PLL2 clock entry
        HSEClockSelectedAsPLL2ClockEntry = 3
    ],
    /// PLL2 input frequency range
/// This bitfield is set and reset by software to select the proper reference frequency range used for�PLL2. It must be written before enabling the PLL2.
/// 00-01-10: PLL2 input (ref2_ck) clock range frequency between 4 and 8 MHz
    PLL2RGE OFFSET(2) NUMBITS(2) [
        /// PLL2 input (ref2_ck) clock range frequency between 8 and 16 MHz
        PLL2InputRef2_ckClockRangeFrequencyBetween8And16MHz = 3
    ],
    /// PLL2 fractional latch enable
/// This bit is set and reset by software to latch the content of PLL2FRACN in the ΣΔ modulator. In order to latch the PLL2FRACN value into the ΣΔ modulator, PLL2FRACEN must be set to 0, then set to 1: the transition 0 to 1 transfers the content of PLL2FRACN into the modulator (see PLL initialization phase for details).
    PLL2FRACEN OFFSET(4) NUMBITS(1) [],
    /// Prescaler for PLL2
/// This bitfield is set and cleared by software to configure the prescaler of the PLL2. The VCO2 input frequency is PLL2 input clock frequency/PLL2M.
/// This bit can be written only when the PLL2 is disabled (PLL2ON = 0 and PLL2RDY = 0).
/// ...
    PLL2M OFFSET(8) NUMBITS(4) [
        /// division by 1 (bypass)
        DivisionBy1Bypass = 0,
        /// division by 2
        DivisionBy2 = 1,
        /// division by 3
        DivisionBy3 = 2,
        /// division by 16
        DivisionBy16 = 15
    ],
    /// PLL2 DIVP divider output enable
/// This bit is set and reset by software to enable the pll2_p_ck output of the PLL2. To save power, PLL2PEN and PLL2P bits must be set to 0 when pll2_p_ck is not used.
    PLL2PEN OFFSET(16) NUMBITS(1) [
        /// pll2_p_ck output disabled
        Pll2_p_ckOutputDisabled = 0,
        /// pll2_p_ck output enabled
        Pll2_p_ckOutputEnabled = 1
    ],
    /// PLL2 DIVQ divider output enable
/// This bit is set and reset by software to enable the pll2_q_ck output of the PLL2. To save power, PLL2QEN and PLL2Q bits must be set to 0 when pll2_q_ck is not used.
    PLL2QEN OFFSET(17) NUMBITS(1) [
        /// pll2_q_ck output disabled
        Pll2_q_ckOutputDisabled = 0,
        /// pll2_q_ck output enabled
        Pll2_q_ckOutputEnabled = 1
    ],
    /// PLL2 DIVR divider output enable
/// This bit is set and reset by software to enable the pll2_r_ck output of the PLL2. To save power, PLL2REN and PLL2R bits must be set to 0 when pll2_r_ck is not used.
    PLL2REN OFFSET(18) NUMBITS(1) [
        /// pll2_r_ck output disabled
        Pll2_r_ckOutputDisabled = 0,
        /// pll2_r_ck output enabled
        Pll2_r_ckOutputEnabled = 1
    ]
],
    PLL3CFGR [
    /// PLL3 entry clock source
/// This bitfield is set and cleared by software to select PLL3 clock source. It can be written only when the PLL3 is disabled. To save power, when no PLL3 is used, this bitfield value must be�zero.
    PLL3SRC OFFSET(0) NUMBITS(2) [
        /// No clock sent to PLL3
        NoClockSentToPLL3 = 0,
        /// MSIS clock selected as PLL3 clock entry
        MSISClockSelectedAsPLL3ClockEntry = 1,
        /// HSI16 clock selected as PLL3 clock entry
        HSI16ClockSelectedAsPLL3ClockEntry = 2,
        /// HSE clock selected as PLL3 clock entry
        HSEClockSelectedAsPLL3ClockEntry = 3
    ],
    /// PLL3 input frequency range
/// This bit is set and reset by software to select the proper reference frequency range used for�PLL3. It must be written before enabling the PLL3.
/// 00-01-10: PLL3 input (ref3_ck) clock range frequency between 4 and 8 MHz
    PLL3RGE OFFSET(2) NUMBITS(2) [
        /// PLL3 input (ref3_ck) clock range frequency between 8 and 16 MHz
        PLL3InputRef3_ckClockRangeFrequencyBetween8And16MHz = 3
    ],
    /// PLL3 fractional latch enable
/// This bit is set and reset by software to latch the content of PLL3FRACN in the ΣΔ modulator. In order to latch the PLL3FRACN value into the ΣΔ modulator, PLL3FRACEN must be set to 0, then set to 1: the transition 0 to 1 transfers the content of PLL3FRACN into the modulator (see PLL initialization phase for details).
    PLL3FRACEN OFFSET(4) NUMBITS(1) [],
    /// Prescaler for PLL3
/// This bitfield is set and cleared by software to configure the prescaler of the PLL3. The VCO3 input frequency is PLL3 input clock frequency/PLL3M. This bitfield can be written only when the PLL3 is disabled (PLL3ON = 0 and PLL3RDY = 0).
/// ...
    PLL3M OFFSET(8) NUMBITS(4) [
        /// division by 1 (bypass)
        DivisionBy1Bypass = 0,
        /// division by 2
        DivisionBy2 = 1,
        /// division by 3
        DivisionBy3 = 2,
        /// division by 16
        DivisionBy16 = 15
    ],
    /// PLL3 DIVP divider output enable
/// This bit is set and reset by software to enable the pll3_p_ck output of the PLL3. To save power, PLL3PEN and PLL3P bits must be set to 0 when pll3_p_ck is not used.
    PLL3PEN OFFSET(16) NUMBITS(1) [
        /// pll3_p_ck output disabled
        Pll3_p_ckOutputDisabled = 0,
        /// pll3_p_ck output enabled
        Pll3_p_ckOutputEnabled = 1
    ],
    /// PLL3 DIVQ divider output enable
/// This bit is set and reset by software to enable the pll3_q_ck output of the PLL3. To save power, PLL3QEN and PLL3Q bits must be set to 0 when pll3_q_ck is not used.
    PLL3QEN OFFSET(17) NUMBITS(1) [
        /// pll3_q_ck output disabled
        Pll3_q_ckOutputDisabled = 0,
        /// pll3_q_ck output enabled
        Pll3_q_ckOutputEnabled = 1
    ],
    /// PLL3 DIVR divider output enable
/// This bit is set and reset by software to enable the pll3_r_ck output of the PLL3. To save power, PLL3REN and PLL3R bits must be set to 0 when pll3_r_ck is not used.
    PLL3REN OFFSET(18) NUMBITS(1) [
        /// pll3_r_ck output disabled
        Pll3_r_ckOutputDisabled = 0,
        /// pll3_r_ck output enabled
        Pll3_r_ckOutputEnabled = 1
    ]
],
    PLL1DIVR [
    /// Multiplication factor for PLL1 VCO
/// This bitfield is set and reset by software to control the multiplication factor of the VCO. It can be written only when the PLL is disabled (PLL1ON = 0 and PLL1RDY = 0).
/// ...
/// ...
/// Others: reserved
/// VCO output frequency = F<sub>ref1_ck</sub> x PLL1N, when fractional value 0 has been loaded in PLL1FRACN, with:
/// PLL1N between 4 and 512
/// input frequency F<sub>ref1_ck</sub> between 4 and 16�MHz
    PLL1N OFFSET(0) NUMBITS(9) [
        /// PLL1N = 4
        PLL1N4 = 3,
        /// PLL1N = 5
        PLL1N5 = 4,
        /// PLL1N = 6
        PLL1N6 = 5,
        /// PLL1N = 129 (default after reset)
        PLL1N129DefaultAfterReset = 128,
        /// PLL1N = 512
        PLL1N512 = 511
    ],
    /// PLL1 DIVP division factor
/// This bitfield is set and reset by software to control the frequency of the pll1_p_ck clock. It can be written only when the PLL1 is disabled (PLL1ON = 0 and PLL1RDY = 0).
/// ...
    PLL1P OFFSET(9) NUMBITS(7) [
        /// Not allowed
        NotAllowed = 0,
        /// pll1_p_ck = vco1_ck / 2 (default after reset)
        Pll1_p_ckVco1_ck2DefaultAfterReset = 1,
        /// pll1_p_ck = vco1_ck
        Pll1_p_ckVco1_ck = 2,
        /// pll1_p_ck = vco1_ck / 4
        Pll1_p_ckVco1_ck4 = 3,
        /// pll1_p_ck = vco1_ck / 128
        Pll1_p_ckVco1_ck128 = 127
    ],
    /// PLL1 DIVQ division factor
/// This bitfield is set and reset by software to control the frequency of the pll1_q_ck clock. It can be written only when the PLL1 is disabled (PLL1ON = 0 and PLL1RDY = 0).
/// ...
    PLL1Q OFFSET(16) NUMBITS(7) [
        /// pll1_q_ck = vco1_ck
        Pll1_q_ckVco1_ck = 0,
        /// pll1_q_ck = vco1_ck / 2 (default after reset)
        Pll1_q_ckVco1_ck2DefaultAfterReset = 1,
        /// pll1_q_ck = vco1_ck / 3
        Pll1_q_ckVco1_ck3 = 2,
        /// pll1_q_ck = vco1_ck / 4
        Pll1_q_ckVco1_ck4 = 3,
        /// pll1_q_ck = vco1_ck / 128
        Pll1_q_ckVco1_ck128 = 127
    ],
    /// PLL1 DIVR division factor
/// This bitfield is set and reset by software to control frequency of the pll1_r_ck clock. It can be written only when the PLL1 is disabled (PLL1ON = 0 and PLL1RDY = 0). Only division by one and even division factors are allowed.
/// ...
    PLL1R OFFSET(24) NUMBITS(7) [
        /// Not allowed
        NotAllowed = 0,
        /// pll1_r_ck = vco1_ck / 2 (default after reset)
        Pll1_r_ckVco1_ck2DefaultAfterReset = 1,
        /// pll1_r_ck = vco1_ck / 3
        Pll1_r_ckVco1_ck3 = 2,
        /// pll1_r_ck = vco1_ck / 4
        Pll1_r_ckVco1_ck4 = 3,
        /// pll1_r_ck = vco1_ck / 128
        Pll1_r_ckVco1_ck128 = 127
    ]

],
    PLL2DIVR [
    /// Multiplication factor for PLL2 VCO
/// This bitfield is set and reset by software to control the multiplication factor of the VCO. It can be written only when the PLL is disabled (PLL2ON = 0 and PLL2RDY = 0).
/// ...
/// ...
/// Others: reserved
/// VCO output frequency = F<sub>ref2_ck</sub> x PLL2N, when fractional value 0 has been loaded in PLL2FRACN, with:
/// PLL2N between 4 and 512
/// input frequency F<sub>ref2_ck</sub> between 1MHz and 16MHz
    PLL2N OFFSET(0) NUMBITS(9) [
        /// PLL2N = 4
        PLL2N4 = 3,
        /// PLL2N = 5
        PLL2N5 = 4,
        /// PLL2N = 6
        PLL2N6 = 5,
        /// PLL2N = 129 (default after reset)
        PLL2N129DefaultAfterReset = 128,
        /// PLL2N = 512
        PLL2N512 = 511
    ],
    /// PLL2 DIVP division factor
/// This bitfield is set and reset by software to control the frequency of the pll2_p_ck clock. It can be written only when the PLL2 is disabled (PLL2ON = 0 and PLL2RDY = 0).
/// ...
    PLL2P OFFSET(9) NUMBITS(7) [
        /// pll2_p_ck = vco2_ck
        Pll2_p_ckVco2_ck = 0,
        /// pll2_p_ck = vco2_ck / 2 (default after reset)
        Pll2_p_ckVco2_ck2DefaultAfterReset = 1,
        /// pll2_p_ck = vco2_ck / 3
        Pll2_p_ckVco2_ck3 = 2,
        /// pll2_p_ck = vco2_ck / 4
        Pll2_p_ckVco2_ck4 = 3,
        /// pll2_p_ck = vco2_ck / 128
        Pll2_p_ckVco2_ck128 = 127
    ],
    /// PLL2 DIVQ division factor
/// This bitfield is set and reset by software to control the frequency of the pll2_q_ck clock. It can be written only when the PLL2 is disabled (PLL2ON = 0 and PLL2RDY = 0).
/// ...
    PLL2Q OFFSET(16) NUMBITS(7) [
        /// pll2_q_ck = vco2_ck
        Pll2_q_ckVco2_ck = 0,
        /// pll2_q_ck = vco2_ck / 2 (default after reset)
        Pll2_q_ckVco2_ck2DefaultAfterReset = 1,
        /// pll2_q_ck = vco2_ck / 3
        Pll2_q_ckVco2_ck3 = 2,
        /// pll2_q_ck = vco2_ck / 4
        Pll2_q_ckVco2_ck4 = 3,
        /// pll2_q_ck = vco2_ck / 128
        Pll2_q_ckVco2_ck128 = 127
    ],
    /// PLL2 DIVR division factor
/// This bitfield is set and reset by software to control the frequency of the pll2_r_ck clock. It can be written only when the PLL2 is disabled (PLL2ON = 0 and PLL2RDY = 0).
/// ...
    PLL2R OFFSET(24) NUMBITS(7) [
        /// pll2_r_ck = vco2_ck
        Pll2_r_ckVco2_ck = 0,
        /// pll2_r_ck = vco2_ck / 2 (default after reset)
        Pll2_r_ckVco2_ck2DefaultAfterReset = 1,
        /// pll2_r_ck = vco2_ck / 3
        Pll2_r_ckVco2_ck3 = 2,
        /// pll2_r_ck = vco2_ck / 4
        Pll2_r_ckVco2_ck4 = 3,
        /// pll2_r_ck = vco2_ck / 128
        Pll2_r_ckVco2_ck128 = 127
    ]
],
    PLL3DIVR [
    /// Multiplication factor for PLL3 VCO
/// This bitfield is set and reset by software to control the multiplication factor of the VCO. It can be written only when the PLL is disabled (PLL3ON = 0 and PLL3RDY = 0).
/// ...
/// ...
/// Others: reserved
/// VCO output frequency = F<sub>ref3_ck</sub> x PLL3N, when fractional value 0 has been loaded in PLL3FRACN, with:
/// PLL3N between 4 and 512
/// input frequency F<sub>ref3_ck</sub> between 4 and 16MHz
    PLL3N OFFSET(0) NUMBITS(9) [
        /// PLL3N = 4
        PLL3N4 = 3,
        /// PLL3N = 5
        PLL3N5 = 4,
        /// PLL3N = 6
        PLL3N6 = 5,
        /// PLL3N = 129 (default after reset)
        PLL3N129DefaultAfterReset = 128,
        /// PLL3N = 512
        PLL3N512 = 511
    ],
    /// PLL3 DIVP division factor
/// This bitfield is set and reset by software to control the frequency of the pll3_p_ck clock. It can be written only when the PLL3 is disabled (PLL3ON = 0 and PLL3RDY = 0).
/// ...
    PLL3P OFFSET(9) NUMBITS(7) [
        /// pll3_p_ck = vco3_ck
        Pll3_p_ckVco3_ck = 0,
        /// pll3_p_ck = vco3_ck / 2 (default after reset)
        Pll3_p_ckVco3_ck2DefaultAfterReset = 1,
        /// pll3_p_ck = vco3_ck / 3
        Pll3_p_ckVco3_ck3 = 2,
        /// pll3_p_ck = vco3_ck / 4
        Pll3_p_ckVco3_ck4 = 3,
        /// pll3_p_ck = vco3_ck / 128
        Pll3_p_ckVco3_ck128 = 127
    ],
    /// PLL3 DIVQ division factor
/// This bitfield is set and reset by software to control the frequency of the pll3_q_ck clock. It can be written only when the PLL3 is disabled (PLL3ON = 0 and PLL3RDY = 0).
/// ...
    PLL3Q OFFSET(16) NUMBITS(7) [
        /// pll3_q_ck = vco3_ck
        Pll3_q_ckVco3_ck = 0,
        /// pll3_q_ck = vco3_ck / 2 (default after reset)
        Pll3_q_ckVco3_ck2DefaultAfterReset = 1,
        /// pll3_q_ck = vco3_ck / 3
        Pll3_q_ckVco3_ck3 = 2,
        /// pll3_q_ck = vco3_ck / 4
        Pll3_q_ckVco3_ck4 = 3,
        /// pll3_q_ck = vco3_ck / 128
        Pll3_q_ckVco3_ck128 = 127
    ],
    /// PLL3 DIVR division factor
/// This bitfield is set and reset by software to control the frequency of the pll3_r_ck clock. It can be written only when the PLL3 is disabled (PLL3ON = 0 and PLL3RDY = 0).
/// ...
    PLL3R OFFSET(24) NUMBITS(7) [
        /// pll3_r_ck = vco3_ck
        Pll3_r_ckVco3_ck = 0,
        /// pll3_r_ck = vco3_ck / 2 (default after reset)
        Pll3_r_ckVco3_ck2DefaultAfterReset = 1,
        /// pll3_r_ck = vco3_ck / 3
        Pll3_r_ckVco3_ck3 = 2,
        /// pll3_r_ck = vco3_ck / 4
        Pll3_r_ckVco3_ck4 = 3,
        /// pll3_r_ck = vco3_ck / 128
        Pll3_r_ckVco3_ck128 = 127
    ]
],
    CIER [
    /// LSI ready interrupt enable
/// This bit is set and cleared by software to enable/disable interrupt caused by the LSI oscillator stabilization.
    LSIRDYIE OFFSET(0) NUMBITS(1) [
        /// LSI ready interrupt disabled
        LSIReadyInterruptDisabled = 0,
        /// LSI ready interrupt enabled
        LSIReadyInterruptEnabled = 1
    ],
    /// LSE ready interrupt enable
/// This bit is set and cleared by software to enable/disable interrupt caused by the LSE oscillator stabilization.
    LSERDYIE OFFSET(1) NUMBITS(1) [
        /// LSE ready interrupt disabled
        LSEReadyInterruptDisabled = 0,
        /// LSE ready interrupt enabled
        LSEReadyInterruptEnabled = 1
    ],
    /// MSIS ready interrupt enable
/// This bit is set and cleared by software to enable/disable interrupt caused by the MSIS oscillator stabilization.
    MSISRDYIE OFFSET(2) NUMBITS(1) [
        /// MSIS ready interrupt disabled
        MSISReadyInterruptDisabled = 0,
        /// MSIS ready interrupt enabled
        MSISReadyInterruptEnabled = 1
    ],
    /// HSI16 ready interrupt enable
/// This bit is set and cleared by software to enable/disable interrupt caused by the HSI16 oscillator stabilization.
    HSIRDYIE OFFSET(3) NUMBITS(1) [
        /// HSI16 ready interrupt disabled
        HSI16ReadyInterruptDisabled = 0,
        /// HSI16 ready interrupt enabled
        HSI16ReadyInterruptEnabled = 1
    ],
    /// HSE ready interrupt enable
/// This bit is set and cleared by software to enable/disable interrupt caused by the HSE oscillator stabilization.
    HSERDYIE OFFSET(4) NUMBITS(1) [
        /// HSE ready interrupt disabled
        HSEReadyInterruptDisabled = 0,
        /// HSE ready interrupt enabled
        HSEReadyInterruptEnabled = 1
    ],
    /// HSI48 ready interrupt enable
/// This bit is set and cleared by software to enable/disable interrupt caused by the HSI48 oscillator stabilization.
    HSI48RDYIE OFFSET(5) NUMBITS(1) [
        /// HSI48 ready interrupt disabled
        HSI48ReadyInterruptDisabled = 0,
        /// HSI48 ready interrupt enabled
        HSI48ReadyInterruptEnabled = 1
    ],
    /// PLL ready interrupt enable
/// This bit is set and cleared by software to enable/disable interrupt caused by PLL1 lock.
    PLL1RDYIE OFFSET(6) NUMBITS(1) [
        /// PLL1 lock interrupt disabled
        PLL1LockInterruptDisabled = 0,
        /// PLL1 lock interrupt enabled
        PLL1LockInterruptEnabled = 1
    ],
    /// PLL2 ready interrupt enable
/// This bit is set and cleared by software to enable/disable interrupt caused by PLL2 lock.
    PLL2RDYIE OFFSET(7) NUMBITS(1) [
        /// PLL2 lock interrupt disabled
        PLL2LockInterruptDisabled = 0,
        /// PLL2 lock interrupt enabled
        PLL2LockInterruptEnabled = 1
    ],
    /// PLL3 ready interrupt enable
/// This bit is set and cleared by software to enable/disable interrupt caused by PLL3 lock.
    PLL3RDYIE OFFSET(8) NUMBITS(1) [
        /// PLL3 lock interrupt disabled
        PLL3LockInterruptDisabled = 0,
        /// PLL3 lock interrupt enabled
        PLL3LockInterruptEnabled = 1
    ],
    /// MSIK ready interrupt enable
/// This bit is set and cleared by software to enable/disable interrupt caused by the MSIK oscillator stabilization.
    MSIKRDYIE OFFSET(11) NUMBITS(1) [
        /// MSIK ready interrupt disabled
        MSIKReadyInterruptDisabled = 0,
        /// MSIK ready interrupt enabled
        MSIKReadyInterruptEnabled = 1
    ],
    /// SHSI ready interrupt enable
/// This bit is set and cleared by software to enable/disable interrupt caused by the SHSI oscillator stabilization.
    SHSIRDYIE OFFSET(12) NUMBITS(1) [
        /// SHSI ready interrupt disabled
        SHSIReadyInterruptDisabled = 0,
        /// SHSI ready interrupt enabled
        SHSIReadyInterruptEnabled = 1
    ]
],
CIFR [
    /// LSI ready interrupt flag
/// This bit is set by hardware when the LSI clock becomes stable and LSIRDYIE is set. It is cleared by software by�setting the LSIRDYC bit.
    LSIRDYF OFFSET(0) NUMBITS(1) [
        /// No clock ready interrupt caused by the LSI oscillator
        NoClockReadyInterruptCausedByTheLSIOscillator = 0,
        /// Clock ready interrupt caused by the LSI oscillator
        ClockReadyInterruptCausedByTheLSIOscillator = 1
    ],
    /// LSE ready interrupt flag
/// This bit is set by hardware when the LSE clock becomes stable and LSERDYIE is set. It is cleared by software by setting the LSERDYC bit.
    LSERDYF OFFSET(1) NUMBITS(1) [
        /// No clock ready interrupt caused by the LSE oscillator
        NoClockReadyInterruptCausedByTheLSEOscillator = 0,
        /// Clock ready interrupt caused by the LSE oscillator
        ClockReadyInterruptCausedByTheLSEOscillator = 1
    ],
    /// MSIS ready interrupt flag
/// This bit is set by hardware when the MSIS clock becomes stable and MSISRDYIE is set. It�is cleared by software by setting the MSISRDYC bit.
    MSISRDYF OFFSET(2) NUMBITS(1) [
        /// No clock ready interrupt caused by the MSIS oscillator
        NoClockReadyInterruptCausedByTheMSISOscillator = 0,
        /// Clock ready interrupt caused by the MSIS oscillator
        ClockReadyInterruptCausedByTheMSISOscillator = 1
    ],
    /// HSI16 ready interrupt flag
/// This bit is set by hardware when the HSI16 clock becomes stable and HSIRDYIE = 1 in�response to setting the HSION (see RCC_CR). When HSION = 0 but the HSI16 oscillator is enabled by the peripheral through a clock request, this bit is not set and no interrupt is generated. This bit is cleared by software by setting the HSIRDYC bit.
    HSIRDYF OFFSET(3) NUMBITS(1) [
        /// No clock ready interrupt caused by the HSI16 oscillator
        NoClockReadyInterruptCausedByTheHSI16Oscillator = 0,
        /// Clock ready interrupt caused by the HSI16 oscillator
        ClockReadyInterruptCausedByTheHSI16Oscillator = 1
    ],
    /// HSE ready interrupt flag
/// This bit is set by hardware when the HSE clock becomes stable and HSERDYIE is set. It is cleared by software by setting the HSERDYC bit.
    HSERDYF OFFSET(4) NUMBITS(1) [
        /// No clock ready interrupt caused by the HSE oscillator
        NoClockReadyInterruptCausedByTheHSEOscillator = 0,
        /// Clock ready interrupt caused by the HSE oscillator
        ClockReadyInterruptCausedByTheHSEOscillator = 1
    ],
    /// HSI48 ready interrupt flag
/// This bit is set by hardware when the HSI48 clock becomes stable and HSI48RDYIE is set. it�is cleared by software by setting the HSI48RDYC bit.
    HSI48RDYF OFFSET(5) NUMBITS(1) [
        /// No clock ready interrupt caused by the HSI48 oscillator
        NoClockReadyInterruptCausedByTheHSI48Oscillator = 0,
        /// Clock ready interrupt caused by the HSI48 oscillator
        ClockReadyInterruptCausedByTheHSI48Oscillator = 1
    ],
    /// PLL1 ready interrupt flag
/// This bit is set by hardware when the PLL1 locks and PLL1RDYIE is set. It is cleared by software by setting the PLL1RDYC bit.
    PLL1RDYF OFFSET(6) NUMBITS(1) [
        /// No clock ready interrupt caused by PLL1 lock
        NoClockReadyInterruptCausedByPLL1Lock = 0,
        /// Clock ready interrupt caused by PLL1 lock
        ClockReadyInterruptCausedByPLL1Lock = 1
    ],
    /// PLL2 ready interrupt flag
/// This bit is set by hardware when the PLL2 locks and PLL2RDYIE is set. It is cleared by software by setting the PLL2RDYC bit.
    PLL2RDYF OFFSET(7) NUMBITS(1) [
        /// No clock ready interrupt caused by PLL2 lock
        NoClockReadyInterruptCausedByPLL2Lock = 0,
        /// Clock ready interrupt caused by PLL2 lock
        ClockReadyInterruptCausedByPLL2Lock = 1
    ],
    /// PLL3 ready interrupt flag
/// This bit is set by hardware when the PLL3 locks and PLL3RDYIE is set. It is cleared by software by setting the PLL3RDYC bit.
    PLL3RDYF OFFSET(8) NUMBITS(1) [
        /// No clock ready interrupt caused by PLL3 lock
        NoClockReadyInterruptCausedByPLL3Lock = 0,
        /// Clock ready interrupt caused by PLL3 lock
        ClockReadyInterruptCausedByPLL3Lock = 1
    ],
    /// Clock security system interrupt flag
/// This bit is set by hardware when a failure is detected in the HSE oscillator. It is cleared by software by setting the CSSC bit.
    CSSF OFFSET(10) NUMBITS(1) [
        /// No clock security interrupt caused by HSE clock failure
        NoClockSecurityInterruptCausedByHSEClockFailure = 0,
        /// Clock security interrupt caused by HSE clock failure
        ClockSecurityInterruptCausedByHSEClockFailure = 1
    ],
    /// MSIK ready interrupt flag
/// This bit is set by hardware when the MSIK clock becomes stable and MSIKRDYIE is set. It is cleared by software by setting the MSIKRDYC bit.
    MSIKRDYF OFFSET(11) NUMBITS(1) [
        /// No clock ready interrupt caused by the MSIK oscillator
        NoClockReadyInterruptCausedByTheMSIKOscillator = 0,
        /// Clock ready interrupt caused by the MSIK oscillator
        ClockReadyInterruptCausedByTheMSIKOscillator = 1
    ],
    /// SHSI ready interrupt flag
/// This bit is set by hardware when the SHSI clock becomes stable and SHSIRDYIE is set. It is cleared by software by setting the SHSIRDYC bit.
    SHSIRDYF OFFSET(12) NUMBITS(1) [
        /// No clock ready interrupt caused by the SHSI oscillator
        NoClockReadyInterruptCausedByTheSHSIOscillator = 0,
        /// Clock ready interrupt caused by the SHSI oscillator
        ClockReadyInterruptCausedByTheSHSIOscillator = 1
    ]
],
CICR [
    /// LSI ready interrupt clear
/// Writing this bit to 1 clears the LSIRDYF flag. Writing 0 has no effect.
    LSIRDYC OFFSET(0) NUMBITS(1) [],
    /// LSE ready interrupt clear
/// Writing this bit to 1 clears the LSERDYF flag. Writing 0 has no effect.
    LSERDYC OFFSET(1) NUMBITS(1) [],
    /// MSIS ready interrupt clear
/// Writing this bit to 1 clears the MSISRDYF flag. Writing 0 has no effect.
    MSISRDYC OFFSET(2) NUMBITS(1) [],
    /// HSI16 ready interrupt clear
/// Writing this bit to 1 clears the HSIRDYF flag. Writing 0 has no effect.
    HSIRDYC OFFSET(3) NUMBITS(1) [],
    /// HSE ready interrupt clear
/// Writing this bit to 1 clears the HSERDYF flag. Writing 0 has no effect.
    HSERDYC OFFSET(4) NUMBITS(1) [],
    /// HSI48 ready interrupt clear
/// Writing this bit to 1 clears the HSI48RDYF flag. Writing 0 has no effect.
    HSI48RDYC OFFSET(5) NUMBITS(1) [],
    /// PLL1 ready interrupt clear
/// Writing this bit to 1 clears the PLL1RDYF flag. Writing 0 has no effect.
    PLL1RDYC OFFSET(6) NUMBITS(1) [],
    /// PLL2 ready interrupt clear
/// Writing this bit to 1 clears the PLL2RDYF flag. Writing 0 has no effect.
    PLL2RDYC OFFSET(7) NUMBITS(1) [],
    /// PLL3 ready interrupt clear
/// Writing this bit to 1 clears the PLL3RDYF flag. Writing 0 has no effect.
    PLL3RDYC OFFSET(8) NUMBITS(1) [],
    /// Clock security system interrupt clear
/// Writing this bit to 1 clears the CSSF flag. Writing 0 has no effect.
    CSSC OFFSET(10) NUMBITS(1) [],
    /// MSIK oscillator ready interrupt clear
/// Writing this bit to 1 clears the MSIKRDYF flag. Writing 0 has no effect.
    MSIKRDYC OFFSET(11) NUMBITS(1) [],
    /// SHSI oscillator ready interrupt clear
/// Writing this bit to 1 clears the SHSIRDYF flag. Writing 0 has no effect.
    SHSIRDYC OFFSET(12) NUMBITS(1) []
],
AHB1RSTR [
    /// GPDMA1 reset
/// This bit is set and cleared by software.
    GPDMA1RST OFFSET(0) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the GPDMA1.
        ResetTheGPDMA1 = 1
    ],
    /// CORDIC reset
/// This bit is set and cleared by software.
    CORDICRST OFFSET(1) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the CORDIC.
        ResetTheCORDIC = 1
    ],
    /// FMAC reset
/// This bit is set and cleared by software.
    FMACRST OFFSET(2) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the FMAC.
        ResetTheFMAC = 1
    ],
    /// MDF1 reset
/// This bit is set and cleared by software.
    MDF1RST OFFSET(3) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the MDF1.
        ResetTheMDF1 = 1
    ],
    /// CRC reset
/// This bit is set and cleared by software.
    CRCRST OFFSET(12) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the CRC.
        ResetTheCRC = 1
    ],
    /// JPEG reset
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    JPEGRST OFFSET(15) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the JPEG.
        ResetTheJPEG = 1
    ],
    /// TSC reset
/// This bit is set and cleared by software.
    TSCRST OFFSET(16) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the TSC.
        ResetTheTSC = 1
    ],
    /// RAMCFG reset
/// This bit is set and cleared by software.
    RAMCFGRST OFFSET(17) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the RAMCFG.
        ResetTheRAMCFG = 1
    ],
    /// DMA2D reset
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    DMA2DRST OFFSET(18) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the DMA2D.
        ResetTheDMA2D = 1
    ],
    /// GFXMMU reset
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    GFXMMURST OFFSET(19) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the GFXMMU.
        ResetTheGFXMMU = 1
    ],
    /// GPU2D reset
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    GPU2DRST OFFSET(20) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the GPU2D.
        ResetTheGPU2D = 1
    ]
],
AHB2RSTR1 [
    /// I/O port A reset
/// This bit is set and cleared by software.
    GPIOARST OFFSET(0) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the I/O port A.
        ResetTheIOPortA = 1
    ],
    /// I/O port B reset
/// This bit is set and cleared by software.
    GPIOBRST OFFSET(1) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the I/O port B.
        ResetTheIOPortB = 1
    ],
    /// I/O port C reset
/// This bit is set and cleared by software.
    GPIOCRST OFFSET(2) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the I/O port C.
        ResetTheIOPortC = 1
    ],
    /// I/O port D reset
/// This bit is set and cleared by software.
    GPIODRST OFFSET(3) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the I/O port D.
        ResetTheIOPortD = 1
    ],
    /// I/O port E reset
/// This bit is set and cleared by software.
    GPIOERST OFFSET(4) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the I/O port E.
        ResetTheIOPortE = 1
    ],
    /// I/O port F reset
/// This bit is set and cleared by software.
/// This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral.
/// Note: If not present, consider this bit as reserved and keep it at reset value.
    GPIOFRST OFFSET(5) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset I/O port F
        ResetIOPortF = 1
    ],
    /// I/O port G reset
/// This bit is set and cleared by software.
    GPIOGRST OFFSET(6) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the I/O port G.
        ResetTheIOPortG = 1
    ],
    /// I/O port H reset
/// This bit is set and cleared by software.
    GPIOHRST OFFSET(7) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the I/O port H.
        ResetTheIOPortH = 1
    ],
    /// I/O port I reset
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    GPIOIRST OFFSET(8) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the I/O port .I
        ResetTheIOPortI = 1
    ],
    /// I/O port J reset
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    GPIOJRST OFFSET(9) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the I/O port J.
        ResetTheIOPortJ = 1
    ],
    /// ADC1 and ADC2 reset
/// This bit is set and cleared by software.
/// Note: This bit impacts ADC1 in STM32U535/545/575/585, and ADC1/ADC2 in�STM32U59x/5Ax/5Fx/5Gx.
    ADC12RST OFFSET(10) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the ADC1 and ADC2.
        ResetTheADC1AndADC2 = 1
    ],
    /// DCMI and PSSI reset
/// This bit is set and cleared by software.
    DCMI_PSSIRST OFFSET(12) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the DCMI and PSSI.
        ResetTheDCMIAndPSSI = 1
    ],
    /// OTG_FS or OTG_HS reset
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    OTGRST OFFSET(14) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the OTG_FS or OTG_HS.
        ResetTheOTG_FSOrOTG_HS = 1
    ],
    /// AES hardware accelerator reset
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    AESRST OFFSET(16) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the AES.
        ResetTheAES = 1
    ],
    /// HASH reset
/// This bit is set and cleared by software.
    HASHRST OFFSET(17) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the HASH.
        ResetTheHASH = 1
    ],
    /// RNG reset
/// This bit is set and cleared by software.
    RNGRST OFFSET(18) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the RNG.
        ResetTheRNG = 1
    ],
    /// PKA reset
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    PKARST OFFSET(19) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the PKA.
        ResetThePKA = 1
    ],
    /// SAES hardware accelerator reset
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    SAESRST OFFSET(20) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the SAES.
        ResetTheSAES = 1
    ],
    /// OCTOSPIM reset
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    OCTOSPIMRST OFFSET(21) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the OCTOSPIM.
        ResetTheOCTOSPIM = 1
    ],
    /// OTFDEC1 reset
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    OTFDEC1RST OFFSET(23) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the OTFDEC1.
        ResetTheOTFDEC1 = 1
    ],
    /// OTFDEC2 reset
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    OTFDEC2RST OFFSET(24) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the OTFDEC2.
        ResetTheOTFDEC2 = 1
    ],
    /// SDMMC1 reset
/// This bit is set and cleared by software.
    SDMMC1RST OFFSET(27) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the SDMMC1.
        ResetTheSDMMC1 = 1
    ],
    /// SDMMC2 reset
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    SDMMC2RST OFFSET(28) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the SDMMC2.
        ResetTheSDMMC2 = 1
    ]
],
AHB2RSTR2 [
    /// Flexible memory controller reset
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    FSMCRST OFFSET(0) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the FSMC
        ResetTheFSMC = 1
    ],
    /// OCTOSPI1 reset
/// This bit is set and cleared by software.
    OCTOSPI1RST OFFSET(4) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the OCTOSPI1.
        ResetTheOCTOSPI1 = 1
    ],
    /// OCTOSPI2 reset
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    OCTOSPI2RST OFFSET(8) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the OCTOSPI2.
        ResetTheOCTOSPI2 = 1
    ],
    /// HSPI1 reset
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    HSPI1RST OFFSET(12) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the HSPI1.
        ResetTheHSPI1 = 1
    ]
],
AHB3RSTR [
    /// LPGPIO1 reset
/// This bit is set and cleared by software.
    LPGPIO1RST OFFSET(0) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the LPGPIO1.
        ResetTheLPGPIO1 = 1
    ],
    /// ADC4 reset
/// This bit is set and cleared by software.
    ADC4RST OFFSET(5) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the ADC4 interface.
        ResetTheADC4Interface = 1
    ],
    /// DAC1 reset
/// This bit is set and cleared by software.
    DAC1RST OFFSET(6) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the DAC1.
        ResetTheDAC1 = 1
    ],
    /// LPDMA1 reset
/// This bit is set and cleared by software.
    LPDMA1RST OFFSET(9) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the LPDMA1.
        ResetTheLPDMA1 = 1
    ],
    /// ADF1 reset
/// This bit is set and cleared by software.
    ADF1RST OFFSET(10) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the ADF1.
        ResetTheADF1 = 1
    ]
],
APB1RSTR1 [
    /// TIM2 reset
/// This bit is set and cleared by software.
    TIM2RST OFFSET(0) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the TIM2.
        ResetTheTIM2 = 1
    ],
    /// TIM3 reset
/// This bit is set and cleared by software.
    TIM3RST OFFSET(1) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the TIM3.
        ResetTheTIM3 = 1
    ],
    /// TIM4 reset
/// This bit is set and cleared by software.
    TIM4RST OFFSET(2) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the TIM4.
        ResetTheTIM4 = 1
    ],
    /// TIM5 reset
/// This bit is set and cleared by software.
    TIM5RST OFFSET(3) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the TIM5.
        ResetTheTIM5 = 1
    ],
    /// TIM6 reset
/// This bit is set and cleared by software.
    TIM6RST OFFSET(4) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the TIM6.
        ResetTheTIM6 = 1
    ],
    /// TIM7 reset
/// This bit is set and cleared by software.
    TIM7RST OFFSET(5) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the TIM7.
        ResetTheTIM7 = 1
    ],
    /// SPI2 reset
/// This bit is set and cleared by software.
    SPI2RST OFFSET(14) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the SPI2.
        ResetTheSPI2 = 1
    ],
    /// USART2 reset
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series.Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    USART2RST OFFSET(17) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the USART2
        ResetTheUSART2 = 1
    ],
    /// USART3 reset
/// This bit is set and cleared by software.
    USART3RST OFFSET(18) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the USART3.
        ResetTheUSART3 = 1
    ],
    /// UART4 reset
/// This bit is set and cleared by software.
    UART4RST OFFSET(19) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the UART4.
        ResetTheUART4 = 1
    ],
    /// UART5 reset
/// This bit is set and cleared by software.
    UART5RST OFFSET(20) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the UART5.
        ResetTheUART5 = 1
    ],
    /// I2C1 reset
/// This bit is set and cleared by software.
    I2C1RST OFFSET(21) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the I2C1.
        ResetTheI2C1 = 1
    ],
    /// I2C2 reset
/// This bit is set and cleared by software.
    I2C2RST OFFSET(22) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the I2C2.
        ResetTheI2C2 = 1
    ],
    /// CRS reset
/// This bit is set and cleared by software.
    CRSRST OFFSET(24) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the CRS.
        ResetTheCRS = 1
    ],
    /// USART6 reset
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    USART6RST OFFSET(25) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the USART6.
        ResetTheUSART6 = 1
    ]
],
APB1RSTR2 [
    /// I2C4 reset
/// This bit is set and cleared by software
    I2C4RST OFFSET(1) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the I2C4.
        ResetTheI2C4 = 1
    ],
    /// LPTIM2 reset
/// This bit is set and cleared by software.
    LPTIM2RST OFFSET(5) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the LPTIM2.
        ResetTheLPTIM2 = 1
    ],
    /// I2C5 reset
/// This bit is set and cleared by software
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    I2C5RST OFFSET(6) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the I2C5.
        ResetTheI2C5 = 1
    ],
    /// I2C6 reset
/// This bit is set and cleared by software
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    I2C6RST OFFSET(7) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the I2C6.
        ResetTheI2C6 = 1
    ],
    /// FDCAN1 reset
/// This bit is set and cleared by software.
    FDCAN1RST OFFSET(9) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the FDCAN1.
        ResetTheFDCAN1 = 1
    ],
    /// UCPD1 reset
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    UCPD1RST OFFSET(23) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the UCPD1.
        ResetTheUCPD1 = 1
    ]
],
APB2RSTR [
    /// TIM1 reset
/// This bit is set and cleared by software.
    TIM1RST OFFSET(11) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the TIM1.
        ResetTheTIM1 = 1
    ],
    /// SPI1 reset
/// This bit is set and cleared by software.
    SPI1RST OFFSET(12) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the SPI1.
        ResetTheSPI1 = 1
    ],
    /// TIM8 reset
/// This bit is set and cleared by software.
    TIM8RST OFFSET(13) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the TIM8.
        ResetTheTIM8 = 1
    ],
    /// USART1 reset
/// This bit is set and cleared by software.
    USART1RST OFFSET(14) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the USART1.
        ResetTheUSART1 = 1
    ],
    /// TIM15 reset
/// This bit is set and cleared by software.
    TIM15RST OFFSET(16) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the TIM15.
        ResetTheTIM15 = 1
    ],
    /// TIM16 reset
/// This bit is set and cleared by software.
    TIM16RST OFFSET(17) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the TIM16.
        ResetTheTIM16 = 1
    ],
    /// TIM17 reset
/// This bit is set and cleared by software.
    TIM17RST OFFSET(18) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the TIM17.
        ResetTheTIM17 = 1
    ],
    /// SAI1 reset
/// This bit is set and cleared by software.
    SAI1RST OFFSET(21) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the SAI1.
        ResetTheSAI1 = 1
    ],
    /// SAI2 reset
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    SAI2RST OFFSET(22) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the SAI2.
        ResetTheSAI2 = 1
    ],
    /// USB reset
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    USBRST OFFSET(24) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the USB.
        ResetTheUSB = 1
    ],
    /// GFXTIM reset
/// This bit is set and cleared by software.
/// Note: .This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    GFXTIMRST OFFSET(25) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the GFXTIM.
        ResetTheGFXTIM = 1
    ],
    /// LTDC reset
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    LTDCRST OFFSET(26) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the LTDC.
        ResetTheLTDC = 1
    ],
    /// DSI reset
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    DSIRST OFFSET(27) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the DSI.
        ResetTheDSI = 1
    ]
],
APB3RSTR [
    /// SYSCFG reset
/// This bit is set and cleared by software.
    SYSCFGRST OFFSET(1) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the SYSCFG.
        ResetTheSYSCFG = 1
    ],
    /// SPI3 reset
/// This bit is set and cleared by software.
    SPI3RST OFFSET(5) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the SPI3.
        ResetTheSPI3 = 1
    ],
    /// LPUART1 reset
/// This bit is set and cleared by software.
    LPUART1RST OFFSET(6) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the LPUART1.
        ResetTheLPUART1 = 1
    ],
    /// I2C3 reset
/// This bit is set and cleared by software.
    I2C3RST OFFSET(7) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the I2C3.
        ResetTheI2C3 = 1
    ],
    /// LPTIM1 reset
/// This bit is set and cleared by software.
    LPTIM1RST OFFSET(11) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the LPTIM1.
        ResetTheLPTIM1 = 1
    ],
    /// LPTIM3 reset
/// This bit is set and cleared by software.
    LPTIM3RST OFFSET(12) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the LPTIM3.
        ResetTheLPTIM3 = 1
    ],
    /// LPTIM4 reset
/// This bit is set and cleared by software.
    LPTIM4RST OFFSET(13) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the LPTIM4.
        ResetTheLPTIM4 = 1
    ],
    /// OPAMP reset
/// This bit is set and cleared by software.
    OPAMPRST OFFSET(14) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the OPAMP.
        ResetTheOPAMP = 1
    ],
    /// COMP reset
/// This bit is set and cleared by software.
    COMPRST OFFSET(15) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the COMP.
        ResetTheCOMP = 1
    ],
    /// VREFBUF reset
/// This bit is set and cleared by software.
    VREFRST OFFSET(20) NUMBITS(1) [
        /// No effect
        NoEffect = 0,
        /// Reset the VREFBUF.
        ResetTheVREFBUF = 1
    ]
],
AHB1ENR [
    /// GPDMA1 clock enable
/// This bit is set and cleared by software.
    GPDMA1EN OFFSET(0) NUMBITS(1) [
        /// GPDMA1 clock disabled
        GPDMA1ClockDisabled = 0,
        /// GPDMA1 clock enabled
        GPDMA1ClockEnabled = 1
    ],
    /// CORDIC clock enable
/// This bit is set and cleared by software.
    CORDICEN OFFSET(1) NUMBITS(1) [
        /// CORDIC clock disabled
        CORDICClockDisabled = 0,
        /// CORDIC clock enabled
        CORDICClockEnabled = 1
    ],
    /// FMAC clock enable
/// This bit is set and reset by software.
    FMACEN OFFSET(2) NUMBITS(1) [
        /// FMAC clock disabled
        FMACClockDisabled = 0,
        /// FMAC clock enabled
        FMACClockEnabled = 1
    ],
    /// MDF1 clock enable
/// This bit is set and reset by software.
    MDF1EN OFFSET(3) NUMBITS(1) [
        /// MDF1 clock disabled
        MDF1ClockDisabled = 0,
        /// MDF1 clock enabled
        MDF1ClockEnabled = 1
    ],
    /// FLASH clock enable
/// This bit is set and cleared by software. This bit can be disabled only when the flash memory is in power-down mode.
    FLASHEN OFFSET(8) NUMBITS(1) [
        /// FLASH clock disabled
        FLASHClockDisabled = 0,
        /// FLASH clock enabled
        FLASHClockEnabled = 1
    ],
    /// CRC clock enable
/// This bit is set and cleared by software.
    CRCEN OFFSET(12) NUMBITS(1) [
        /// CRC clock disabled
        CRCClockDisabled = 0,
        /// CRC clock enabled
        CRCClockEnabled = 1
    ],
    /// JPEG clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    JPEGEN OFFSET(15) NUMBITS(1) [
        /// JPEG clock disabled
        JPEGClockDisabled = 0,
        /// JPEG clock enabled
        JPEGClockEnabled = 1
    ],
    /// Touch sensing controller clock enable
/// This bit is set and cleared by software.
    TSCEN OFFSET(16) NUMBITS(1) [
        /// TSC clock disabled
        TSCClockDisabled = 0,
        /// TSC clock enabled
        TSCClockEnabled = 1
    ],
    /// RAMCFG clock enable
/// This bit is set and cleared by software.
    RAMCFGEN OFFSET(17) NUMBITS(1) [
        /// RAMCFG clock disabled
        RAMCFGClockDisabled = 0,
        /// RAMCFG clock enabled
        RAMCFGClockEnabled = 1
    ],
    /// DMA2D clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    DMA2DEN OFFSET(18) NUMBITS(1) [
        /// DMA2D clock disabled
        DMA2DClockDisabled = 0,
        /// DMA2D clock enabled
        DMA2DClockEnabled = 1
    ],
    /// GFXMMU clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    GFXMMUEN OFFSET(19) NUMBITS(1) [
        /// GFXMMU clock disabled
        GFXMMUClockDisabled = 0,
        /// GFXMMU clock enabled
        GFXMMUClockEnabled = 1
    ],
    /// GPU2D clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    GPU2DEN OFFSET(20) NUMBITS(1) [
        /// GPU2D clock disabled
        GPU2DClockDisabled = 0,
        /// GPU2D clock enabled
        GPU2DClockEnabled = 1
    ],
    /// DCACHE2 clock enable
/// This bit is set and reset by software.
/// Note: DCACHE2 clock must be enabled to access memories, even if the DCACHE2 is bypassed.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    DCACHE2EN OFFSET(21) NUMBITS(1) [
        /// DCACHE2 clock disabled
        DCACHE2ClockDisabled = 0,
        /// DCACHE2 clock enabled
        DCACHE2ClockEnabled = 1
    ],
    /// GTZC1 clock enable
/// This bit is set and reset by software.
    GTZC1EN OFFSET(24) NUMBITS(1) [
        /// GTZC1 clock disabled
        GTZC1ClockDisabled = 0,
        /// GTZC1 clock enabled
        GTZC1ClockEnabled = 1
    ],
    /// BKPSRAM clock enable
/// This bit is set and reset by software.
    BKPSRAMEN OFFSET(28) NUMBITS(1) [
        /// BKPSRAM clock disabled
        BKPSRAMClockDisabled = 0,
        /// BKPSRAM clock enabled
        BKPSRAMClockEnabled = 1
    ],
    /// DCACHE1 clock enable
/// This bit is set and reset by software.
/// Note: DCACHE1 clock must be enabled when external memories are accessed through OCTOSPI1, OCTOSPI2, HSPI1 or FSMC, even if the DCACHE1 is bypassed.
    DCACHE1EN OFFSET(30) NUMBITS(1) [
        /// DCACHE1 clock disabled
        DCACHE1ClockDisabled = 0,
        /// DCACHE1 clock enabled
        DCACHE1ClockEnabled = 1
    ],
    /// SRAM1 clock enable
/// This bit is set and reset by software.
    SRAM1EN OFFSET(31) NUMBITS(1) [
        /// SRAM1 clock disabled
        SRAM1ClockDisabled = 0,
        /// SRAM1 clock enabled
        SRAM1ClockEnabled = 1
    ]
],
AHB2ENR1 [
    /// I/O port A clock enable
/// This bit is set and cleared by software.
    GPIOAEN OFFSET(0) NUMBITS(1) [
        /// I/O port A clock disabled
        IOPortAClockDisabled = 0,
        /// I/O port A clock enabled
        IOPortAClockEnabled = 1
    ],
    /// I/O port B clock enable
/// This bit is set and cleared by software.
    GPIOBEN OFFSET(1) NUMBITS(1) [
        /// I/O port B clock disabled
        IOPortBClockDisabled = 0,
        /// I/O port B clock enabled
        IOPortBClockEnabled = 1
    ],
    /// I/O port C clock enable
/// This bit is set and cleared by software.
    GPIOCEN OFFSET(2) NUMBITS(1) [
        /// I/O port C clock disabled
        IOPortCClockDisabled = 0,
        /// I/O port C clock enabled
        IOPortCClockEnabled = 1
    ],
    /// I/O port D clock enable
/// This bit is set and cleared by software.
    GPIODEN OFFSET(3) NUMBITS(1) [
        /// I/O port D clock disabled
        IOPortDClockDisabled = 0,
        /// I/O port D clock enabled
        IOPortDClockEnabled = 1
    ],
    /// I/O port E clock enable
/// This bit is set and cleared by software.
    GPIOEEN OFFSET(4) NUMBITS(1) [
        /// I/O port E clock disabled
        IOPortEClockDisabled = 0,
        /// I/O port E clock enabled
        IOPortEClockEnabled = 1
    ],
    /// I/O port F clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    GPIOFEN OFFSET(5) NUMBITS(1) [
        /// I/O port F clock disabled
        IOPortFClockDisabled = 0,
        /// I/O port F clock enabled
        IOPortFClockEnabled = 1
    ],
    /// I/O port G clock enable
/// This bit is set and cleared by software.
    GPIOGEN OFFSET(6) NUMBITS(1) [
        /// I/O port G clock disabled
        IOPortGClockDisabled = 0,
        /// I/O port G clock enabled
        IOPortGClockEnabled = 1
    ],
    /// I/O port H clock enable
/// This bit is set and cleared by software.
    GPIOHEN OFFSET(7) NUMBITS(1) [
        /// I/O port H clock disabled
        IOPortHClockDisabled = 0,
        /// I/O port H clock enabled
        IOPortHClockEnabled = 1
    ],
    /// I/O port I clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    GPIOIEN OFFSET(8) NUMBITS(1) [
        /// I/O port I clock disabled
        IOPortIClockDisabled = 0,
        /// I/O port I clock enabled
        IOPortIClockEnabled = 1
    ],
    /// I/O port J clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    GPIOJEN OFFSET(9) NUMBITS(1) [
        /// I/O port J clock disabled
        IOPortJClockDisabled = 0,
        /// I/O port J clock enabled
        IOPortJClockEnabled = 1
    ],
    /// ADC1 and ADC2 clock enable
/// This bit is set and cleared by software.
/// Note: This bit impacts ADC1 in STM32U535/545/575/585, and ADC1/ADC2 in�STM32U59x/5Ax/5Fx/5Gx.
    ADC12EN OFFSET(10) NUMBITS(1) [
        /// ADC1 and ADC2 clock disabled
        ADC1AndADC2ClockDisabled = 0,
        /// ADC1 and ADC2 clock enabled
        ADC1AndADC2ClockEnabled = 1
    ],
    /// DCMI and PSSI clock enable
/// This bit is set and cleared by software.
    DCMI_PSSIEN OFFSET(12) NUMBITS(1) [
        /// DCMI and PSSI clock disabled
        DCMIAndPSSIClockDisabled = 0,
        /// DCMI and PSSI clock enabled
        DCMIAndPSSIClockEnabled = 1
    ],
    /// OTG_FS or OTG_HS clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    OTGEN OFFSET(14) NUMBITS(1) [
        /// OTG_FS or OTG_HS clock disabled
        OTG_FSOrOTG_HSClockDisabled = 0,
        /// OTG_FS or OTG_HS clock enabled
        OTG_FSOrOTG_HSClockEnabled = 1
    ],
    /// OTG_HS PHY clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    OTGHSPHYEN OFFSET(15) NUMBITS(1) [
        /// OTG_HS PHY clock disabled
        OTG_HSPHYClockDisabled = 0,
        /// OTG_HS PHY clock enabled
        OTG_HSPHYClockEnabled = 1
    ],
    /// AES clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    AESEN OFFSET(16) NUMBITS(1) [
        /// AES clock disabled
        AESClockDisabled = 0,
        /// AES clock enabled
        AESClockEnabled = 1
    ],
    /// HASH clock enable
/// This bit is set and cleared by software
    HASHEN OFFSET(17) NUMBITS(1) [
        /// HASH clock disabled
        HASHClockDisabled = 0,
        /// HASH clock enabled
        HASHClockEnabled = 1
    ],
    /// RNG clock enable
/// This bit is set and cleared by software.
    RNGEN OFFSET(18) NUMBITS(1) [
        /// RNG clock disabled
        RNGClockDisabled = 0,
        /// RNG clock enabled
        RNGClockEnabled = 1
    ],
    /// PKA clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    PKAEN OFFSET(19) NUMBITS(1) [
        /// PKA clock disabled
        PKAClockDisabled = 0,
        /// PKA clock enabled
        PKAClockEnabled = 1
    ],
    /// SAES clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    SAESEN OFFSET(20) NUMBITS(1) [
        /// SAES clock disabled
        SAESClockDisabled = 0,
        /// SAES clock enabled
        SAESClockEnabled = 1
    ],
    /// OCTOSPIM clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    OCTOSPIMEN OFFSET(21) NUMBITS(1) [
        /// OCTOSPIM clock disabled
        OCTOSPIMClockDisabled = 0,
        /// OCTOSPIM clock enabled
        OCTOSPIMClockEnabled = 1
    ],
    /// OTFDEC1 clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    OTFDEC1EN OFFSET(23) NUMBITS(1) [
        /// OTFDEC1 clock disabled
        OTFDEC1ClockDisabled = 0,
        /// OTFDEC1 clock enabled
        OTFDEC1ClockEnabled = 1
    ],
    /// OTFDEC2 clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    OTFDEC2EN OFFSET(24) NUMBITS(1) [
        /// OTFDEC2 clock disabled
        OTFDEC2ClockDisabled = 0,
        /// OTFDEC2 clock enabled
        OTFDEC2ClockEnabled = 1
    ],
    /// SDMMC1 clock enable
/// This bit is set and cleared by software.
    SDMMC1EN OFFSET(27) NUMBITS(1) [
        /// SDMMC1 clock disabled
        SDMMC1ClockDisabled = 0,
        /// SDMMC1 clock enabled
        SDMMC1ClockEnabled = 1
    ],
    /// SDMMC2 clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    SDMMC2EN OFFSET(28) NUMBITS(1) [
        /// SDMMC2 clock disabled
        SDMMC2ClockDisabled = 0,
        /// SDMMC2 clock enabled
        SDMMC2ClockEnabled = 1
    ],
    /// SRAM2 clock enable
/// This bit is set and reset by software.
    SRAM2EN OFFSET(30) NUMBITS(1) [
        /// SRAM2 clock disabled
        SRAM2ClockDisabled = 0,
        /// SRAM2 clock enabled
        SRAM2ClockEnabled = 1
    ],
    /// SRAM3 clock enable
/// This bit is set and reset by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    SRAM3EN OFFSET(31) NUMBITS(1) [
        /// SRAM3 clock disabled
        SRAM3ClockDisabled = 0,
        /// SRAM3 clock enabled
        SRAM3ClockEnabled = 1
    ]
],
AHB2ENR2 [
    /// FSMC clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    FSMCEN OFFSET(0) NUMBITS(1) [
        /// FSMC clock disabled
        FSMCClockDisabled = 0,
        /// FSMC clock enabled
        FSMCClockEnabled = 1
    ],
    /// OCTOSPI1 clock enable
/// This bit is set and cleared by software.
    OCTOSPI1EN OFFSET(4) NUMBITS(1) [
        /// OCTOSPI1 clock disabled
        OCTOSPI1ClockDisabled = 0,
        /// OCTOSPI1 clock enabled
        OCTOSPI1ClockEnabled = 1
    ],
    /// OCTOSPI2 clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    OCTOSPI2EN OFFSET(8) NUMBITS(1) [
        /// OCTOSPI2 clock disabled
        OCTOSPI2ClockDisabled = 0,
        /// OCTOSPI2 clock enabled
        OCTOSPI2ClockEnabled = 1
    ],
    /// HSPI1 clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    HSPI1EN OFFSET(12) NUMBITS(1) [
        /// HSPI1 clock disabled
        HSPI1ClockDisabled = 0,
        /// HSPI1 clock enabled
        HSPI1ClockEnabled = 1
    ],
    /// SRAM6 clock enable
/// This bit is set and reset by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    SRAM6EN OFFSET(30) NUMBITS(1) [
        /// SRAM6 clock disabled
        SRAM6ClockDisabled = 0,
        /// SRAM6 clock enabled
        SRAM6ClockEnabled = 1
    ],
    /// SRAM5 clock enable
/// This bit is set and reset by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    SRAM5EN OFFSET(31) NUMBITS(1) [
        /// SRAM5 clock disabled
        SRAM5ClockDisabled = 0,
        /// SRAM5 clock enabled
        SRAM5ClockEnabled = 1
    ]
],
AHB3ENR [
    /// LPGPIO1 enable
/// This bit is set and cleared by software.
    LPGPIO1EN OFFSET(0) NUMBITS(1) [
        /// LPGPIO1 clock disabled
        LPGPIO1ClockDisabled = 0,
        /// LPGPIO1 clock enabled
        LPGPIO1ClockEnabled = 1
    ],
    /// PWR clock enable
/// This bit is set and cleared by software.
    PWREN OFFSET(2) NUMBITS(1) [
        /// PWR clock disabled
        PWRClockDisabled = 0,
        /// PWR clock enabled
        PWRClockEnabled = 1
    ],
    /// ADC4 clock enable
/// This bit is set and cleared by software.
    ADC4EN OFFSET(5) NUMBITS(1) [
        /// ADC4 clock disabled
        ADC4ClockDisabled = 0,
        /// ADC4 clock enabled
        ADC4ClockEnabled = 1
    ],
    /// DAC1 clock enable
/// This bit is set and cleared by software.
    DAC1EN OFFSET(6) NUMBITS(1) [
        /// DAC1 clock disabled
        DAC1ClockDisabled = 0,
        /// DAC1 clock enabled
        DAC1ClockEnabled = 1
    ],
    /// LPDMA1 clock enable
/// This bit is set and cleared by software.
    LPDMA1EN OFFSET(9) NUMBITS(1) [
        /// LPDMA1 clock disabled
        LPDMA1ClockDisabled = 0,
        /// LPDMA1 clock enabled
        LPDMA1ClockEnabled = 1
    ],
    /// ADF1 clock enable
/// This bit is set and cleared by software.
    ADF1EN OFFSET(10) NUMBITS(1) [
        /// ADF1 clock disabled
        ADF1ClockDisabled = 0,
        /// ADF1 clock enabled
        ADF1ClockEnabled = 1
    ],
    /// GTZC2 clock enable
/// This bit is set and cleared by software.
    GTZC2EN OFFSET(12) NUMBITS(1) [
        /// GTZC2 clock disabled
        GTZC2ClockDisabled = 0,
        /// GTZC2 clock enabled
        GTZC2ClockEnabled = 1
    ],
    /// SRAM4 clock enable
/// This bit is set and reset by software.
    SRAM4EN OFFSET(31) NUMBITS(1) [
        /// SRAM4 clock disabled
        SRAM4ClockDisabled = 0,
        /// SRAM4 clock enabled
        SRAM4ClockEnabled = 1
    ]
],
APB1ENR1 [
    /// TIM2 clock enable
/// This bit is set and cleared by software.
    TIM2EN OFFSET(0) NUMBITS(1) [
        /// TIM2 clock disabled
        TIM2ClockDisabled = 0,
        /// TIM2 clock enabled
        TIM2ClockEnabled = 1
    ],
    /// TIM3 clock enable
/// This bit is set and cleared by software.
    TIM3EN OFFSET(1) NUMBITS(1) [
        /// TIM3 clock disabled
        TIM3ClockDisabled = 0,
        /// TIM3 clock enabled
        TIM3ClockEnabled = 1
    ],
    /// TIM4 clock enable
/// This bit is set and cleared by software.
    TIM4EN OFFSET(2) NUMBITS(1) [
        /// TIM4 clock disabled
        TIM4ClockDisabled = 0,
        /// TIM4 clock enabled
        TIM4ClockEnabled = 1
    ],
    /// TIM5 clock enable
/// This bit is set and cleared by software.
    TIM5EN OFFSET(3) NUMBITS(1) [
        /// TIM5 clock disabled
        TIM5ClockDisabled = 0,
        /// TIM5 clock enabled
        TIM5ClockEnabled = 1
    ],
    /// TIM6 clock enable
/// This bit is set and cleared by software.
    TIM6EN OFFSET(4) NUMBITS(1) [
        /// TIM6 clock disabled
        TIM6ClockDisabled = 0,
        /// TIM6 clock enabled
        TIM6ClockEnabled = 1
    ],
    /// TIM7 clock enable
/// This bit is set and cleared by software.
    TIM7EN OFFSET(5) NUMBITS(1) [
        /// TIM7 clock disabled
        TIM7ClockDisabled = 0,
        /// TIM7 clock enabled
        TIM7ClockEnabled = 1
    ],
    /// WWDG clock enable
/// This bit is set by software to enable the window watchdog clock. It is reset by hardware system reset. This bit can also be set by hardware if the WWDG_SW option bit is reset.
    WWDGEN OFFSET(11) NUMBITS(1) [
        /// WWDG clock disabled
        WWDGClockDisabled = 0,
        /// WWDG clock enabled
        WWDGClockEnabled = 1
    ],
    /// SPI2 clock enable
/// This bit is set and cleared by software.
    SPI2EN OFFSET(14) NUMBITS(1) [
        /// SPI2 clock disabled
        SPI2ClockDisabled = 0,
        /// SPI2 clock enabled
        SPI2ClockEnabled = 1
    ],
    /// USART2 clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    USART2EN OFFSET(17) NUMBITS(1) [
        /// USART2 clock disabled
        USART2ClockDisabled = 0,
        /// USART2 clock enabled
        USART2ClockEnabled = 1
    ],
    /// USART3 clock enable
/// This bit is set and cleared by software.
    USART3EN OFFSET(18) NUMBITS(1) [
        /// USART3 clock disabled
        USART3ClockDisabled = 0,
        /// USART3 clock enabled
        USART3ClockEnabled = 1
    ],
    /// UART4 clock enable
/// This bit is set and cleared by software.
    UART4EN OFFSET(19) NUMBITS(1) [
        /// UART4 clock disabled
        UART4ClockDisabled = 0,
        /// UART4 clock enabled
        UART4ClockEnabled = 1
    ],
    /// UART5 clock enable
/// This bit is set and cleared by software.
    UART5EN OFFSET(20) NUMBITS(1) [
        /// UART5 clock disabled
        UART5ClockDisabled = 0,
        /// UART5 clock enabled
        UART5ClockEnabled = 1
    ],
    /// I2C1 clock enable
/// This bit is set and cleared by software.
    I2C1EN OFFSET(21) NUMBITS(1) [
        /// I2C1 clock disabled
        I2C1ClockDisabled = 0,
        /// I2C1 clock enabled
        I2C1ClockEnabled = 1
    ],
    /// I2C2 clock enable
/// This bit is set and cleared by software.
    I2C2EN OFFSET(22) NUMBITS(1) [
        /// I2C2 clock disabled
        I2C2ClockDisabled = 0,
        /// I2C2 clock enabled
        I2C2ClockEnabled = 1
    ],
    /// CRS clock enable
/// This bit is set and cleared by software.
    CRSEN OFFSET(24) NUMBITS(1) [
        /// CRS clock disabled
        CRSClockDisabled = 0,
        /// CRS clock enabled
        CRSClockEnabled = 1
    ],
    /// USART6 clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    USART6EN OFFSET(25) NUMBITS(1) [
        /// USART6 clock disabled
        USART6ClockDisabled = 0,
        /// USART6 clock enabled
        USART6ClockEnabled = 1
    ]
],
APB1ENR2 [
    /// I2C4 clock enable
/// This bit is set and cleared by software
    I2C4EN OFFSET(1) NUMBITS(1) [
        /// I2C4 clock disabled
        I2C4ClockDisabled = 0,
        /// I2C4 clock enabled
        I2C4ClockEnabled = 1
    ],
    /// LPTIM2 clock enable
/// This bit is set and cleared by software.
    LPTIM2EN OFFSET(5) NUMBITS(1) [
        /// LPTIM2 clock disabled
        LPTIM2ClockDisabled = 0,
        /// LPTIM2 clock enabled
        LPTIM2ClockEnabled = 1
    ],
    /// I2C5 clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    I2C5EN OFFSET(6) NUMBITS(1) [
        /// I2C5 clock disabled
        I2C5ClockDisabled = 0,
        /// I2C5 clock enabled
        I2C5ClockEnabled = 1
    ],
    /// I2C6 clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    I2C6EN OFFSET(7) NUMBITS(1) [
        /// I2C6 clock disabled
        I2C6ClockDisabled = 0,
        /// I2C6 clock enabled
        I2C6ClockEnabled = 1
    ],
    /// FDCAN1 clock enable
/// This bit is set and cleared by software.
    FDCAN1EN OFFSET(9) NUMBITS(1) [
        /// FDCAN1 clock disabled
        FDCAN1ClockDisabled = 0,
        /// FDCAN1 clock enabled
        FDCAN1ClockEnabled = 1
    ],
    /// UCPD1 clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    UCPD1EN OFFSET(23) NUMBITS(1) [
        /// UCPD1 clock disabled
        UCPD1ClockDisabled = 0,
        /// UCPD1 clock enabled
        UCPD1ClockEnabled = 1
    ]
],
APB2ENR [
    /// TIM1 clock enable
/// This bit is set and cleared by software.
    TIM1EN OFFSET(11) NUMBITS(1) [
        /// TIM1 clock disabled
        TIM1ClockDisabled = 0,
        /// TIM1 clock enabled
        TIM1ClockEnabled = 1
    ],
    /// SPI1 clock enable
/// This bit is set and cleared by software.
    SPI1EN OFFSET(12) NUMBITS(1) [
        /// SPI1 clock disabled
        SPI1ClockDisabled = 0,
        /// SPI1 clock enabled
        SPI1ClockEnabled = 1
    ],
    /// TIM8 clock enable
/// This bit is set and cleared by software.
    TIM8EN OFFSET(13) NUMBITS(1) [
        /// TIM8 clock disabled
        TIM8ClockDisabled = 0,
        /// TIM8 clock enabled
        TIM8ClockEnabled = 1
    ],
    /// USART1clock enable
/// This bit is set and cleared by software.
    USART1EN OFFSET(14) NUMBITS(1) [
        /// USART1 clock disabled
        USART1ClockDisabled = 0,
        /// USART1 clock enabled
        USART1ClockEnabled = 1
    ],
    /// TIM15 clock enable
/// This bit is set and cleared by software.
    TIM15EN OFFSET(16) NUMBITS(1) [
        /// TIM15 clock disabled
        TIM15ClockDisabled = 0,
        /// TIM15 clock enabled
        TIM15ClockEnabled = 1
    ],
    /// TIM16 clock enable
/// This bit is set and cleared by software.
    TIM16EN OFFSET(17) NUMBITS(1) [
        /// TIM16 clock disabled
        TIM16ClockDisabled = 0,
        /// TIM16 clock enabled
        TIM16ClockEnabled = 1
    ],
    /// TIM17 clock enable
/// This bit is set and cleared by software.
    TIM17EN OFFSET(18) NUMBITS(1) [
        /// TIM17 clock disabled
        TIM17ClockDisabled = 0,
        /// TIM17 clock enabled
        TIM17ClockEnabled = 1
    ],
    /// SAI1 clock enable
/// This bit is set and cleared by software.
    SAI1EN OFFSET(21) NUMBITS(1) [
        /// SAI1 clock disabled
        SAI1ClockDisabled = 0,
        /// SAI1 clock enabled
        SAI1ClockEnabled = 1
    ],
    /// SAI2 clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    SAI2EN OFFSET(22) NUMBITS(1) [
        /// SAI2 clock disabled
        SAI2ClockDisabled = 0,
        /// SAI2 clock enabled
        SAI2ClockEnabled = 1
    ],
    /// USB clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    USBEN OFFSET(24) NUMBITS(1) [
        /// USB clock disabled
        USBClockDisabled = 0,
        /// USB clock enabled
        USBClockEnabled = 1
    ],
    /// GFXTIM clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    GFXTIMEN OFFSET(25) NUMBITS(1) [
        /// GFXTIM clock disabled
        GFXTIMClockDisabled = 0,
        /// GFXTIM clock enabled
        GFXTIMClockEnabled = 1
    ],
    /// LTDC clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    LTDCEN OFFSET(26) NUMBITS(1) [
        /// LTDC clock disabled
        LTDCClockDisabled = 0,
        /// LTDC clock enabled
        LTDCClockEnabled = 1
    ],
    /// DSI clock enable
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    DSIEN OFFSET(27) NUMBITS(1) [
        /// DSI clock disabled
        DSIClockDisabled = 0,
        /// DSI clock enabled
        DSIClockEnabled = 1
    ]
],
APB3ENR [
    /// SYSCFG clock enable
/// This bit is set and cleared by software.
    SYSCFGEN OFFSET(1) NUMBITS(1) [
        /// SYSCFG clock disabled
        SYSCFGClockDisabled = 0,
        /// SYSCFG clock enabled
        SYSCFGClockEnabled = 1
    ],
    /// SPI3 clock enable
/// This bit is set and cleared by software.
    SPI3EN OFFSET(5) NUMBITS(1) [
        /// SPI3 clock disabled
        SPI3ClockDisabled = 0,
        /// SPI3 clock enabled
        SPI3ClockEnabled = 1
    ],
    /// LPUART1 clock enable
/// This bit is set and cleared by software.
    LPUART1EN OFFSET(6) NUMBITS(1) [
        /// LPUART1 clock disabled
        LPUART1ClockDisabled = 0,
        /// LPUART1 clock enabled
        LPUART1ClockEnabled = 1
    ],
    /// I2C3 clock enable
/// This bit is set and cleared by software.
    I2C3EN OFFSET(7) NUMBITS(1) [
        /// I2C3 clock disabled
        I2C3ClockDisabled = 0,
        /// I2C3 clock enabled
        I2C3ClockEnabled = 1
    ],
    /// LPTIM1 clock enable
/// This bit is set and cleared by software.
    LPTIM1EN OFFSET(11) NUMBITS(1) [
        /// LPTIM1 clock disabled
        LPTIM1ClockDisabled = 0,
        /// LPTIM1 clock enabled
        LPTIM1ClockEnabled = 1
    ],
    /// LPTIM3 clock enable
/// This bit is set and cleared by software.
    LPTIM3EN OFFSET(12) NUMBITS(1) [
        /// LPTIM3 clock disabled
        LPTIM3ClockDisabled = 0,
        /// LPTIM3 clock enabled
        LPTIM3ClockEnabled = 1
    ],
    /// LPTIM4 clock enable
/// This bit is set and cleared by software.
    LPTIM4EN OFFSET(13) NUMBITS(1) [
        /// LPTIM4 clock disabled
        LPTIM4ClockDisabled = 0,
        /// LPTIM4 clock enabled
        LPTIM4ClockEnabled = 1
    ],
    /// OPAMP clock enable
/// This bit is set and cleared by software.
    OPAMPEN OFFSET(14) NUMBITS(1) [
        /// OPAMP clock disabled
        OPAMPClockDisabled = 0,
        /// OPAMP clock enabled
        OPAMPClockEnabled = 1
    ],
    /// COMP clock enable
/// This bit is set and cleared by software.
    COMPEN OFFSET(15) NUMBITS(1) [
        /// COMP clock disabled
        COMPClockDisabled = 0,
        /// COMP clock enabled
        COMPClockEnabled = 1
    ],
    /// VREFBUF clock enable
/// This bit is set and cleared by software.
    VREFEN OFFSET(20) NUMBITS(1) [
        /// VREFBUF clock disabled
        VREFBUFClockDisabled = 0,
        /// VREFBUF clock enabled
        VREFBUFClockEnabled = 1
    ],
    /// RTC and TAMP APB clock enable
/// This bit is set and cleared by software.
    RTCAPBEN OFFSET(21) NUMBITS(1) [
        /// RTC and TAMP APB clock disabled
        RTCAndTAMPAPBClockDisabled = 0,
        /// RTC and TAMP APB clock enabled
        RTCAndTAMPAPBClockEnabled = 1
    ]
],
AHB1SMENR [
    /// GPDMA1 clocks enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
    GPDMA1SMEN OFFSET(0) NUMBITS(1) [
        /// GPDMA1 clocks disabled by the clock gating during Sleep and Stop modes
        GPDMA1ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// GPDMA1 clocks enabled by the clock gating during Sleep and Stop modes
        GPDMA1ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// CORDIC clocks enable during Sleep and Stop modes
/// This bit is set and cleared by software during Sleep mode.
    CORDICSMEN OFFSET(1) NUMBITS(1) [
        /// CORDIC clocks disabled by the clock gating during Sleep and Stop modes
        CORDICClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// CORDIC clocks enabled by the clock gating during Sleep and Stop modes
        CORDICClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// FMAC clocks enable during Sleep and Stop modes.
/// This bit is set and cleared by software.
    FMACSMEN OFFSET(2) NUMBITS(1) [
        /// FMAC clocks disabled by the clock gating during Sleep and Stop modes
        FMACClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// FMAC clocks enabled by the clock gating during Sleep and Stop modes
        FMACClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// MDF1 clocks enable during Sleep and Stop modes.
/// This bit is set and cleared by software.
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
    MDF1SMEN OFFSET(3) NUMBITS(1) [
        /// MDF1 clocks disabled by the clock gating during Sleep and Stop modes
        MDF1ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// MDF1 clocks enabled by the clock gating during Sleep and Stop modes
        MDF1ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// FLASH clocks enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    FLASHSMEN OFFSET(8) NUMBITS(1) [
        /// FLASH clocks disabled by the clock gating during Sleep and Stop modes
        FLASHClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// FLASH clocks enabled by the clock gating during Sleep and Stop modes
        FLASHClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// CRC clocks enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    CRCSMEN OFFSET(12) NUMBITS(1) [
        /// CRC clocks disabled by the clock gating during Sleep and Stop modes
        CRCClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// CRC clocks enabled by the clock gating during Sleep and Stop modes
        CRCClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// JPEG clocks enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    JPEGSMEN OFFSET(15) NUMBITS(1) [
        /// JPEG clocks disabled by the clock gating during Sleep and Stop modes
        JPEGClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// JPEG clocks enabled by the clock gating during Sleep and Stop modes
        JPEGClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// TSC clocks enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    TSCSMEN OFFSET(16) NUMBITS(1) [
        /// TSC clocks disabled by the clock gating during Sleep and Stop modes
        TSCClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// TSC clocks enabled by the clock gating during Sleep and Stop modes
        TSCClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// RAMCFG clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    RAMCFGSMEN OFFSET(17) NUMBITS(1) [
        /// RAMCFG clocks disabled by the clock gating during Sleep and Stop modes
        RAMCFGClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// RAMCFG clocks enabled by the clock gating during Sleep and Stop modes
        RAMCFGClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// DMA2D clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    DMA2DSMEN OFFSET(18) NUMBITS(1) [
        /// DMA2D clocks disabled by the clock gating during Sleep and Stop modes
        DMA2DClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// DMA2D clocks enabled by the clock gating during Sleep and Stop modes
        DMA2DClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// GFXMMU clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    GFXMMUSMEN OFFSET(19) NUMBITS(1) [
        /// GFXMMU clocks disabled by the clock gating during Sleep and Stop modes
        GFXMMUClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// GFXMMU clocks enabled by the clock gating during Sleep and Stop modes
        GFXMMUClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// GPU2D clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    GPU2DSMEN OFFSET(20) NUMBITS(1) [
        /// GPU2D clocks disabled by the clock gating during Sleep and Stop modes
        GPU2DClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// GPU2D clocks enabled by the clock gating during Sleep and Stop modes
        GPU2DClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// DCACHE2 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    DCACHE2SMEN OFFSET(21) NUMBITS(1) [
        /// DCACHE2 clocks disabled by the clock gating during Sleep and Stop modes
        DCACHE2ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// DCACHE2 clocks enabled by the clock gating during Sleep and Stop modes
        DCACHE2ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// GTZC1 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    GTZC1SMEN OFFSET(24) NUMBITS(1) [
        /// GTZC1 clocks disabled by the clock gating during Sleep and Stop modes
        GTZC1ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// GTZC1 clocks enabled by the clock gating during Sleep and Stop modes
        GTZC1ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// BKPSRAM clock enable during Sleep and Stop modes
/// This bit is set and cleared by software
    BKPSRAMSMEN OFFSET(28) NUMBITS(1) [
        /// BKPSRAM clocks disabled by the clock gating during Sleep and Stop modes
        BKPSRAMClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// BKPSRAM clocks enabled by the clock gating during Sleep and Stop modes
        BKPSRAMClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// ICACHE clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    ICACHESMEN OFFSET(29) NUMBITS(1) [
        /// ICACHE clocks disabled by the clock gating during Sleep and Stop modes
        ICACHEClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// ICACHE clocks enabled by the clock gating during Sleep and Stop modes
        ICACHEClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// DCACHE1 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    DCACHE1SMEN OFFSET(30) NUMBITS(1) [
        /// DCACHE1 clocks disabled by the clock gating during Sleep and Stop modes
        DCACHE1ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// DCACHE1 clocks enabled by the clock gating during Sleep and Stop modes
        DCACHE1ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// SRAM1 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    SRAM1SMEN OFFSET(31) NUMBITS(1) [
        /// SRAM1 clocks disabled by the clock gating during Sleep and Stop modes
        SRAM1ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// SRAM1 clocks enabled by the clock gating during Sleep and Stop modes
        SRAM1ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ]
],
AHB2SMENR1 [
    /// I/O port A clocks enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    GPIOASMEN OFFSET(0) NUMBITS(1) [
        /// I/O port A clocks disabled by the clock gating during Sleep and Stop modes
        IOPortAClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// I/O port A clocks enabled by the clock gating during Sleep and Stop modes
        IOPortAClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// I/O port B clocks enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    GPIOBSMEN OFFSET(1) NUMBITS(1) [
        /// I/O port B clocks disabled by the clock gating during Sleep and Stop modes
        IOPortBClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// I/O port B clocks enabled by the clock gating during Sleep and Stop modes
        IOPortBClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// I/O port C clocks enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    GPIOCSMEN OFFSET(2) NUMBITS(1) [
        /// I/O port C clocks disabled by the clock gating during Sleep and Stop modes
        IOPortCClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// I/O port C clocks enabled by the clock gating during Sleep and Stop modes
        IOPortCClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// I/O port D clocks enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    GPIODSMEN OFFSET(3) NUMBITS(1) [
        /// I/O port D clocks disabled by the clock gating during Sleep and Stop modes
        IOPortDClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// I/O port D clocks enabled by the clock gating during Sleep and Stop modes
        IOPortDClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// I/O port E clocks enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    GPIOESMEN OFFSET(4) NUMBITS(1) [
        /// I/O port E clocks disabled by the clock gating during Sleep and Stop modes
        IOPortEClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// I/O port E clocks enabled by the clock gating during Sleep and Stop modes
        IOPortEClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// I/O port F clocks enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    GPIOFSMEN OFFSET(5) NUMBITS(1) [
        /// I/O port F clocks disabled by the clock gating during Sleep and Stop modes
        IOPortFClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// I/O port F clocks enabled by the clock gating during Sleep and Stop modes
        IOPortFClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// I/O port G clocks enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    GPIOGSMEN OFFSET(6) NUMBITS(1) [
        /// I/O port G clocks disabled by the clock gating during Sleep and Stop modes
        IOPortGClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// I/O port G clocks enabled by the clock gating during Sleep and Stop modes
        IOPortGClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// I/O port H clocks enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    GPIOHSMEN OFFSET(7) NUMBITS(1) [
        /// I/O port H clocks disabled by the clock gating during Sleep and Stop modes
        IOPortHClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// I/O port H clocks enabled by the clock gating during Sleep and Stop modes
        IOPortHClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// I/O port I clocks enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    GPIOISMEN OFFSET(8) NUMBITS(1) [
        /// I/O port I clocks disabled by the clock gating during Sleep and Stop modes
        IOPortIClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// I/O port I clocks enabled by the clock gating during Sleep and Stop modes
        IOPortIClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// I/O port J clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    GPIOJSMEN OFFSET(9) NUMBITS(1) [
        /// I/O port J clocks disabled by the clock gating during Sleep and Stop modes
        IOPortJClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// I/O port J clocks enabled by the clock gating during Sleep and Stop modes
        IOPortJClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// ADC1 and ADC2 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit impacts ADC1 in STM32U535/545/575/585 and ADC1/ADC2 in�STM32U59x/5Ax/5Fx/5Gx.
    ADC12SMEN OFFSET(10) NUMBITS(1) [
        /// ADC1 and ADC2 clocks disabled by the clock gating during Sleep and Stop modes
        ADC1AndADC2ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// ADC1 and ADC2 clocks enabled by the clock gating during Sleep and Stop modes
        ADC1AndADC2ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// DCMI and PSSI clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    DCMI_PSSISMEN OFFSET(12) NUMBITS(1) [
        /// DCMI and PSSI clocks disabled by the clock gating during Sleep and Stop modes
        DCMIAndPSSIClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// DCMI and PSSI clocks enabled by the clock gating during Sleep and Stop modes
        DCMIAndPSSIClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// OTG_FS and OTG_HS clocks enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    OTGSMEN OFFSET(14) NUMBITS(1) [
        /// OTG_FS and OTG_HS clocks disabled by the clock gating during Sleep and Stop modes
        OTG_FSAndOTG_HSClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// OTG_FS and OTG_HS clocks enabled by the clock gating during Sleep and Stop modes
        OTG_FSAndOTG_HSClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// OTG_HS PHY clock enable during Sleep and Stop modes
/// This bit is set and cleared by software
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    OTGHSPHYSMEN OFFSET(15) NUMBITS(1) [
        /// OTG_HS PHY clocks disabled by the clock gating during Sleep and Stop modes
        OTG_HSPHYClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// OTG_HS PHY clocks enabled by the clock gating during Sleep and Stop modes
        OTG_HSPHYClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// AES clock enable during Sleep and Stop modes
/// This bit is set and cleared by software
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    AESSMEN OFFSET(16) NUMBITS(1) [
        /// AES clocks disabled by the clock gating during Sleep and Stop modes
        AESClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// AES clocks enabled by the clock gating during Sleep and Stop modes
        AESClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// HASH clock enable during Sleep and Stop modes
/// This bit is set and cleared by software
    HASHSMEN OFFSET(17) NUMBITS(1) [
        /// HASH clocks disabled by the clock gating during Sleep and Stop modes
        HASHClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// HASH clocks enabled by the clock gating during Sleep and Stop modes
        HASHClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// RNG clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    RNGSMEN OFFSET(18) NUMBITS(1) [
        /// RNG clocks disabled by the clock gating during Sleep and Stop modes
        RNGClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// RNG clocks enabled by the clock gating during Sleep and Stop modes
        RNGClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// PKA clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    PKASMEN OFFSET(19) NUMBITS(1) [
        /// PKA clocks disabled by the clock gating during Sleep and Stop modes
        PKAClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// PKA clocks enabled by the clock gating during Sleep and Stop modes
        PKAClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// SAES accelerator clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    SAESSMEN OFFSET(20) NUMBITS(1) [
        /// SAES clocks disabled by the clock gating during Sleep and Stop modes
        SAESClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// SAES clocks enabled by the clock gating during Sleep and Stop modes
        SAESClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// OCTOSPIM clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    OCTOSPIMSMEN OFFSET(21) NUMBITS(1) [
        /// OCTOSPIM clocks disabled by the clock gating during Sleep and Stop modes
        OCTOSPIMClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// OCTOSPIM clocks enabled by the clock gating during Sleep and Stop modes
        OCTOSPIMClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// OTFDEC1 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    OTFDEC1SMEN OFFSET(23) NUMBITS(1) [
        /// OTFDEC1 clocks disabled by the clock gating during Sleep and Stop modes
        OTFDEC1ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// OTFDEC1 clocks enabled by the clock gating during Sleep and Stop modes
        OTFDEC1ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// OTFDEC2 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    OTFDEC2SMEN OFFSET(24) NUMBITS(1) [
        /// OTFDEC2 clocks disabled by the clock gating during Sleep and Stop modes
        OTFDEC2ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// OTFDEC2 clocks enabled by the clock gating during Sleep and Stop modes
        OTFDEC2ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// SDMMC1 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    SDMMC1SMEN OFFSET(27) NUMBITS(1) [
        /// SDMMC1 clocks disabled by the clock gating during Sleep and Stop modes
        SDMMC1ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// SDMMC1 clocks enabled by the clock gating during Sleep and Stop modes
        SDMMC1ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// SDMMC2 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    SDMMC2SMEN OFFSET(28) NUMBITS(1) [
        /// SDMMC2 clocks disabled by the clock gating during Sleep and Stop modes
        SDMMC2ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// SDMMC2 clocks enabled by the clock gating during Sleep and Stop modes
        SDMMC2ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// SRAM2 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    SRAM2SMEN OFFSET(30) NUMBITS(1) [
        /// SRAM2 clocks disabled by the clock gating during Sleep and Stop modes
        SRAM2ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// SRAM2 clocks enabled by the clock gating during Sleep and Stop modes
        SRAM2ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// SRAM3 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    SRAM3SMEN OFFSET(31) NUMBITS(1) [
        /// SRAM3 clocks disabled by the clock gating during Sleep and Stop modes
        SRAM3ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// SRAM3 clocks enabled by the clock gating during Sleep and Stop modes
        SRAM3ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ]
],
AHB2SMENR2 [
    /// FSMC clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    FSMCSMEN OFFSET(0) NUMBITS(1) [
        /// FSMC clocks disabled by the clock gating during Sleep and Stop modes
        FSMCClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// FSMC clocks enabled by the clock gating during Sleep and Stop modes
        FSMCClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// OCTOSPI1 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    OCTOSPI1SMEN OFFSET(4) NUMBITS(1) [
        /// OCTOSPI1 clocks disabled by the clock gating during Sleep and Stop modes
        OCTOSPI1ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// OCTOSPI1 clocks enabled by the clock gating during Sleep and Stop modes
        OCTOSPI1ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// OCTOSPI2 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    OCTOSPI2SMEN OFFSET(8) NUMBITS(1) [
        /// OCTOSPI2 clocks disabled by the clock gating during Sleep and Stop modes
        OCTOSPI2ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// OCTOSPI2 clocks enabled by the clock gating during Sleep and Stop modes
        OCTOSPI2ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// HSPI1 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    HSPI1SMEN OFFSET(12) NUMBITS(1) [
        /// HSPI1 clocks disabled by the clock gating during Sleep and Stop modes
        HSPI1ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// HSP1I clocks enabled by the clock gating during Sleep and Stop modes
        HSP1IClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// SRAM6 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    SRAM6SMEN OFFSET(30) NUMBITS(1) [
        /// SRAM6 clocks disabled by the clock gating during Sleep and Stop modes
        SRAM6ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// SRAM6 clocks enabled by the clock gating during Sleep and Stop modes
        SRAM6ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// SRAM5 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    SRAM5SMEN OFFSET(31) NUMBITS(1) [
        /// SRAM5 clocks disabled by the clock gating during Sleep and Stop modes
        SRAM5ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// SRAM5 clocks enabled by the clock gating during Sleep and Stop modes
        SRAM5ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ]
],
AHB3SMENR [
    /// LPGPIO1 enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    LPGPIO1SMEN OFFSET(0) NUMBITS(1) [
        /// LPGPIO1 clock disabled by the clock gating during Sleep and Stop modes
        LPGPIO1ClockDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// LPGPIO1 clock enabled by the clock gating during Sleep and Stop modes
        LPGPIO1ClockEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// PWR clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    PWRSMEN OFFSET(2) NUMBITS(1) [
        /// PWR clock disabled by the clock gating during Sleep and Stop modes
        PWRClockDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// PWR clock enabled by the clock gating during Sleep and Stop modes
        PWRClockEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// ADC4 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
    ADC4SMEN OFFSET(5) NUMBITS(1) [
        /// ADC4 clock disabled by the clock gating during Sleep and Stop modes
        ADC4ClockDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// ADC4 clock enabled by the clock gating during Sleep and Stop modes
        ADC4ClockEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// DAC1 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
    DAC1SMEN OFFSET(6) NUMBITS(1) [
        /// DAC1 clock disabled by the clock gating during Sleep and Stop modes
        DAC1ClockDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// DAC1 clock enabled by the clock gating during Sleep and Stop modes
        DAC1ClockEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// LPDMA1 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
    LPDMA1SMEN OFFSET(9) NUMBITS(1) [
        /// LPDMA1 clock disabled by the clock gating during Sleep and Stop modes
        LPDMA1ClockDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// LPDMA1 clock enabled by the clock gating during Sleep and Stop modes
        LPDMA1ClockEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// ADF1 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
    ADF1SMEN OFFSET(10) NUMBITS(1) [
        /// ADF1 clock disabled by the clock gating during Sleep and Stop modes
        ADF1ClockDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// ADF1 clock enabled by the clock gating during Sleep and Stop modes
        ADF1ClockEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// GTZC2 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    GTZC2SMEN OFFSET(12) NUMBITS(1) [
        /// GTZC2 clock disabled by the clock gating during Sleep and Stop modes
        GTZC2ClockDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// GTZC2 clock enabled by the clock gating during Sleep and Stop modes
        GTZC2ClockEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// SRAM4 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    SRAM4SMEN OFFSET(31) NUMBITS(1) [
        /// SRAM4 clocks disabled by the clock gating during Sleep and Stop modes
        SRAM4ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// SRAM4 clocks enabled by the clock gating during Sleep and Stop modes
        SRAM4ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ]
],
APB1SMENR1 [
    /// TIM2 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    TIM2SMEN OFFSET(0) NUMBITS(1) [
        /// TIM2 clocks disabled by the clock gating during Sleep and Stop modes
        TIM2ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// TIM2 clocks enabled by the clock gating during Sleep and Stop modes
        TIM2ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// TIM3 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    TIM3SMEN OFFSET(1) NUMBITS(1) [
        /// TIM3 clocks disabled by the clock gating during Sleep and Stop modes
        TIM3ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// TIM3 clocks enabled by the clock gating during Sleep and Stop modes
        TIM3ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// TIM4 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    TIM4SMEN OFFSET(2) NUMBITS(1) [
        /// TIM4 clocks disabled by the clock gating during Sleep and Stop modes
        TIM4ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// TIM4 clocks enabled by the clock gating during Sleep and Stop modes
        TIM4ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// TIM5 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    TIM5SMEN OFFSET(3) NUMBITS(1) [
        /// TIM5 clocks disabled by the clock gating during Sleep and Stop modes
        TIM5ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// TIM5 clocks enabled by the clock gating during Sleep and Stop modes
        TIM5ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// TIM6 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    TIM6SMEN OFFSET(4) NUMBITS(1) [
        /// TIM6 clocks disabled by the clock gating during Sleep and Stop modes
        TIM6ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// TIM6 clocks enabled by the clock gating during Sleep and Stop modes
        TIM6ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// TIM7 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    TIM7SMEN OFFSET(5) NUMBITS(1) [
        /// TIM7 clocks disabled by the clock gating during Sleep and Stop modes
        TIM7ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// TIM7 clocks enabled by the clock gating during Sleep and Stop modes
        TIM7ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// Window watchdog clock enable during Sleep and Stop modes
/// This bit is set and cleared by software. It is forced to one by hardware when the hardware WWDG option is activated.
    WWDGSMEN OFFSET(11) NUMBITS(1) [
        /// Window watchdog clocks disabled by the clock gating during Sleep and Stop modes
        WindowWatchdogClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// Window watchdog clocks enabled by the clock gating during Sleep and Stop modes
        WindowWatchdogClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// SPI2 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
    SPI2SMEN OFFSET(14) NUMBITS(1) [
        /// SPI2 clocks disabled by the clock gating during Sleep and Stop modes
        SPI2ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// SPI2 clocks enabled by the clock gating during Sleep and Stop modes
        SPI2ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// USART2 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    USART2SMEN OFFSET(17) NUMBITS(1) [
        /// USART2 clocks disabled by the clock gating during Sleep and Stop modes
        USART2ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// USART2 clocks enabled by the clock gating during Sleep and Stop modes
        USART2ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// USART3 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
    USART3SMEN OFFSET(18) NUMBITS(1) [
        /// USART3 clocks disabled by the clock gating during Sleep and Stop modes
        USART3ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// USART3 clocks enabled by the clock gating during Sleep and Stop modes
        USART3ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// UART4 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
    UART4SMEN OFFSET(19) NUMBITS(1) [
        /// UART4 clocks disabled by the clock gating during Sleep and Stop modes
        UART4ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// UART4 clocks enabled by the clock gating during Sleep and Stop modes
        UART4ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// UART5 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
    UART5SMEN OFFSET(20) NUMBITS(1) [
        /// UART5 clocks disabled by the clock gating during Sleep and Stop modes
        UART5ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// UART5 clocks enabled by the clock gating during Sleep and Stop modes
        UART5ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// I2C1 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
    I2C1SMEN OFFSET(21) NUMBITS(1) [
        /// I2C1 clocks disabled by the clock gating during Sleep and Stop modes
        I2C1ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// I2C1 clocks enabled by the clock gating during Sleep and Stop modes
        I2C1ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// I2C2 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
    I2C2SMEN OFFSET(22) NUMBITS(1) [
        /// I2C2 clocks disabled by the clock gating during Sleep and Stop modes
        I2C2ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// I2C2 clocks enabled by the clock gating during Sleep and Stop modes
        I2C2ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// CRS clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    CRSSMEN OFFSET(24) NUMBITS(1) [
        /// CRS clocks disabled by the clock gating during Sleep and Stop modes
        CRSClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// CRS clocks enabled by the clock gating during Sleep and Stop modes
        CRSClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// USART6 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    USART6SMEN OFFSET(25) NUMBITS(1) [
        /// USART6 clocks disabled by the clock gating during Sleep and Stop modes
        USART6ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// USART6 clocks enabled by the clock gating during Sleep and Stop modes
        USART6ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ]
],
APB1SMENR2 [
    /// I2C4 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
    I2C4SMEN OFFSET(1) NUMBITS(1) [
        /// I2C4 clocks disabled by the clock gating during Sleep and Stop modes
        I2C4ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// I2C4 clocks enabled by the clock gating during Sleep and Stop modes
        I2C4ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// LPTIM2 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
    LPTIM2SMEN OFFSET(5) NUMBITS(1) [
        /// LPTIM2 clocks disabled by the clock gating during Sleep and Stop modes
        LPTIM2ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// LPTIM2 clocks enabled by the clock gating during Sleep and Stop modes
        LPTIM2ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// I2C5 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    I2C5SMEN OFFSET(6) NUMBITS(1) [
        /// I2C5 clocks disabled by the clock gating during Sleep and Stop modes
        I2C5ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// I2C5 clocks enabled by the clock gating during Sleep and Stop modes
        I2C5ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// I2C6 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    I2C6SMEN OFFSET(7) NUMBITS(1) [
        /// I2C6 clocks disabled by the clock gating during Sleep and Stop modes
        I2C6ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// I2C6 clocks enabled by the clock gating during Sleep and Stop modes
        I2C6ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// FDCAN1 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    FDCAN1SMEN OFFSET(9) NUMBITS(1) [
        /// FDCAN1 clocks disabled by the clock gating during Sleep and Stop modes
        FDCAN1ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// FDCAN1 clocks enabled by the clock gating during Sleep and Stop modes
        FDCAN1ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// UCPD1 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    UCPD1SMEN OFFSET(23) NUMBITS(1) [
        /// UCPD1 clocks disabled by the clock gating during Sleep and Stop modes
        UCPD1ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// UCPD1 clocks enabled by the clock gating during Sleep and Stop modes
        UCPD1ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ]
],
APB2SMENR [
    /// TIM1 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    TIM1SMEN OFFSET(11) NUMBITS(1) [
        /// TIM1 clocks disabled by the clock gating during Sleep and Stop modes
        TIM1ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// TIM1 clocks enabled by the clock gating during Sleep and Stop modes
        TIM1ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// SPI1 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
    SPI1SMEN OFFSET(12) NUMBITS(1) [
        /// SPI1 clocks disabled by the clock gating during Sleep and Stop modes
        SPI1ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// SPI1 clocks enabled by the clock gating during Sleep and Stop modes
        SPI1ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// TIM8 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    TIM8SMEN OFFSET(13) NUMBITS(1) [
        /// TIM8 clocks disabled by the clock gating during Sleep and Stop modes
        TIM8ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// TIM8 clocks enabled by the clock gating during Sleep and Stop modes
        TIM8ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// USART1 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
    USART1SMEN OFFSET(14) NUMBITS(1) [
        /// USART1clocks disabled by the clock gating during Sleep and Stop modes
        USART1clocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// USART1clocks enabled by the clock gating during Sleep and Stop modes
        USART1clocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// TIM15 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    TIM15SMEN OFFSET(16) NUMBITS(1) [
        /// TIM15 clocks disabled by the clock gating during Sleep and Stop modes
        TIM15ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// TIM15 clocks enabled by the clock gating during Sleep and Stop modes
        TIM15ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// TIM16 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    TIM16SMEN OFFSET(17) NUMBITS(1) [
        /// TIM16 clocks disabled by the clock gating during Sleep and Stop modes
        TIM16ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// TIM16 clocks enabled by the clock gating during Sleep and Stop modes
        TIM16ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// TIM17 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    TIM17SMEN OFFSET(18) NUMBITS(1) [
        /// TIM17 clocks disabled by the clock gating during Sleep and Stop modes
        TIM17ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// TIM17 clocks enabled by the clock gating during Sleep and Stop modes
        TIM17ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// SAI1 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    SAI1SMEN OFFSET(21) NUMBITS(1) [
        /// SAI1 clocks disabled by the clock gating during Sleep and Stop modes
        SAI1ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// SAI1 clocks enabled by the clock gating during Sleep and Stop modes
        SAI1ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// SAI2 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series.Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    SAI2SMEN OFFSET(22) NUMBITS(1) [
        /// SAI2 clocks disabled by the clock gating during Sleep and Stop modes
        SAI2ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// SAI2 clocks enabled by the clock gating during Sleep and Stop modes
        SAI2ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// USB clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    USBSMEN OFFSET(24) NUMBITS(1) [
        /// USB clocks disabled by the clock gating during Sleep and Stop modes
        USBClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// USB clocks enabled by the clock gating during Sleep and Stop modes
        USBClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// GFXTIM clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    GFXTIMSMEN OFFSET(25) NUMBITS(1) [
        /// GFXTIM clocks disabled by the clock gating during Sleep and Stop modes
        GFXTIMClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// GFXTIM clocks enabled by the clock gating during Sleep and Stop modes
        GFXTIMClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// LTDC clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    LTDCSMEN OFFSET(26) NUMBITS(1) [
        /// LTDC clocks disabled by the clock gating during Sleep and Stop modes
        LTDCClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// LTDC clocks enabled by the clock gating during Sleep and Stop modes
        LTDCClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// DSI clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit is only available on some devices in the STM32U5 Series. Refer to the device datasheet for availability of its associated peripheral. If not present, consider this bit as reserved and keep it at reset value.
    DSISMEN OFFSET(27) NUMBITS(1) [
        /// DSI clocks disabled by the clock gating during Sleep and Stop modes
        DSIClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// DSI clocks enabled by the clock gating during Sleep and Stop modes
        DSIClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ]
],
APB3SMENR [
    /// SYSCFG clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    SYSCFGSMEN OFFSET(1) NUMBITS(1) [
        /// SYSCFG clocks disabled by the clock gating during Sleep and Stop modes
        SYSCFGClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// SYSCFG clocks enabled by the clock gating during Sleep and Stop modes
        SYSCFGClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// SPI3 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
    SPI3SMEN OFFSET(5) NUMBITS(1) [
        /// SPI3 clocks disabled by the clock gating during Sleep and Stop modes
        SPI3ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// SPI3 clocks enabled by the clock gating during Sleep and Stop modes
        SPI3ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// LPUART1 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
    LPUART1SMEN OFFSET(6) NUMBITS(1) [
        /// LPUART1 clocks disabled by the clock gating during Sleep and Stop modes
        LPUART1ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// LPUART1 clocks enabled by the clock gating during Sleep and Stop modes
        LPUART1ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// I2C3 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
    I2C3SMEN OFFSET(7) NUMBITS(1) [
        /// I2C3 clocks disabled by the clock gating during Sleep and Stop modes
        I2C3ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// I2C3 clocks enabled by the clock gating during Sleep and Stop modes
        I2C3ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// LPTIM1 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
    LPTIM1SMEN OFFSET(11) NUMBITS(1) [
        /// LPTIM1 clocks disabled by the clock gating during Sleep and Stop modes
        LPTIM1ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// LPTIM1 clocks enabled by the clock gating during Sleep and Stop modes
        LPTIM1ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// LPTIM3 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
    LPTIM3SMEN OFFSET(12) NUMBITS(1) [
        /// LPTIM3 clocks disabled by the clock gating during Sleep and Stop modes
        LPTIM3ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// LPTIM3 clocks enabled by the clock gating during Sleep and Stop modes
        LPTIM3ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// LPTIM4 clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
    LPTIM4SMEN OFFSET(13) NUMBITS(1) [
        /// LPTIM4 clocks disabled by the clock gating during Sleep and Stop modes
        LPTIM4ClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// LPTIM4 clocks enabled by the clock gating during Sleep and Stop modes
        LPTIM4ClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// OPAMP clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    OPAMPSMEN OFFSET(14) NUMBITS(1) [
        /// OPAMP clocks disabled by the clock gating during Sleep and Stop modes
        OPAMPClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// OPAMP clocks enabled by the clock gating during Sleep and Stop modes
        OPAMPClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// COMP clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    COMPSMEN OFFSET(15) NUMBITS(1) [
        /// COMP clocks disabled by the clock gating during Sleep and Stop modes
        COMPClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// COMP clocks enabled by the clock gating during Sleep and Stop modes
        COMPClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// VREFBUF clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
    VREFSMEN OFFSET(20) NUMBITS(1) [
        /// VREFBUF clocks disabled by the clock gating during Sleep and Stop modes
        VREFBUFClocksDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// VREFBUF clocks enabled by the clock gating during Sleep and Stop modes
        VREFBUFClocksEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ],
    /// RTC and TAMP APB clock enable during Sleep and Stop modes
/// This bit is set and cleared by software.
/// Note: This bit must be set to allow the peripheral to wake up from Stop modes.
    RTCAPBSMEN OFFSET(21) NUMBITS(1) [
        /// RTC and TAMP APB clock disabled by the clock gating during Sleep and Stop modes
        RTCAndTAMPAPBClockDisabledByTheClockGatingDuringSleepAndStopModes = 0,
        /// RTC and TAMP APB clock enabled by the clock gating during Sleep and Stop modes
        RTCAndTAMPAPBClockEnabledByTheClockGatingDuringSleepAndStopModes = 1
    ]
],

    CCIPR1 [
        /// USART1 kernel clock source selection
        USART1SEL OFFSET(0) NUMBITS(2) [
            PCLK2Selected = 0,
            SYSCLKSelected = 1,
            HSI16Selected = 2,
            LSESelected = 3
        ],
        /// USART2 kernel clock source selection
        USART2SEL OFFSET(2) NUMBITS(2) [
            PCLK1Selected = 0,
            SYSCLKSelected = 1,
            HSI16Selected = 2,
            LSESelected = 3
        ],
        /// USART3 kernel clock source selection
        USART3SEL OFFSET(4) NUMBITS(2) [
            PCLK1Selected = 0,
            SYSCLKSelected = 1,
            HSI16Selected = 2,
            LSESelected = 3
        ]
    ],

];

/// RCC base address for STM32U5x.
///
/// From RM0456 / device tree:
/// RCC at 0x4602_0C00, size 0x400.
const RCC_BASE: StaticRef<RccRegisters> =
    unsafe { StaticRef::new(0x4602_0C00 as *const RccRegisters) };

/// System clock source as seen by software.
///
/// We only care about HSI16 for now; other values are grouped into `Other`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SysClockSource {
    /// High-speed internal 16 MHz RC
    HSI16,
    /// Some other source (MSIS, HSE, PLL1, MSIK, etc.)
    Other(u8),
}

/// AHB prescaler
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AHBPrescaler {
    DivideBy1 = 0b0000,
    DivideBy2 = 0b1000,
    DivideBy4 = 0b1001,
    DivideBy8 = 0b1010,
    DivideBy16 = 0b1011,
    DivideBy64 = 0b1100,
    DivideBy128 = 0b1101,
    DivideBy256 = 0b1110,
    DivideBy512 = 0b1111,
}

impl From<AHBPrescaler> for usize {
    fn from(item: AHBPrescaler) -> usize {
        match item {
            AHBPrescaler::DivideBy1 => 1,
            AHBPrescaler::DivideBy2 => 2,
            AHBPrescaler::DivideBy4 => 4,
            AHBPrescaler::DivideBy8 => 8,
            AHBPrescaler::DivideBy16 => 16,
            AHBPrescaler::DivideBy64 => 64,
            AHBPrescaler::DivideBy128 => 128,
            AHBPrescaler::DivideBy256 => 256,
            AHBPrescaler::DivideBy512 => 512,
        }
    }
}

/// APB prescaler (PCLK1 / PCLK2)
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum APBPrescaler {
    DivideBy1 = 0b000, // no division
    DivideBy2 = 0b100,
    DivideBy4 = 0b101,
    DivideBy8 = 0b110,
    DivideBy16 = 0b111,
}

impl From<APBPrescaler> for usize {
    fn from(item: APBPrescaler) -> Self {
        match item {
            APBPrescaler::DivideBy1 => 1,
            APBPrescaler::DivideBy2 => 2,
            APBPrescaler::DivideBy4 => 4,
            APBPrescaler::DivideBy8 => 8,
            APBPrescaler::DivideBy16 => 16,
        }
    }
}

/// Minimal RCC wrapper.
pub struct Rcc {
    registers: StaticRef<RccRegisters>,
}

pub enum RtcClockSource {
    LSI,
    LSE,
    HSERTC,
}

pub enum GpioPort {
    A,
    B,
    C,
    G,
    H,
}

impl Rcc {
    /// Create an `Rcc` with raw access to the RCC registers.
    ///
    /// Does **not** change any clock configuration by itself.
    pub const fn new() -> Self {
        Self {
            registers: RCC_BASE,
        }
    }

    /* ---- SYSCLK source ---- */

    pub(crate) fn get_sys_clock_source(&self) -> SysClockSource {
        let sws = self.registers.cfgr1.read(CFGR1::SWS) as u8;
        match sws {
            0b01 => SysClockSource::HSI16,
            other => SysClockSource::Other(other),
        }
    }

    /// Set system clock source.
    ///
    /// For now, we only support switching to HSI16 explicitly; everything else
    /// is left for later stages of the port.
    pub(crate) fn set_sys_clock_source(&self, source: SysClockSource) {
        match source {
            SysClockSource::HSI16 => {
                self.registers
                    .cfgr1
                    .modify(CFGR1::SW::HSI16SelectedAsSystemClock);
            }
            SysClockSource::Other(_) => {
                // Add more variants once PLL/HSE/MSIS/MSIK are supported.
                panic!("Unsupported SYSCLK source switch requested");
            }
        }
    }

    /* ---- HSI16 clock control ---- */

    /// Disable HSI16.
    ///
    /// WARNING: do not call this while HSI16 is used as SYSCLK.
    pub(crate) fn disable_hsi_clock(&self) {
        self.registers.cr.modify(CR::HSION::CLEAR);
    }

    /// Enable HSI16.
    pub(crate) fn enable_hsi_clock(&self) {
        self.registers.cr.modify(CR::HSION::SET);
    }

    pub(crate) fn is_enabled_hsi_clock(&self) -> bool {
        self.registers.cr.is_set(CR::HSION)
    }

    /// Indicates whether the HSI16 oscillator is stable.
    pub(crate) fn is_ready_hsi_clock(&self) -> bool {
        self.registers.cr.is_set(CR::HSIRDY)
    }

    /* ---- AHB / APB prescalers ---- */

    pub(crate) fn set_ahb_prescaler(&self, ahb_prescaler: AHBPrescaler) {
        self.registers
            .cfgr2
            .modify(CFGR2::HPRE.val(ahb_prescaler as u32));
    }

    pub(crate) fn get_ahb_prescaler(&self) -> AHBPrescaler {
        match self.registers.cfgr2.read(CFGR2::HPRE) {
            0b1000 => AHBPrescaler::DivideBy2,
            0b1001 => AHBPrescaler::DivideBy4,
            0b1010 => AHBPrescaler::DivideBy8,
            0b1011 => AHBPrescaler::DivideBy16,
            0b1100 => AHBPrescaler::DivideBy64,
            0b1101 => AHBPrescaler::DivideBy128,
            0b1110 => AHBPrescaler::DivideBy256,
            0b1111 => AHBPrescaler::DivideBy512,
            _ => AHBPrescaler::DivideBy1,
        }
    }

    pub(crate) fn set_apb1_prescaler(&self, apb1_prescaler: APBPrescaler) {
        self.registers
            .cfgr2
            .modify(CFGR2::PPRE1.val(apb1_prescaler as u32));
    }

    pub(crate) fn get_apb1_prescaler(&self) -> APBPrescaler {
        match self.registers.cfgr2.read(CFGR2::PPRE1) {
            0b100 => APBPrescaler::DivideBy2,
            0b101 => APBPrescaler::DivideBy4,
            0b110 => APBPrescaler::DivideBy8,
            0b111 => APBPrescaler::DivideBy16,
            _ => APBPrescaler::DivideBy1,
        }
    }

    pub(crate) fn set_apb2_prescaler(&self, apb2_prescaler: APBPrescaler) {
        self.registers
            .cfgr2
            .modify(CFGR2::PPRE2.val(apb2_prescaler as u32));
    }

    pub(crate) fn get_apb2_prescaler(&self) -> APBPrescaler {
        match self.registers.cfgr2.read(CFGR2::PPRE2) {
            0b100 => APBPrescaler::DivideBy2,
            0b101 => APBPrescaler::DivideBy4,
            0b110 => APBPrescaler::DivideBy8,
            0b111 => APBPrescaler::DivideBy16,
            _ => APBPrescaler::DivideBy1,
        }
    }

    pub(crate) fn enable_gpio_port(&self, port: GpioPort) {
        match port {
            GpioPort::A => self.registers.ahb2enr1.modify(AHB2ENR1::GPIOAEN::SET),
            GpioPort::B => self.registers.ahb2enr1.modify(AHB2ENR1::GPIOBEN::SET),
            GpioPort::C => self.registers.ahb2enr1.modify(AHB2ENR1::GPIOCEN::SET),
            GpioPort::G => self.registers.ahb2enr1.modify(AHB2ENR1::GPIOGEN::SET),
            GpioPort::H => self.registers.ahb2enr1.modify(AHB2ENR1::GPIOHEN::SET),
        }
    }

    pub(crate) fn disable_gpio_port(&self, port: GpioPort) {
        match port {
            GpioPort::A => self.registers.ahb2enr1.modify(AHB2ENR1::GPIOAEN::CLEAR),
            GpioPort::B => self.registers.ahb2enr1.modify(AHB2ENR1::GPIOBEN::CLEAR),
            GpioPort::C => self.registers.ahb2enr1.modify(AHB2ENR1::GPIOCEN::CLEAR),
            GpioPort::G => self.registers.ahb2enr1.modify(AHB2ENR1::GPIOGEN::CLEAR),
            GpioPort::H => self.registers.ahb2enr1.modify(AHB2ENR1::GPIOHEN::CLEAR),
        }
    }

    pub(crate) fn is_enabled_gpio_port(&self, port: GpioPort) -> bool {
        match port {
            GpioPort::A => self.registers.ahb2enr1.is_set(AHB2ENR1::GPIOAEN),
            GpioPort::B => self.registers.ahb2enr1.is_set(AHB2ENR1::GPIOBEN),
            GpioPort::C => self.registers.ahb2enr1.is_set(AHB2ENR1::GPIOCEN),
            GpioPort::G => self.registers.ahb2enr1.is_set(AHB2ENR1::GPIOGEN),
            GpioPort::H => self.registers.ahb2enr1.is_set(AHB2ENR1::GPIOHEN),
        }
    }

    pub(crate) fn configure_rng_clock(&self) {
        unimplemented!()
    }

    pub(crate) fn enable_usart1_clock(&self) {
        self.registers.apb2enr.modify(APB2ENR::USART1EN::SET);
    }

    pub(crate) fn disable_usart1_clock(&self) {
        self.registers.apb2enr.modify(APB2ENR::USART1EN::CLEAR);
    }

    pub(crate) fn is_enabled_usart1_clock(&self) -> bool {
        self.registers.apb2enr.is_set(APB2ENR::USART1EN)
    }

    pub(crate) fn is_enabled_tim2_clock(&self) -> bool {
        self.registers.apb1enr1.is_set(APB1ENR1::TIM2EN)
    }

    pub(crate) fn enable_tim2_clock(&self) {
        self.registers.apb1enr1.modify(APB1ENR1::TIM2EN::SET);
    }

    pub(crate) fn disable_tim2_clock(&self) {
        self.registers.apb1enr1.modify(APB1ENR1::TIM2EN::CLEAR);
    }
}
