use crate::iomuxc::{DriveStrength, MuxMode, OpenDrainEn, PullKeepEn, PullUpDown, Sion, Speed};
use kernel::common::registers::{register_bitfields, ReadWrite};
use kernel::common::StaticRef;

/// IOMUX SNVS Controller Module
#[repr(C)]
struct IomuxcSnvsRegisters {
    sw_mux_ctl_pad_wakeup: ReadWrite<u32, SW_MUX_CTL_PAD_WAKEUP::Register>,
    sw_mux_ctl_pad_pmic_on_req: ReadWrite<u32, SW_MUX_CTL_PAD_PMIC_ON_REQ::Register>,
    sw_mux_ctl_pad_pmic_stby_req: ReadWrite<u32, SW_MUX_CTL_PAD_PMIC_STBY_REQ::Register>,
    sw_pad_ctl_pad_test_mode: ReadWrite<u32, SW_PAD_CTL_PAD_TEST_MODE::Register>,
    sw_pad_ctl_pad_por_b: ReadWrite<u32, SW_PAD_CTL_PAD_POR_B::Register>,
    sw_pad_ctl_pad_onoff: ReadWrite<u32, SW_PAD_CTL_PAD_ONOFF::Register>,
    sw_pad_ctl_pad_wakeup: ReadWrite<u32, SW_PAD_CTL_PAD_WAKEUP::Register>,
    sw_pad_ctl_pad_pmic_on_req: ReadWrite<u32, SW_PAD_CTL_PAD_PMIC_ON_REQ::Register>,
    sw_pad_ctl_pad_pmic_stby_req: ReadWrite<u32, SW_PAD_CTL_PAD_PMIC_STBY_REQ::Register>,
}

register_bitfields![u32,
    SW_MUX_CTL_PAD_WAKEUP [
        // Software Input On Field
        SION OFFSET(4) NUMBITS(1) [],
        // MUX Mode Select Field
        MUX_MODE OFFSET(0) NUMBITS(3) []
    ],

    SW_MUX_CTL_PAD_PMIC_ON_REQ [
        // Software Input On Field
        SION OFFSET(4) NUMBITS(1) [],
        // MUX Mode Select Field
        MUX_MODE OFFSET(0) NUMBITS(3) []
    ],

    SW_MUX_CTL_PAD_PMIC_STBY_REQ [
        // Software Input On Field
        SION OFFSET(4) NUMBITS(1) [],
        // MUX Mode Select Field
        MUX_MODE OFFSET(0) NUMBITS(3) []
    ],

    SW_PAD_CTL_PAD_TEST_MODE [
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

    SW_PAD_CTL_PAD_POR_B [
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

    SW_PAD_CTL_PAD_ONOFF [
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

    SW_PAD_CTL_PAD_WAKEUP [
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

    SW_PAD_CTL_PAD_PMIC_ON_REQ [
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

    SW_PAD_CTL_PAD_PMIC_STBY_REQ [
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
    ]

];

const IOMUXC_SNVS_BASE: StaticRef<IomuxcSnvsRegisters> =
    unsafe { StaticRef::new(0x400A8000 as *const IomuxcSnvsRegisters) };

pub struct IomuxcSnvs {
    registers: StaticRef<IomuxcSnvsRegisters>,
}

pub static mut IOMUXC_SNVS: IomuxcSnvs = IomuxcSnvs::new();

impl IomuxcSnvs {
    const fn new() -> IomuxcSnvs {
        IomuxcSnvs {
            registers: IOMUXC_SNVS_BASE,
        }
    }

    pub fn is_enabled_sw_mux_ctl_pad_gpio_mode(&self, pin: usize) -> bool {
        match pin {
            0 => self
                .registers
                .sw_mux_ctl_pad_wakeup
                .is_set(SW_MUX_CTL_PAD_WAKEUP::MUX_MODE),
            1 => self
                .registers
                .sw_mux_ctl_pad_pmic_on_req
                .is_set(SW_MUX_CTL_PAD_PMIC_ON_REQ::MUX_MODE),
            2 => self
                .registers
                .sw_mux_ctl_pad_pmic_stby_req
                .is_set(SW_MUX_CTL_PAD_PMIC_STBY_REQ::MUX_MODE),
            _ => false,
        }
    }

    pub fn enable_sw_mux_ctl_pad_gpio(&self, mode: MuxMode, sion: Sion, pin: usize) {
        match pin {
            0 => {
                self.registers.sw_mux_ctl_pad_wakeup.modify(
                    SW_MUX_CTL_PAD_WAKEUP::MUX_MODE.val(mode as u32)
                        + SW_MUX_CTL_PAD_WAKEUP::SION.val(sion as u32),
                );
            }
            1 => {
                self.registers.sw_mux_ctl_pad_pmic_on_req.modify(
                    SW_MUX_CTL_PAD_PMIC_ON_REQ::MUX_MODE.val(mode as u32)
                        + SW_MUX_CTL_PAD_PMIC_ON_REQ::SION.val(sion as u32),
                );
            }
            2 => {
                self.registers.sw_mux_ctl_pad_pmic_stby_req.modify(
                    SW_MUX_CTL_PAD_PMIC_STBY_REQ::MUX_MODE.val(mode as u32)
                        + SW_MUX_CTL_PAD_PMIC_STBY_REQ::SION.val(sion as u32),
                );
            }
            _ => {}
        }
    }

    pub fn disable_sw_mux_ctl_pad_gpio(&self, pin: usize) {
        match pin {
            0 => {
                self.registers.sw_mux_ctl_pad_wakeup.modify(
                    SW_MUX_CTL_PAD_WAKEUP::MUX_MODE::CLEAR + SW_MUX_CTL_PAD_WAKEUP::SION::CLEAR,
                );
            }
            1 => {
                self.registers.sw_mux_ctl_pad_pmic_on_req.modify(
                    SW_MUX_CTL_PAD_PMIC_ON_REQ::MUX_MODE::CLEAR
                        + SW_MUX_CTL_PAD_PMIC_ON_REQ::SION::CLEAR,
                );
            }
            2 => {
                self.registers.sw_mux_ctl_pad_pmic_stby_req.modify(
                    SW_MUX_CTL_PAD_PMIC_STBY_REQ::MUX_MODE::CLEAR
                        + SW_MUX_CTL_PAD_PMIC_STBY_REQ::SION::CLEAR,
                );
            }
            _ => {}
        }
    }

    pub fn configure_sw_pad_ctl_pad_gpio(
        &self,
        pin: usize,
        pus: PullUpDown,
        pke: PullKeepEn,
        ode: OpenDrainEn,
        speed: Speed,
        dse: DriveStrength,
    ) {
        match pin {
            0 => {
                self.registers.sw_pad_ctl_pad_wakeup.modify(
                    SW_PAD_CTL_PAD_WAKEUP::PUS.val(pus as u32)
                        + SW_PAD_CTL_PAD_WAKEUP::PKE.val(pke as u32)
                        + SW_PAD_CTL_PAD_WAKEUP::ODE.val(ode as u32)
                        + SW_PAD_CTL_PAD_WAKEUP::SPEED.val(speed as u32)
                        + SW_PAD_CTL_PAD_WAKEUP::DSE.val(dse as u32),
                );
            }
            1 => {
                self.registers.sw_pad_ctl_pad_pmic_on_req.modify(
                    SW_PAD_CTL_PAD_PMIC_ON_REQ::PUS.val(pus as u32)
                        + SW_PAD_CTL_PAD_PMIC_ON_REQ::PKE.val(pke as u32)
                        + SW_PAD_CTL_PAD_PMIC_ON_REQ::ODE.val(ode as u32)
                        + SW_PAD_CTL_PAD_PMIC_ON_REQ::SPEED.val(speed as u32)
                        + SW_PAD_CTL_PAD_PMIC_ON_REQ::DSE.val(dse as u32),
                );
            }
            2 => {
                self.registers.sw_pad_ctl_pad_pmic_stby_req.modify(
                    SW_PAD_CTL_PAD_PMIC_STBY_REQ::PUS.val(pus as u32)
                        + SW_PAD_CTL_PAD_PMIC_STBY_REQ::PKE.val(pke as u32)
                        + SW_PAD_CTL_PAD_PMIC_STBY_REQ::ODE.val(ode as u32)
                        + SW_PAD_CTL_PAD_PMIC_STBY_REQ::SPEED.val(speed as u32)
                        + SW_PAD_CTL_PAD_PMIC_STBY_REQ::DSE.val(dse as u32),
                );
            }
            _ => {}
        }
    }
}
