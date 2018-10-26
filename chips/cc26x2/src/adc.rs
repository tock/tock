use adi;
use adi::AuxAdi4Registers;
use aux;
use cortexm4::nvic;
use enum_primitive::cast::FromPrimitive;
use kernel::common::cells::OptionalCell;
use kernel::common::StaticRef;
use peripheral_interrupts;
use rom;

use memory_map::AUX_ADI4_BASE;

// Redeclaration of bitfield enums s.t. client only needs adc.rs dependency
#[allow(non_camel_case_types)]
pub enum SampleCycle {
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

enum_from_primitive!{
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Input {
    Auxio0 = 0, // pin 30
    Auxio1 = 1, // pin 29
    Auxio2 = 2, // pin 28
    Auxio3 = 3, // pin 27
    Auxio4 = 4, // pin 26
    Auxio5 = 5, // pin 25
    Auxio6 = 6, // pin 24
    Auxio7 = 7, // pin 23
}
}

#[derive(Clone, Copy)]
pub enum Source {
    Fixed4P5V,
    NominalVdds,
}

const ADC_BITS: usize = 12;
const CC26X_MAX_CHANNELS: usize = 8;

const AUX_ADI4: StaticRef<AuxAdi4Registers> =
    unsafe { StaticRef::new(AUX_ADI4_BASE as *const AuxAdi4Registers) };

const AUX_ADI_NVIQ: nvic::Nvic =
    unsafe { nvic::Nvic::new(peripheral_interrupts::NVIC_IRQ::AUX_ADC as u32) };

pub static mut ADC: Adc = Adc::new(&AUX_ADI_NVIQ);

pub struct Channel {
    aux_input: Input,
    client: OptionalCell<&'static hil::adc::Client>,
}

// giving each Channel it's own client, because in theory, ADC driver could support this
impl Channel {
    const fn new(aux_input: Input) -> Channel {
        Channel {
            aux_input,
            client: OptionalCell::empty(),
        }
    }
}

pub struct Adc {
    aux_adi4: StaticRef<AuxAdi4Registers>,
    nvic: &'static nvic::Nvic,
    voltage_setting: OptionalCell<Source>,
    pub nominal_voltage: Option<usize>,
    channel: [Channel; CC26X_MAX_CHANNELS],
    single_shot_channel: OptionalCell<usize>,
}

impl Adc {
    const fn new(nvic: &'static nvic::Nvic) -> Adc {
        Adc {
            aux_adi4: AUX_ADI4,
            nvic,
            voltage_setting: OptionalCell::empty(),
            nominal_voltage: None,
            channel: [
                Channel::new(Input::Auxio0),
                Channel::new(Input::Auxio1),
                Channel::new(Input::Auxio2),
                Channel::new(Input::Auxio3),
                Channel::new(Input::Auxio4),
                Channel::new(Input::Auxio5),
                Channel::new(Input::Auxio6),
                Channel::new(Input::Auxio7),
            ],
            single_shot_channel: OptionalCell::empty(),
        }
    }

    pub fn set_input(&self, pin: Input) {
        let hapi_param;
        // extracted from code
        // doesn't match table 13-2 which is odd
        match pin {
            Input::Auxio0 => hapi_param = rom::ADC_COMPB_IN::AUXIO0,
            Input::Auxio1 => hapi_param = rom::ADC_COMPB_IN::AUXIO1,
            Input::Auxio2 => hapi_param = rom::ADC_COMPB_IN::AUXIO2,
            Input::Auxio3 => hapi_param = rom::ADC_COMPB_IN::AUXIO3,
            Input::Auxio4 => hapi_param = rom::ADC_COMPB_IN::AUXIO4,
            Input::Auxio5 => hapi_param = rom::ADC_COMPB_IN::AUXIO5,
            Input::Auxio6 => hapi_param = rom::ADC_COMPB_IN::AUXIO6,
            Input::Auxio7 => hapi_param = rom::ADC_COMPB_IN::AUXIO7,
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

    pub fn configure(&self, source: Source, sample_time: SampleCycle) {
        // Enable ADC reference
        let source_value;
        match source {
            Source::Fixed4P5V => source_value = adi::Reference0::SRC::FIXED_4P5V,
            Source::NominalVdds => source_value = adi::Reference0::SRC::NOMINAL_VDDS,
        }

        self.voltage_setting.set(source);

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
        aux::anaif::REG
            .adc_ctl
            .write(aux::anaif::AdcCtl::START_SRC::NO_EVENT + aux::anaif::AdcCtl::CMD::Enable);

        let sample_time_value;
        match sample_time {
            SampleCycle::_2p7_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_2P7_US,
            SampleCycle::_5p3_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_5P3_US,
            SampleCycle::_10p6_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_10P6_US,
            SampleCycle::_21p3_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_21P3_US,
            SampleCycle::_42p6_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_42P6_US,
            SampleCycle::_85p3_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_85P3_US,
            SampleCycle::_170_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_170_US,
            SampleCycle::_341_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_341_US,
            SampleCycle::_682_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_682_US,
            SampleCycle::_1p37_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_1P37_MS,
            SampleCycle::_2p73_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_2P73_MS,
            SampleCycle::_5p46_ms => sample_time_value = adi::Control0::SAMPLE_CYCLE::_5P46_US,
            SampleCycle::_10p9_ms => sample_time_value = adi::Control0::SAMPLE_CYCLE::_10P9_US,
        }

        self.aux_adi4.control0.write(adi::Control0::RESET_N::CLEAR);

        self.aux_adi4.control0.write(
            sample_time_value
                + adi::Control0::SAMPLE_MODE::SYNC
                + adi::Control0::RESET_N::SET
                + adi::Control0::EN::SET,
        );
    }

    pub fn handle_events(&self) {
        if self.has_data() {
            let data = self.pop_fifo();

            if let Some(index) = self.single_shot_channel.take() {
                self.channel[index].client.map(|client| {
                    client.sample_ready(data);
                });
            }
        }

        // clear the event flags in AUX_EVTCTL otherwise NVIC fires again
        aux::evtctl::REG
            .ev_to_mcu_flags_clr
            .write(aux::evtctl::EvToMcu::ADC_IRQ::SET + aux::evtctl::EvToMcu::ADC_DONE::SET);

        self.nvic.clear_pending();
        self.nvic.enable();
    }

    pub fn single_shot(&self) {
        aux::anaif::REG
            .adc_trigger
            .write(aux::anaif::AdcTrigger::START::SET);
    }

    pub fn set_client(&self, client: &'static hil::adc::Client, channel: &Input) {
        let index: usize = (*channel) as usize;
        self.channel[index].client.set(client);
    }
}

use kernel::hil;
use kernel::ReturnCode;

impl hil::adc::Adc for Adc {
    type Channel = Input;

    fn sample(&self, channel: &Self::Channel) -> ReturnCode {
        let index: usize = (*channel) as usize;

        self.set_input(self.channel[index].aux_input);
        self.flush_fifo();
        self.single_shot();

        // save index so we can fire it back to the right channel
        self.single_shot_channel.set(index);

        ReturnCode::SUCCESS
    }

    fn sample_continuous(&self, _channel: &Self::Channel, _frequency: u32) -> ReturnCode {
        ReturnCode::ENOSUPPORT
    }

    fn stop_sampling(&self) -> ReturnCode {
        ReturnCode::SUCCESS
    }

    fn get_resolution_bits(&self) -> usize {
        ADC_BITS
    }

    /// The returned reference voltage is in millivolts, or `None` if unknown.
    fn get_voltage_reference_mv(&self) -> Option<usize> {
        self.voltage_setting
            .map_or(None, move |setting| match setting {
                Source::Fixed4P5V => Some(4500),
                Source::NominalVdds => self.nominal_voltage,
            })
    }
}

/// Not implemented at all yet
impl hil::adc::AdcHighSpeed for Adc {
    fn sample_highspeed(
        &self,
        _channel: &Self::Channel,
        _frequency: u32,
        _buffer1: &'static mut [u16],
        _length1: usize,
        _buffer2: &'static mut [u16],
        _length2: usize,
    ) -> (
        ReturnCode,
        Option<&'static mut [u16]>,
        Option<&'static mut [u16]>,
    ) {
        (ReturnCode::ENOSUPPORT, None, None)
    }

    fn provide_buffer(
        &self,
        _buf: &'static mut [u16],
        _length: usize,
    ) -> (ReturnCode, Option<&'static mut [u16]>) {
        (ReturnCode::ENOSUPPORT, None)
    }

    fn retrieve_buffers(
        &self,
    ) -> (
        ReturnCode,
        Option<&'static mut [u16]>,
        Option<&'static mut [u16]>,
    ) {
        (ReturnCode::ENOSUPPORT, None, None)
    }
}
