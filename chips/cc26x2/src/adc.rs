use adi;
use adi::AuxAdi4Registers;
use aux;
use kernel::common::StaticRef;
use rom;

use memory_map::AUX_ADI4_BASE;

// Redeclaration of bitfield enums s.t. client only needs adc.rs dependency
#[allow(non_camel_case_types)]
pub enum SAMPLE_CYCLE {
    _2p7_us,  // 2.7  uS
    _5p3_us,  // 5.3  uS
    _10p6_us, // 10.6 uS
    _21p3_us, // 21.3 uS
    _42p6_us, // 42.6 uS
    _85p3_us, // 85.3.uS
    _170_us,  // 170  uS
    _341_us,  // 341  uS
    _682_us,  // 682  uS
    _1p37_us, // 1.37 mS
    _2p73_us, // 2.73 mS
    _5p46_ms, // 5.46 mS
    _10p9_ms, // 10.9 mS
}

pub enum SOURCE {
    Fixed4P5V,
    NominalVdds,
}

const AUX_ADI4: StaticRef<AuxAdi4Registers> =
    unsafe { StaticRef::new(AUX_ADI4_BASE as *const AuxAdi4Registers) };

pub struct Adc {
    aux_adi4: StaticRef<AuxAdi4Registers>,
}

pub static mut ADC: Adc = Adc::new();

impl Adc {
    const fn new() -> Adc {
        Adc { aux_adi4: AUX_ADI4 }
    }

    pub fn set_input(&self, pin: usize) {
        let hapi_param;
        // extracted from code
        // doesn't match table 13-2 which is odd
        match pin {
            30 => hapi_param = rom::ADC_COMPB_IN::AUXIO0,
            29 => hapi_param = rom::ADC_COMPB_IN::AUXIO1,
            28 => hapi_param = rom::ADC_COMPB_IN::AUXIO2,
            27 => hapi_param = rom::ADC_COMPB_IN::AUXIO3,
            26 => hapi_param = rom::ADC_COMPB_IN::AUXIO4,
            25 => hapi_param = rom::ADC_COMPB_IN::AUXIO5,
            24 => hapi_param = rom::ADC_COMPB_IN::AUXIO6,
            23 => hapi_param = rom::ADC_COMPB_IN::AUXIO7,
            _ => {
                return;
            }
        }
        unsafe { (rom::HAPI.select_adc_comp_b_input)(hapi_param) };
    }

    pub fn flush_fifo(&self) {
        aux::anaif::REG
            .adc_ctl
            .write(aux::anaif::AdcCtl::CMD::FlushFifo);
        aux::anaif::REG
            .adc_ctl
            .write(aux::anaif::AdcCtl::CMD::Enable);
    }

    pub fn has_data(&self) -> bool {
        aux::anaif::REG
            .adc_fifo_status
            .read(aux::anaif::AdcFifoStatus::EMPTY)
            == 0
    }

    // Returns 12 bit value from FIFO
    pub fn pop_fifo(&self) -> u16 {
        aux::anaif::REG.adc_fifo.read(aux::anaif::AdcFifo::DATA) as u16
    }

    pub fn configure(&self, source: SOURCE, sample_time: SAMPLE_CYCLE) {
        // Enable ADC reference
        let source_value;
        match source {
            SOURCE::Fixed4P5V => source_value = adi::Reference0::SRC::FIXED_4P5V,
            SOURCE::NominalVdds => source_value = adi::Reference0::SRC::NOMINAL_VDDS,
        }

        self.aux_adi4
            .reference0
            .write(source_value + adi::Reference0::EN::SET);

        // Enable ADC Clock
        let adc_clk_ctl = &aux::sysif::REGISTERS.adc_clk_ctl;
        adc_clk_ctl.req().write(aux::sysif::Req::CLOCK::Enable);

        while !adc_clk_ctl
            .ack()
            .matches_all(aux::sysif::Ack::CLOCK::Enabled)
        {}

        // Enable the ADC data interface
        // assume manual for now
        aux::anaif::REG
            .adc_ctl
            .write(aux::anaif::AdcCtl::START_SRC::NO_EVENT + aux::anaif::AdcCtl::CMD::Enable);

        // Notes on how to do it with special events
        // GPT trigger: Configure event routing via MCU_EV to the AUX domain
        // HWREG(EVENT_BASE + EVENT_O_AUXSEL0) = trigger;
        // HWREG(AUX_ANAIF_BASE + AUX_ANAIF_O_ADCCTL) = AUX_ANAIF_ADCCTL_START_SRC_MCU_EV | AUX_ANAIF_ADCCTL_CMD_EN;

        let sample_time_value;
        match sample_time {
            SAMPLE_CYCLE::_2p7_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_2P7_US,
            SAMPLE_CYCLE::_5p3_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_5P3_US,
            SAMPLE_CYCLE::_10p6_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_10P6_US,
            SAMPLE_CYCLE::_21p3_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_21P3_US,
            SAMPLE_CYCLE::_42p6_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_42P6_US,
            SAMPLE_CYCLE::_85p3_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_85P3_US,
            SAMPLE_CYCLE::_170_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_170_US,
            SAMPLE_CYCLE::_341_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_341_US,
            SAMPLE_CYCLE::_682_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_682_US,
            SAMPLE_CYCLE::_1p37_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_1P37_MS,
            SAMPLE_CYCLE::_2p73_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_2P73_MS,
            SAMPLE_CYCLE::_5p46_ms => sample_time_value = adi::Control0::SAMPLE_CYCLE::_5P46_US,
            SAMPLE_CYCLE::_10p9_ms => sample_time_value = adi::Control0::SAMPLE_CYCLE::_10P9_US,
        }

        self.aux_adi4.control0.write(adi::Control0::RESET_N::CLEAR);

        self.aux_adi4.control0.write(
            sample_time_value
                + adi::Control0::SAMPLE_MODE::SYNC
                + adi::Control0::RESET_N::SET
                + adi::Control0::EN::SET,
        );
    }

    // todo: recurring event mode
    pub fn single_shot(&self) {
        //unsafe { *(0x400C901c as *mut usize) = 0b1 };
        aux::anaif::REG
            .adc_trigger
            .write(aux::anaif::AdcTrigger::START::SET);
    }
}
