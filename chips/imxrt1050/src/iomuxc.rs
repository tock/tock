use kernel::common::registers::{register_bitfields, ReadWrite};
use kernel::common::StaticRef;
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;

/// IOMUX Controller Module
#[repr(C)]
struct IomuxcRegisters {
    sw_mux_ctl_pad_gpio_emc: [ReadWrite<u32, SW_MUX_CTL_PAD_GPIO::Register>; 42],
    sw_mux_ctl_pad_gpio_ad_b0: [ReadWrite<u32, SW_MUX_CTL_PAD_GPIO::Register>; 16],
    sw_mux_ctl_pad_gpio_ad_b1: [ReadWrite<u32, SW_MUX_CTL_PAD_GPIO::Register>; 16],
    sw_mux_ctl_pad_gpio_b0: [ReadWrite<u32, SW_MUX_CTL_PAD_GPIO::Register>; 16],
    sw_mux_ctl_pad_gpio_b1: [ReadWrite<u32, SW_MUX_CTL_PAD_GPIO::Register>; 16],
    sw_mux_ctl_pad_gpio_sd_b0: [ReadWrite<u32, SW_MUX_CTL_PAD_GPIO::Register>; 6],
    sw_mux_ctl_pad_gpio_sd_b1: [ReadWrite<u32, SW_MUX_CTL_PAD_GPIO::Register>; 12],

    sw_pad_ctl_pad_gpio_emc: [ReadWrite<u32, SW_PAD_CTL_PAD_GPIO::Register>; 42],
    sw_pad_ctl_pad_gpio_ad_b0: [ReadWrite<u32, SW_PAD_CTL_PAD_GPIO::Register>; 16],
    sw_pad_ctl_pad_gpio_ad_b1: [ReadWrite<u32, SW_PAD_CTL_PAD_GPIO::Register>; 16],
    sw_pad_ctl_pad_gpio_b0: [ReadWrite<u32, SW_PAD_CTL_PAD_GPIO::Register>; 16],
    sw_pad_ctl_pad_gpio_b1: [ReadWrite<u32, SW_PAD_CTL_PAD_GPIO::Register>; 16],
    sw_pad_ctl_pad_gpio_sd_b0: [ReadWrite<u32, SW_PAD_CTL_PAD_GPIO::Register>; 6],
    sw_pad_ctl_pad_gpio_sd_b1: [ReadWrite<u32, SW_PAD_CTL_PAD_GPIO::Register>; 12],

    anatop_usb_otg1_id_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    anatop_usb_otg2_id_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    
    ccm_pmic_ready_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,

    csi_data0_x_select_input: [ReadWrite<u32, DAISY_SELECT_INPUT::Register>; 8],
    csi_hsync_select_input: ReadWrite<u32, DAISY_2BIT_SELECT_INPUT::Register>,
    csi_pixclk_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    csi_vsync_select_input: ReadWrite<u32, DAISY_2BIT_SELECT_INPUT::Register>,

    enet_ipg_clk_rmii_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    enet_mdio_select_input: ReadWrite<u32, DAISY_2BIT_SELECT_INPUT::Register>,
    enet0_rxdata_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    enet1_rxdata_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    enet_rxen_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    enet_rxerr_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    enet0_timer_select_input: ReadWrite<u32, DAISY_2BIT_SELECT_INPUT::Register>,
    enet_txclk_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,

    flexcan1_rx_select_input: ReadWrite<u32, DAISY_2BIT_SELECT_INPUT::Register>,
    flexcan2_rx_select_input: ReadWrite<u32, DAISY_2BIT_SELECT_INPUT::Register>,

    flexpwm1_pwma3_select_input: ReadWrite<u32, DAISY_3BIT_SELECT_INPUT::Register>,
    flexpwm1_pwma0_2_select_input: [ReadWrite<u32, DAISY_SELECT_INPUT::Register>; 3],
    flexpwm1_pwmb3_select_input: ReadWrite<u32, DAISY_3BIT_SELECT_INPUT::Register>,
    flexpwm1_pwmb0_2_select_input: [ReadWrite<u32, DAISY_SELECT_INPUT::Register>; 3],

    flexpwm2_pwma3_select_input: ReadWrite<u32, DAISY_3BIT_SELECT_INPUT::Register>,
    flexpwm2_pwma0_2_select_input: [ReadWrite<u32, DAISY_SELECT_INPUT::Register>; 3],
    flexpwm2_pwmb3_select_input: ReadWrite<u32, DAISY_3BIT_SELECT_INPUT::Register>,
    flexpwm2_pwmb0_2_select_input: [ReadWrite<u32, DAISY_SELECT_INPUT::Register>; 3],

    flexpwm4_pwma0_3_select_input: [ReadWrite<u32, DAISY_SELECT_INPUT::Register>; 4],

    flexspi_a_dqs_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    flexspi_a_data_x_select_input: [ReadWrite<u32, DAISY_SELECT_INPUT::Register>; 4],
    flexspi_b_data_x_select_input: [ReadWrite<u32, DAISY_SELECT_INPUT::Register>; 4],
    flexspi_a_sck_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,

    lpi2c1_scl_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    lpi2c1_sda_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,

    lpi2c2_scl_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    lpi2c2_sda_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,

    lpi2c3_scl_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    lpi2c3_sda_select_input: ReadWrite<u32, DAISY_2BIT_SELECT_INPUT::Register>,

    lpi2c4_scl_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    lpi2c4_sda_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,

    lpspi1_pcs0_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    lpspi1_sck_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    lpspi1_sdi_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    lpspi1_sdo_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,

    lpspi2_pcs0_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    lpspi2_sck_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    lpspi2_sdi_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    lpspi2_sdo_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,

    lpspi3_pcs0_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    lpspi3_sck_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    lpspi3_sdi_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    lpspi3_sdo_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,

    lpspi4_pcs0_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    lpspi4_sck_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    lpspi4_sdi_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    lpspi4_sdo_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,

    lpuart2_rx_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    lpuart2_tx_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,

    lpuart3_cts_b_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    lpuart3_rx_select_input: ReadWrite<u32, DAISY_2BIT_SELECT_INPUT::Register>,
    lpuart3_tx_select_input: ReadWrite<u32, DAISY_2BIT_SELECT_INPUT::Register>,

    lpuart4_rx_select_input: ReadWrite<u32, DAISY_2BIT_SELECT_INPUT::Register>,
    lpuart4_tx_select_input: ReadWrite<u32, DAISY_2BIT_SELECT_INPUT::Register>,

    lpuart5_rx_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    lpuart5_tx_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,

    lpuart6_rx_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    lpuart6_tx_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,

    lpuart7_rx_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    lpuart7_tx_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,

    lpuart8_rx_select_input: ReadWrite<u32, DAISY_2BIT_SELECT_INPUT::Register>,
    lpuart8_tx_select_input: ReadWrite<u32, DAISY_2BIT_SELECT_INPUT::Register>,

    nmi_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,

    qtimer2_timer_x_select_input: [ReadWrite<u32, DAISY_SELECT_INPUT::Register>; 4],
    qtimer3_timer_x_select_input: [ReadWrite<u32, DAISY_2BIT_SELECT_INPUT::Register>; 4],

    sai1_mclk2_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    sai1_rx_bclk_select_input: ReadWrite<u32, DAISY_2BIT_SELECT_INPUT::Register>,
    sai1_rx_data0_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    sai1_rx_data1_3_select_input: [ReadWrite<u32, DAISY_2BIT_SELECT_INPUT::Register>; 3],
    sai1_rx_sync_select_input: ReadWrite<u32, DAISY_2BIT_SELECT_INPUT::Register>,
    sai1_tx_bclk_select_input: ReadWrite<u32, DAISY_2BIT_SELECT_INPUT::Register>,
    sai1_tx_sync_select_input: ReadWrite<u32, DAISY_2BIT_SELECT_INPUT::Register>,

    sai2_mclk2_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    sai2_rx_bclk_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    sai2_rx_data0_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    sai2_rx_sync_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    sai2_tx_bclk_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    sai2_tx_sync_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,

    spdif_in_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,

    usb_otg2_oc_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    usb_otg1_oc_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,

    usdhc1_cd_b_select_input: ReadWrite<u32, DAISY_2BIT_SELECT_INPUT::Register>,
    usdhc1_wp_select_input: ReadWrite<u32, DAISY_2BIT_SELECT_INPUT::Register>,
    usdhc2_clk_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    usdhc2_cd_b_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    usdhc2_cmd_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    usdhc2_data_x_select_input: [ReadWrite<u32, DAISY_SELECT_INPUT::Register>; 8],
    usdhc2_wp_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,

    xbar_inout02_09_select_input: [ReadWrite<u32, DAISY_SELECT_INPUT::Register>; 8],
    xbar_inout17_select_input: ReadWrite<u32, DAISY_2BIT_SELECT_INPUT::Register>,
    xbar_inout18_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    xbar_inout20_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    xbar_inout22_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    xbar_inout23_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    xbar_inout24_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    xbar_inout14_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    xbar_inout15_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    xbar_inout16_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    xbar_inout25_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    xbar_inout19_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>,
    xbar_inout21_select_input: ReadWrite<u32, DAISY_SELECT_INPUT::Register>
}

register_bitfields![u32,
    SW_MUX_CTL_PAD_GPIO [
        // Software Input On Field
        SION OFFSET(4) NUMBITS(1) [],

        // MUX Mode Select Field
        MUX_MODE OFFSET(0) NUMBITS(3) []
    ],

    SW_PAD_CTL_PAD_GPIO [
        // Hyst. Enable Field
        HYS OFFSET(16) NUMBITS(1) [],

        // Pull Up / Down Config Field
        PUS OFFSET(14) NUMBITS(2) [],

        // Pull / Keep Select Field
        PUE OFFSET(13) NUMBITS(1) [],

        // Pull / Keep enable field
        PKE OFFSET(12) NUMBITS(1) [],

        // Open drain enable field
        ODE OFFSET(11) NUMBITS(1) [],

    	// Speed
    	SPEED OFFSET(6) NUMBITS(2) [],

    	// Drive Strength Field
    	DSE OFFSET(3) NUMBITS(3) [],

    	// Slew Rate Field
    	SRE OFFSET(0) NUMBITS(1) []
    ],

    DAISY_SELECT_INPUT [
        //Selecting Pads Involved in Daisy Chain.
        DAISY OFFSET(0) NUMBITS(1) []
    ],

    DAISY_2BIT_SELECT_INPUT [
        //Selecting Pads Involved in Daisy Chain.
        DAISY OFFSET(0) NUMBITS(2) []
    ],

    DAISY_3BIT_SELECT_INPUT [
        //Selecting Pads Involved in Daisy Chain.
        DAISY OFFSET(0) NUMBITS(3) []
    ]
];

const IOMUXC_BASE: StaticRef<IomuxcRegisters> =
    unsafe { StaticRef::new(0x401F8014 as *const IomuxcRegisters) };

pub struct Iomuxc {
    registers: StaticRef<IomuxcRegisters>,
}

pub static mut IOMUXC: Iomuxc = Iomuxc::new();

#[repr(u32)]
pub enum PadId {
    EMC = 0b000,
    AdB0 = 0b001,
    AdB1 = 0b010,
    B0 = 0b011,
    B1 = 0b100,
    SdB0 = 0b101,
    SdB1 = 0b110
}

#[repr(u32)]
pub enum Sion {
    Enabled = 1,
    Disabled = 0
}

// Alternative Modes for Mux Mode Select Field
// Each mode is specific for the iomux pad.
enum_from_primitive! {
    #[repr(u32)]
    pub enum MuxMode {
       ALT0 = 0b000, // Tipically used for semc, jtag_mux
       ALT1 = 0b001, // Tipically used for gpt, lpi2c, flexpwm
       ALT2 = 0b010, // Tipically used for lpuart, lpspi, flexpwm
       ALT3 = 0b011, // Tipically used for xbar, usdhc
       ALT4 = 0b100, // Tipically used for flexio, qtimer, gpt
       ALT5 = 0b101, // Tipically used for gpio mode
       ALT6 = 0b110, // Rarely used. In EMC_18, used for snvs_hp
       ALT7 = 0b111, // 
    }
}

#[repr(u32)]
pub enum HystEn {
    Hys0HysteresisDisabled = 0b0,
    Hys1HysteresisEnabled = 0b1
}

/// GPIO pin internal pull-up and pull-down
#[repr(u32)]
pub enum PullUpDown {
    Pus0_100kOhmPullDown = 0b00,
    Pus1_47kOhmPullUp    = 0b01,
    Pus2_100kOhmPullUp   = 0b10,
    Pus3_22kOhmPullUp    = 0b11
}

#[repr(u32)]
pub enum PullKeepSel {
    Pue0Keeper = 0b0,
    Pue1Pull   = 0b1
}

#[repr(u32)]
pub enum PullKeepEn {
    Pke0PullKeeperDisabled = 0b0,
    Pke1PullKeeperEnabled  = 0b1
}

#[repr(u32)]
pub enum OpenDrainEn {
    Ode0OpenDrainDisabled = 0b0,
    Ode1OpenDrainEnabled = 0b1
}

#[repr(u32)]
pub enum Speed {
    Low = 0b00,     // 50MHz
    Medium1 = 0b01, // 100MHz - 150MHz
    Medium2 = 0b10, // 100MHz - 150MHz
    Maximum = 0b11  // 150MHz - 200MHz
}

#[repr(u32)]
pub enum DriveStrength {
    DSE0 = 0b000,   // HI-Z
    DSE1 = 0b001,   // Dual/Single voltage: 262/260 Ohm @ 1.8V, 247/157 Ohm @ 3.3V
    DSE2 = 0b010,   // Dual/Single voltage: 134/130 Ohm @ 1.8V, 126/78 Ohm @ 3.3V
    DSE3 = 0b011,   // Dual/Single voltage: 88/88 Ohm @ 1.8V, 84/53 Ohm @ 3.3V
    DSE4 = 0b100,   // Dual/Single voltage: 62/65 Ohm @ 1.8V, 57/39 Ohm @ 3.3V
    DSE5 = 0b101,   // Dual/Single voltage: 51/52 Ohm @ 1.8V, 47/32 Ohm @ 3.3V
    DSE6 = 0b110,   // Dual/Single voltage: 43/43 Ohm @ 1.8V, 40/26 Ohm @ 3.3V
    DSE7 = 0b111,   // Dual/Single voltage: 37/37 Ohm @ 1.8V, 34/23 Ohm @ 3.3V
}

#[repr(u32)]
pub enum SlewRate {
    Sre0SlowSlewRate = 0b0,
    Sre1FastSlewRate = 0b1
}

impl Iomuxc {
    const fn new() -> Iomuxc {
        Iomuxc {
            registers: IOMUXC_BASE,
        }
    }

    pub fn is_enabled_sw_mux_ctl_pad_gpio_mode(&self, pad: PadId, pin: usize) -> bool {
        match pad {
            PadId::EMC => {
                self.registers.sw_mux_ctl_pad_gpio_emc[pin].is_set(SW_MUX_CTL_PAD_GPIO::MUX_MODE)
            },
            PadId::AdB0 => {
                self.registers.sw_mux_ctl_pad_gpio_ad_b0[pin].is_set(SW_MUX_CTL_PAD_GPIO::MUX_MODE)
            },
            PadId::AdB1 => {
                self.registers.sw_mux_ctl_pad_gpio_ad_b1[pin].is_set(SW_MUX_CTL_PAD_GPIO::MUX_MODE)
            },
            PadId::B0 => {
                self.registers.sw_mux_ctl_pad_gpio_b0[pin].is_set(SW_MUX_CTL_PAD_GPIO::MUX_MODE)
            },
            PadId::B1 => {
                self.registers.sw_mux_ctl_pad_gpio_b1[pin].is_set(SW_MUX_CTL_PAD_GPIO::MUX_MODE)
            },
            PadId::SdB0 => {
                self.registers.sw_mux_ctl_pad_gpio_sd_b0[pin].is_set(SW_MUX_CTL_PAD_GPIO::MUX_MODE)
            },
            PadId::SdB1 => {
                self.registers.sw_mux_ctl_pad_gpio_sd_b1[pin].is_set(SW_MUX_CTL_PAD_GPIO::MUX_MODE)
            }
        }
    }

    pub fn enable_sw_mux_ctl_pad_gpio(&self, pad: PadId, mode: MuxMode, sion: Sion, pin: usize) {
        match pad {
            PadId::EMC => {
                self.registers.sw_mux_ctl_pad_gpio_emc[pin].modify(SW_MUX_CTL_PAD_GPIO::MUX_MODE.val(mode as u32)
                                                                + SW_MUX_CTL_PAD_GPIO::SION.val(sion as u32));
            },
            PadId::AdB0 => {
                self.registers.sw_mux_ctl_pad_gpio_ad_b0[pin].modify(SW_MUX_CTL_PAD_GPIO::MUX_MODE.val(mode as u32)
                                                                + SW_MUX_CTL_PAD_GPIO::SION.val(sion as u32));
            },
            PadId::AdB1 => {
                self.registers.sw_mux_ctl_pad_gpio_ad_b1[pin].modify(SW_MUX_CTL_PAD_GPIO::MUX_MODE.val(mode as u32)
                                                                + SW_MUX_CTL_PAD_GPIO::SION.val(sion as u32));
            },
            PadId::B0 => {
                self.registers.sw_mux_ctl_pad_gpio_b0[pin].modify(SW_MUX_CTL_PAD_GPIO::MUX_MODE.val(mode as u32)
                                                                + SW_MUX_CTL_PAD_GPIO::SION.val(sion as u32));
            },
            PadId::B1 => {
                self.registers.sw_mux_ctl_pad_gpio_b1[pin].modify(SW_MUX_CTL_PAD_GPIO::MUX_MODE.val(mode as u32)
                                                                + SW_MUX_CTL_PAD_GPIO::SION.val(sion as u32));
            },
            PadId::SdB0 => {
                self.registers.sw_mux_ctl_pad_gpio_sd_b0[pin].modify(SW_MUX_CTL_PAD_GPIO::MUX_MODE.val(mode as u32)
                                                                + SW_MUX_CTL_PAD_GPIO::SION.val(sion as u32));
            },
            PadId::SdB1 => {
                self.registers.sw_mux_ctl_pad_gpio_sd_b1[pin].modify(SW_MUX_CTL_PAD_GPIO::MUX_MODE.val(mode as u32)
                                                                + SW_MUX_CTL_PAD_GPIO::SION.val(sion as u32));
            }
        }
    }

    pub fn disable_sw_mux_ctl_pad_gpio(&self, pad: PadId, pin: usize) {
        match pad {
            PadId::EMC => {
                self.registers.sw_mux_ctl_pad_gpio_emc[pin].modify(SW_MUX_CTL_PAD_GPIO::MUX_MODE::CLEAR
                                                                + SW_MUX_CTL_PAD_GPIO::SION::CLEAR);
            },
            PadId::AdB0 => {
                self.registers.sw_mux_ctl_pad_gpio_ad_b0[pin].modify(SW_MUX_CTL_PAD_GPIO::MUX_MODE::CLEAR
                                                                + SW_MUX_CTL_PAD_GPIO::SION::CLEAR);
            },
            PadId::AdB1 => {
                self.registers.sw_mux_ctl_pad_gpio_ad_b1[pin].modify(SW_MUX_CTL_PAD_GPIO::MUX_MODE::CLEAR
                                                                + SW_MUX_CTL_PAD_GPIO::SION::CLEAR);
            },
            PadId::B0 => {
                self.registers.sw_mux_ctl_pad_gpio_b0[pin].modify(SW_MUX_CTL_PAD_GPIO::MUX_MODE::CLEAR
                                                                + SW_MUX_CTL_PAD_GPIO::SION::CLEAR);
            },
            PadId::B1 => {
                self.registers.sw_mux_ctl_pad_gpio_b1[pin].modify(SW_MUX_CTL_PAD_GPIO::MUX_MODE::CLEAR
                                                                + SW_MUX_CTL_PAD_GPIO::SION::CLEAR);
            },
            PadId::SdB0 => {
                self.registers.sw_mux_ctl_pad_gpio_sd_b0[pin].modify(SW_MUX_CTL_PAD_GPIO::MUX_MODE::CLEAR
                                                                + SW_MUX_CTL_PAD_GPIO::SION::CLEAR);
            },
            PadId::SdB1 => {
                self.registers.sw_mux_ctl_pad_gpio_sd_b1[pin].modify(SW_MUX_CTL_PAD_GPIO::MUX_MODE::CLEAR
                                                                + SW_MUX_CTL_PAD_GPIO::SION::CLEAR);
            }
        }
    }

    pub fn configure_sw_pad_ctl_pad_gpio(&self, pad: PadId, pin: usize, pus: PullUpDown, 
            pke: PullKeepEn, ode: OpenDrainEn, speed: Speed, dse: DriveStrength) {
        match pad {
            PadId::EMC => {
                self.registers.sw_pad_ctl_pad_gpio_emc[pin].modify(SW_PAD_CTL_PAD_GPIO::PUS.val(pus as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::PKE.val(pke as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::ODE.val(ode as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::SPEED.val(speed as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::DSE.val(dse as u32));
            },
            PadId::AdB0 => {
                self.registers.sw_pad_ctl_pad_gpio_ad_b0[pin].modify(SW_PAD_CTL_PAD_GPIO::PUS.val(pus as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::PKE.val(pke as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::ODE.val(ode as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::SPEED.val(speed as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::DSE.val(dse as u32));
            },
            PadId::AdB1 => {
                self.registers.sw_pad_ctl_pad_gpio_ad_b1[pin].modify(SW_PAD_CTL_PAD_GPIO::PUS.val(pus as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::PKE.val(pke as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::ODE.val(ode as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::SPEED.val(speed as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::DSE.val(dse as u32));
            },
            PadId::B0 => {
                self.registers.sw_pad_ctl_pad_gpio_b0[pin].modify(SW_PAD_CTL_PAD_GPIO::PUS.val(pus as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::PKE.val(pke as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::ODE.val(ode as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::SPEED.val(speed as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::DSE.val(dse as u32));
            },
            PadId::B1 => {
                self.registers.sw_pad_ctl_pad_gpio_b1[pin].modify(SW_PAD_CTL_PAD_GPIO::PUS.val(pus as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::PKE.val(pke as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::ODE.val(ode as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::SPEED.val(speed as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::DSE.val(dse as u32));
            },
            PadId::SdB0 => {
                self.registers.sw_pad_ctl_pad_gpio_sd_b0[pin].modify(SW_PAD_CTL_PAD_GPIO::PUS.val(pus as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::PKE.val(pke as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::ODE.val(ode as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::SPEED.val(speed as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::DSE.val(dse as u32));
            },
            PadId::SdB1 => {
                self.registers.sw_pad_ctl_pad_gpio_sd_b1[pin].modify(SW_PAD_CTL_PAD_GPIO::PUS.val(pus as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::PKE.val(pke as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::ODE.val(ode as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::SPEED.val(speed as u32)
                                                                + SW_PAD_CTL_PAD_GPIO::DSE.val(dse as u32));
            }
        }
    }
    // pub fn enable_lpi2c1_scl_16(&self) {
    //     self.registers.sw_pad_ctl_pad_gpio_ad_b1_00.modify(SW_PAD_CTL_PAD_GPIO_AD_B1_00::PUS.val(0b11 as u32));
    //     self.registers.sw_pad_ctl_pad_gpio_ad_b1_00.modify(SW_PAD_CTL_PAD_GPIO_AD_B1_00::PKE::SET);
    //     self.registers.sw_pad_ctl_pad_gpio_ad_b1_00.modify(SW_PAD_CTL_PAD_GPIO_AD_B1_00::ODE::SET);
    //     self.registers.sw_pad_ctl_pad_gpio_ad_b1_00.modify(SW_PAD_CTL_PAD_GPIO_AD_B1_00::SPEED.val(0b10 as u32));
    //     self.registers.sw_pad_ctl_pad_gpio_ad_b1_00.modify(SW_PAD_CTL_PAD_GPIO_AD_B1_00::DSE.val(0b110 as u32));
    // }

    // pub fn enable_lpi2c1_sda_17(&self) {
    //     self.registers.sw_pad_ctl_pad_gpio_ad_b1_01.modify(SW_PAD_CTL_PAD_GPIO_AD_B1_01::PUS.val(0b11 as u32));
    //     self.registers.sw_pad_ctl_pad_gpio_ad_b1_01.modify(SW_PAD_CTL_PAD_GPIO_AD_B1_01::PKE::SET);
    //     self.registers.sw_pad_ctl_pad_gpio_ad_b1_01.modify(SW_PAD_CTL_PAD_GPIO_AD_B1_01::ODE::SET);
    //     self.registers.sw_pad_ctl_pad_gpio_ad_b1_01.modify(SW_PAD_CTL_PAD_GPIO_AD_B1_01::SPEED.val(0b10 as u32));
    //     self.registers.sw_pad_ctl_pad_gpio_ad_b1_01.modify(SW_PAD_CTL_PAD_GPIO_AD_B1_01::DSE.val(0b110 as u32));
    // }

    // // Enable GPIO1 on pin AD_B0_09
    // pub fn enable_gpio1_09(&self) {
    //     self.registers.sw_pad_ctl_pad_gpio_ad_b0_09.modify(SW_PAD_CTL_PAD_GPIO_AD_B0_09::DSE.val(0b110 as u32));
    //     self.registers.sw_pad_ctl_pad_gpio_ad_b0_09.modify(SW_PAD_CTL_PAD_GPIO_AD_B0_09::SPEED.val(0b10 as u32));
    //     self.registers.sw_pad_ctl_pad_gpio_ad_b0_09.modify(SW_PAD_CTL_PAD_GPIO_AD_B0_09::PKE::SET);   
    // }

    // pub fn set_pin_config_lpuart1(&self) {
    //     self.registers.sw_pad_ctl_pad_gpio_ad_b0_12.modify(SW_PAD_CTL_PAD_GPIO_AD_B0_12::DSE.val(0b110 as u32));
    //     self.registers.sw_pad_ctl_pad_gpio_ad_b0_12.modify(SW_PAD_CTL_PAD_GPIO_AD_B0_12::SPEED.val(0b10 as u32));
    //     self.registers.sw_pad_ctl_pad_gpio_ad_b0_12.modify(SW_PAD_CTL_PAD_GPIO_AD_B0_12::PKE::SET); 

    //     self.registers.sw_pad_ctl_pad_gpio_ad_b0_13.modify(SW_PAD_CTL_PAD_GPIO_AD_B0_13::DSE.val(0b110 as u32));
    //     self.registers.sw_pad_ctl_pad_gpio_ad_b0_13.modify(SW_PAD_CTL_PAD_GPIO_AD_B0_13::SPEED.val(0b10 as u32));
    //     self.registers.sw_pad_ctl_pad_gpio_ad_b0_13.modify(SW_PAD_CTL_PAD_GPIO_AD_B0_13::PKE::SET); 
    // }

    // LPI2C1_SCL_SELECT_INPUT 
    pub fn is_enabled_lpi2c_scl_select_input(&self) ->bool{
        self.registers.lpi2c1_scl_select_input.is_set(DAISY_SELECT_INPUT::DAISY) 
    }

    pub fn enable_lpi2c_scl_select_input(&self) {
        self.registers.lpi2c1_scl_select_input.modify(DAISY_SELECT_INPUT::DAISY::SET) 
    }

    pub fn disable_lpi2c_scl_select_input(&self) {
        self.registers.lpi2c1_scl_select_input.modify(DAISY_SELECT_INPUT::DAISY::CLEAR);
    }

    // LPI2C1_SDA_SELECT_INPUT 
    pub fn is_enabled_lpi2c_sda_select_input(&self) -> bool {
        self.registers.lpi2c1_sda_select_input.is_set(DAISY_SELECT_INPUT::DAISY) 
    }

    pub fn enable_lpi2c_sda_select_input(&self) {
        self.registers.lpi2c1_sda_select_input.modify(DAISY_SELECT_INPUT::DAISY::SET) 
    }

    pub fn disable_lpi2c_sda_select_input(&self) {
        self.registers.lpi2c1_sda_select_input.modify(DAISY_SELECT_INPUT::DAISY::CLEAR);
    }
}