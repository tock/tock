// Generic interface for CRC computation

use returncode::ReturnCode;

#[derive(Copy, Clone)]
pub enum Polynomial {
	CCIT8023,   // Polynomial 0x04C11DB7
	CASTAGNOLI, // Polynomial 0x1EDC6F41
	CCIT16,     // Polynomial 0x1021
}

pub fn poly_from_int(i: usize) -> Option<Polynomial> {
    match i {
        0 => Some(Polynomial::CCIT8023),
        1 => Some(Polynomial::CASTAGNOLI),
        2 => Some(Polynomial::CCIT16),
        _ => None
    }
}

pub trait CRC {
    // Get the version of the CRC unit
    fn get_version(&self) -> u32;

    // Initiate a CRC calculation
    fn compute(&self, data: &[u8], Polynomial) -> ReturnCode;

    // Disable the CRC unit until compute() is next called
    fn disable(&self);
}

pub trait Client {
    // Receive the successful result of a CRC calculation
    fn receive_result(&self, u32);
}
