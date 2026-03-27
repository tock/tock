use core::fmt::Debug;

pub trait FlashChipSpecific {
    type FlashLatency: RegisterToFlashLatency + Clone + Copy + PartialEq + Debug + Into<u32>;

    fn get_number_wait_cycles_based_on_frequency(frequency_mhz: usize) -> Self::FlashLatency;
}

pub trait RegisterToFlashLatency {
    fn convert_register_to_enum(flash_latency_register: u32) -> Self;
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum FlashLatency16 {
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

impl RegisterToFlashLatency for FlashLatency16 {
    fn convert_register_to_enum(flash_latency_register: u32) -> Self {
        match flash_latency_register {
            0 => Self::Latency0,
            1 => Self::Latency1,
            2 => Self::Latency2,
            3 => Self::Latency3,
            4 => Self::Latency4,
            5 => Self::Latency5,
            6 => Self::Latency6,
            7 => Self::Latency7,
            8 => Self::Latency8,
            9 => Self::Latency9,
            10 => Self::Latency10,
            11 => Self::Latency11,
            12 => Self::Latency12,
            13 => Self::Latency13,
            14 => Self::Latency14,
            _ => Self::Latency15,
        }
    }
}

impl From<FlashLatency16> for u32 {
    fn from(val: FlashLatency16) -> Self {
        val as u32
    }
}
