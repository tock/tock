# STM32U5xx Tock Chip Crate

This crate provides support for the STM32U5 series of ultra-low-power
microcontrollers from STMicroelectronics.

## Status

Currently supported peripherals:

- ADC (Analog to Digital Converter)
- AES (encryption and decryption)
- DAC (Digital to Analog Converter)
- EXTI (External Interrupts)
- GPDMA (Global Programmable DMA)
- GPIO (General Purpose I/O)
- HASH (Hash processor)
- I2C master (Inter-Integrated Circuit)
- PKA (Public Key Accelerator)
- PWM (Pulse Width Modulation)
- RCC (Reset and Clock Control)
- RTC (Real Time Clock)
- SPI master (Serial Peripheral Interface)
- TIM2 (Timer)
- TIM3 (PWM)
- TRNG (True Random Number Generator)
- USART (Universal Synchronous/Asynchronous Receiver Transmitter)

# RCC clock configuration

The RCC (Reset and Clock Control) peripheral is responsible for enabling clock
sources, configuring the PLLs, deriving the system clock (SYSCLK) and bus clocks
(HCLK/PCLK1/PCLK2/PCLK3), and routing individual  clocks to peripherals that
support multiple clock sources (USART, SPI, I2C, ADC, RTC, etc.).

See RM0456 § 10-11 for the full clock tree reference.

## The  `RccConfig` structure

`RccConfig` fully describes the desired clock setup. It gets  passed to `Rcc::init()`,
which initializes the hardware and returns a `Clocks` structure, containing the
effective frequency of every clock in the tree. This is then passed to each peripheral
driver that needs it, via its `set_clocks()`.

```rust
let mut rcc_config = RccConfig {
    msis: Some(MsiRange::Range4mhz),
    msik: Some(MsiRange::Range4mhz),
    hsi16: true,
    hse: None,
    hsi48: false,
    lsi: true,
    pll1: None,
    pll2: None,
    pll3: None,
    sys: Sysclk::Hsi,
    ahb_pre: AHBPrescaler::Div1,
    apb1_pre: APBPrescaler::Div1,
    apb2_pre: APBPrescaler::Div1,
    apb3_pre: APBPrescaler::Div1,
    voltage_range: VoltageScale::Range1,
    mux: ClockMuxConfig::default(),
};

let clocks = rcc.init(rcc_config, &pwr);

usart1.set_clocks(clocks);
...
```

### Base clock sources

These are the raw oscillators that everything else is derived from.

Each one is independently enabled/disabled:

| Field | Description |
|:-:|-|
| `msis`  | An output of the MSI oscillator, usable directly as SYSCLK; `None` disables it|
| `msik`  | An output of the MSI oscillator, usable only as a peripheral kernel clock; `None` disables it|
| `hsi16` | Internal (fixed) 16 MHz RC oscillator; `False` disables it |
| `hse`   | External crystal/resonator or external clock input, see [HSE](#hse); `None` disables it |
| `hsi48` | Internal (fixed) 48 MHz RC oscillator; `False` disables it |
| `lsi`   | Low-power 32 kHz oscillator; `False` disables it |

#### HSE

```rust
pub struct Hse {
    pub freq: Hertz,
    pub mode: HseMode,
}

pub enum HseMode {
    Oscillator,    // crystal/ceramic resonator
    Bypass,        // external analog clock
    BypassDigital, // external digital clock
}
```

Use `Oscillator` when a crystal is wired across the HSE pins. Use `Bypass` or
`BypassDigital` when an external oscillator module is driving the pin
directly, matching whichever signal type the module produces.

### PLLs

There are three independent PLLs: `pll1`, `pll2`, `pll3`; each one is configured
the same way, via `Option<Pll>`:

```rust
pub struct Pll {
    pub source: PllSource,
    pub prediv: PllPreDiv,
    pub mul: PllMul,
    pub divp: Option<PllDiv>,
    pub divq: Option<PllDiv>,
    pub divr: Option<PllDiv>,
}
```

- `source` selects the input clock (MSIS/HSI16/HSE)
- `prediv` (the "M" divider) brings the chosen input clock down into the mandatory
  **4–16 MHz** range before multiplication
- `mul` (the "N" multiplier) produces the VCO frequency, which must land between
  **128–544 MHz** (the practical ceiling is lower at lower `voltage_range` settings)
- `divp`, `divq`, `divr` are independent output dividers ("P/Q/R"), each optional:
  - **P** feeds the SAI/MDF/ADF clock muxes
  - **Q** feeds the 48 MHz-class peripherals (USB/RNG/SDMMC/OCTOSPI) and can
    also feed the MDF/ADF muxes
  - **R** is the output typically used to drive SYSCLK; when used for
    SYSCLK, the final frequency (`source / M * N / R`) must be at most **160 MHz**

Any PLL can be left disabled if nothing needs it, by setting `pll1`/`pll2`/`pll3` to `None`.

#### Example: PLL1 driving SYSCLK at 96 MHz from HSI

```rust
let rcc_config = RccConfig {
    msis: None,
    msik: None,
    hsi16: true, // enable HSI16
    hse: None,
    hsi48: false,
    lsi: true,
    pll1: Some(Pll {
        source: PllSource::Hse,   // 16 MHz
        prediv: PllPreDiv::Div1,  // 16 MHz / 1 = 16 MHz (within the 4-16 MHz input window)
        mul: PllMul::Mul12,       // 16 MHz * 12 = 192 MHz (VCO)
        divp: None,
        divq: None,
        divr: Some(PllDiv::Div2), // 192 MHz / 2 = 96 MHz SYSCLK
    }),
    pll2: None,
    pll3: None,
    sys: Sysclk::Pll1R, // select PLL1 as SYSCLK
    ahb_pre: AHBPrescaler::Div1,
    apb1_pre: APBPrescaler::Div1,
    apb2_pre: APBPrescaler::Div1,
    apb3_pre: APBPrescaler::Div1,
    voltage_range: VoltageScale::Range1,
    mux: ClockMuxConfig::default(),
};
```

### System clock and bus prescalers

```rust
pub sys: Sysclk,
pub ahb_pre: AHBPrescaler,
pub apb1_pre: APBPrescaler,
pub apb2_pre: APBPrescaler,
pub apb3_pre: APBPrescaler,
pub voltage_range: VoltageScale,
```

- `sys` selects which source becomes SYSCLK (MSIS/HSI16/HSE/PLL1)
- `ahb_pre` divides SYSCLK down to HCLK (the core/AHB bus clock)
- `apb1_pre`, `apb2_pre`, `apb3_pre` divide HCLK down to their
  respective peripheral bus clock (PCLK1/PCLK2/PCLK3)

The `voltage_range` field sets the internal regulator voltage, which caps how fast every clock in the tree is allowed to run.
There are four ranges:
- **Range 1** allows the highest performance: up to 160 MHz on AHB1/AHB2/AHB3/APB1/APB2/APB3, and PLL outputs up to 208 MHz
(VCO 128–544 MHz, with the actual max depending on which peripherals are attached to the PLL output)
- **Range 2** caps the bus clocks at 110 MHz and PLL outputs at 110 MHz
- **Range 3** caps buses at 55 MHz and PLL outputs at 55 MHz (VCO narrows to 128–330 MHz)
- **Range 4** is the lowest-power option, capping buses at 25 MHz, disallowing PLL use entirely, and even halving the usable HSI16/HSI48 frequency

For more details, check out *RM0456 § 10.5.4* for the general behavior, and *§ 11.4.10* for the specific frequency limits per clock source.

### Peripheral kernel clock muxes (`ClockMuxConfig`)

Many peripherals can take their operating clock from more than one source.

`ClockMuxConfig` holds one selector field per such peripheral (`usart1sel`, `spi1sel`, `i2c1sel`, `rtcsel`, etc.).

`ClockMuxConfig::default()` reproduces the hardware reset values, so a config only needs to override the muxes it cares about:

```rust
let mut mux = ClockMuxConfig::default();
mux.adcdacsel = Adcdacsel::Hsi;   // ADC/DAC clocked from HSI16
mux.rtcsel    = Rtcsel::Lsi;      // RTC clocked from LSI
mux.usart1sel = Usart1sel::Pclk2; // not actually necessary, since PCLK2 is the default for USART1
...
```
