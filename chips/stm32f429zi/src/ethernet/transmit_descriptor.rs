use kernel::utilities::registers::{register_bitfields, register_structs, InMemoryRegister};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::ErrorCode;

register_bitfields![u32,
TDES0 [
    OWN OFFSET(31) NUMBITS(1) [],
    IC OFFSET(30) NUMBITS(1) [],
    LS OFFSET(29) NUMBITS(1) [],
    FS OFFSET(28) NUMBITS(1) [],
    DC OFFSET(27) NUMBITS(1) [],
    DP OFFSET(26) NUMBITS(1) [],
    TTSE OFFSET(25) NUMBITS(1) [],
    CIC OFFSET(22) NUMBITS(2) [
        ChecksumInsertionDisabled = 0,
        IpHeaderChecksumInserionOnly = 1,
        IpHeaderAndPayloadChecksumInsertion = 2,
        IpHeaderPayloadAndPseudoHeaderChecksumInserion = 3,
    ],
    TER OFFSET(21) NUMBITS(1) [],
    TCH OFFSET(20) NUMBITS(1) [],
    TTSS OFFSET(17) NUMBITS(1) [],
    IHE OFFSET(16) NUMBITS(1) [],
    ES OFFSET(15) NUMBITS(1) [],
    JT OFFSET(14) NUMBITS(1) [],
    FF OFFSET(13) NUMBITS(1) [],
    IPE OFFSET(12) NUMBITS(1) [],
    LCA OFFSET(11) NUMBITS(1) [],
    NC OFFSET(10) NUMBITS(1) [],
    LCO OFFSET(9) NUMBITS(1) [],
    EC OFFSET(8) NUMBITS(1) [],
    VF OFFSET(7) NUMBITS(1) [],
    CC OFFSET(3) NUMBITS(4) [],
    ED OFFSET(2) NUMBITS(1) [],
    UF OFFSET(1) NUMBITS(1) [],
    DB OFFSET(1) NUMBITS(1) [],
],
TDES1 [
    TBS2 OFFSET(16) NUMBITS(13) [],
    TBS1 OFFSET(0) NUMBITS(13) [],
],
];

register_structs! {
    pub(in crate::ethernet) TransmitDescriptor {
        (0x000 => tdes0: InMemoryRegister<u32, TDES0::Register>),
        (0x004 => tdes1: InMemoryRegister<u32, TDES1::Register>),
        (0x008 => tdes2: InMemoryRegister<u32, ()>),
        (0x00C => tdes3: InMemoryRegister<u32, ()>),
        (0x010 => @END),
    }
}

impl TransmitDescriptor {
    pub(in crate::ethernet) fn new() -> Self {
        Self {
            tdes0: InMemoryRegister::new(0),
            tdes1: InMemoryRegister::new(0),
            tdes2: InMemoryRegister::new(0),
            tdes3: InMemoryRegister::new(0),
        }
    }

    pub(in crate::ethernet) fn enable_interrupt_on_completion(&self) {
        self.tdes0.modify(TDES0::IC::SET);
    }

    pub(in crate::ethernet) fn disable_interrupt_on_completion(&self) {
        self.tdes0.modify(TDES0::IC::CLEAR);
    }

    pub(in crate::ethernet) fn is_interrupt_on_completion_enabled(&self) -> bool {
        self.tdes0.is_set(TDES0::IC)
    }

    pub(in crate::ethernet) fn acquire(&self) {
        self.tdes0.modify(TDES0::OWN::SET);
    }

    pub(in crate::ethernet) fn release(&self) {
        self.tdes0.modify(TDES0::OWN::CLEAR);
    }

    pub(in crate::ethernet) fn is_acquired(&self) -> bool {
        self.tdes0.is_set(TDES0::OWN)
    }

    pub(in crate::ethernet) fn set_as_last_segment(&self) {
        self.tdes0.modify(TDES0::LS::SET);
    }

    pub(in crate::ethernet) fn clear_as_last_segment(&self) {
        self.tdes0.modify(TDES0::LS::CLEAR);
    }

    pub(in crate::ethernet) fn is_last_segment(&self) -> bool {
        self.tdes0.is_set(TDES0::LS)
    }

    pub(in crate::ethernet) fn set_as_first_segment(&self) {
        self.tdes0.modify(TDES0::FS::SET);
    }

    pub(in crate::ethernet) fn clear_as_first_segment(&self) {
        self.tdes0.modify(TDES0::FS::CLEAR);
    }

    pub(in crate::ethernet) fn is_first_segment(&self) -> bool {
        self.tdes0.is_set(TDES0::FS)
    }

    pub(in crate::ethernet) fn enable_crc(&self) {
        self.tdes0.modify(TDES0::DC::CLEAR);
    }

    pub(in crate::ethernet) fn disable_crc(&self) {
        self.tdes0.modify(TDES0::DC::SET);
    }

    pub(in crate::ethernet) fn is_crc_disabled(&self) -> bool {
        self.tdes0.is_set(TDES0::DC)
    }

    pub(in crate::ethernet) fn enable_pad(&self) {
        self.tdes0.modify(TDES0::DP::CLEAR);
    }

    pub(in crate::ethernet) fn disable_pad(&self) {
        self.tdes0.modify(TDES0::DP::SET);
    }

    pub(in crate::ethernet) fn is_pad_disabled(&self) -> bool {
        self.tdes0.is_set(TDES0::DP)
    }

    pub(in crate::ethernet) fn set_transmit_end_of_ring(&self) {
        self.tdes0.modify(TDES0::TER::SET);
    }

    pub(in crate::ethernet) fn clear_transmit_end_of_ring(&self) {
        self.tdes0.modify(TDES0::TER::CLEAR);
    }

    pub(in crate::ethernet) fn is_transmit_end_of_ring(&self) -> bool {
        self.tdes0.is_set(TDES0::TER)
    }

    pub(in crate::ethernet) fn set_buffer1_size(&self, size: u16) -> Result<(), ErrorCode> {
        if size >= 1 << 14 {
            return Err(ErrorCode::SIZE);
        }

        self.tdes1.modify(TDES1::TBS1.val(size as u32));

        Ok(())
    }

    pub(in crate::ethernet) fn get_buffer1_size(&self) -> u16 {
        self.tdes1.read(TDES1::TBS1) as u16
    }

    pub(in crate::ethernet) fn set_buffer1_address(&self, address: u32) {
        self.tdes2.set(address);
    }

    pub(in crate::ethernet) fn get_buffer1_address(&self) -> u32 {
        self.tdes2.get()
    }

    pub(in crate::ethernet) fn error_occurred(&self) -> bool {
        self.tdes0.is_set(TDES0::ES)
    }
}

pub mod tests {
    use super::*;
    use kernel::debug;

    pub fn test_transmit_descriptor() {
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("Testing Ethernet basic configuration...");

        let transmit_descriptor = TransmitDescriptor::new();

        transmit_descriptor.acquire();
        assert_eq!(true, transmit_descriptor.is_acquired());
        transmit_descriptor.release();
        assert_eq!(false, transmit_descriptor.is_acquired());

        transmit_descriptor.enable_interrupt_on_completion();
        assert_eq!(true, transmit_descriptor.is_interrupt_on_completion_enabled());
        transmit_descriptor.disable_interrupt_on_completion();
        assert_eq!(false, transmit_descriptor.is_interrupt_on_completion_enabled());

        transmit_descriptor.set_as_last_segment();
        assert_eq!(true, transmit_descriptor.is_last_segment());
        transmit_descriptor.clear_as_last_segment();
        assert_eq!(false, transmit_descriptor.is_last_segment());

        transmit_descriptor.set_as_first_segment();
        assert_eq!(true, transmit_descriptor.is_first_segment());
        transmit_descriptor.clear_as_first_segment();
        assert_eq!(false, transmit_descriptor.is_first_segment());

        transmit_descriptor.disable_crc();
        assert_eq!(true, transmit_descriptor.is_crc_disabled());
        transmit_descriptor.enable_crc();
        assert_eq!(false, transmit_descriptor.is_crc_disabled());

        transmit_descriptor.disable_pad();
        assert_eq!(true, transmit_descriptor.is_pad_disabled());
        transmit_descriptor.enable_pad();
        assert_eq!(false, transmit_descriptor.is_pad_disabled());

        transmit_descriptor.set_transmit_end_of_ring();
        assert_eq!(true, transmit_descriptor.is_transmit_end_of_ring());
        transmit_descriptor.clear_transmit_end_of_ring();
        assert_eq!(false, transmit_descriptor.is_transmit_end_of_ring());

        assert_eq!(Ok(()), transmit_descriptor.set_buffer1_size(1234));
        assert_eq!(1234, transmit_descriptor.get_buffer1_size());
        assert_eq!(Err(ErrorCode::SIZE), transmit_descriptor.set_buffer1_size(60102));
        assert_eq!(1234, transmit_descriptor.get_buffer1_size());

        let x: u32 = 8;
        transmit_descriptor.set_buffer1_address(&x as *const u32 as u32);
        assert_eq!(&x as *const u32 as u32, transmit_descriptor.get_buffer1_address());

        debug!("Finished testing transmit descriptor...");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    }
}
