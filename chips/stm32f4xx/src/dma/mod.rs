use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite};

pub mod dma1;
pub mod dma2;

/// DMA controller
#[repr(C)]
struct DmaRegisters {
    /// low interrupt status register
    lisr: ReadOnly<u32, LISR::Register>,
    /// high interrupt status register
    hisr: ReadOnly<u32, HISR::Register>,
    /// low interrupt flag clear register
    lifcr: ReadWrite<u32, LIFCR::Register>,
    /// high interrupt flag clear register
    hifcr: ReadWrite<u32, HIFCR::Register>,
    /// stream x configuration register
    s0cr: ReadWrite<u32, S0CR::Register>,
    /// stream x number of data register
    s0ndtr: ReadWrite<u32>,
    /// stream x peripheral address register
    s0par: ReadWrite<u32>,
    /// stream x memory 0 address register
    s0m0ar: ReadWrite<u32>,
    /// stream x memory 1 address register
    s0m1ar: ReadWrite<u32>,
    /// stream x FIFO control register
    s0fcr: ReadWrite<u32, S0FCR::Register>,
    /// stream x configuration register
    s1cr: ReadWrite<u32, S1CR::Register>,
    /// stream x number of data register
    s1ndtr: ReadWrite<u32>,
    /// stream x peripheral address register
    s1par: ReadWrite<u32>,
    /// stream x memory 0 address register
    s1m0ar: ReadWrite<u32>,
    /// stream x memory 1 address register
    s1m1ar: ReadWrite<u32>,
    /// stream x FIFO control register
    s1fcr: ReadWrite<u32, S1FCR::Register>,
    /// stream x configuration register
    s2cr: ReadWrite<u32, S2CR::Register>,
    /// stream x number of data register
    s2ndtr: ReadWrite<u32>,
    /// stream x peripheral address register
    s2par: ReadWrite<u32>,
    /// stream x memory 0 address register
    s2m0ar: ReadWrite<u32>,
    /// stream x memory 1 address register
    s2m1ar: ReadWrite<u32>,
    /// stream x FIFO control register
    s2fcr: ReadWrite<u32, S2FCR::Register>,
    /// stream x configuration register
    s3cr: ReadWrite<u32, S3CR::Register>,
    /// stream x number of data register
    s3ndtr: ReadWrite<u32>,
    /// stream x peripheral address register
    s3par: ReadWrite<u32>,
    /// stream x memory 0 address register
    s3m0ar: ReadWrite<u32>,
    /// stream x memory 1 address register
    s3m1ar: ReadWrite<u32>,
    /// stream x FIFO control register
    s3fcr: ReadWrite<u32, S3FCR::Register>,
    /// stream x configuration register
    s4cr: ReadWrite<u32, S4CR::Register>,
    /// stream x number of data register
    s4ndtr: ReadWrite<u32>,
    /// stream x peripheral address register
    s4par: ReadWrite<u32>,
    /// stream x memory 0 address register
    s4m0ar: ReadWrite<u32>,
    /// stream x memory 1 address register
    s4m1ar: ReadWrite<u32>,
    /// stream x FIFO control register
    s4fcr: ReadWrite<u32, S4FCR::Register>,
    /// stream x configuration register
    s5cr: ReadWrite<u32, S5CR::Register>,
    /// stream x number of data register
    s5ndtr: ReadWrite<u32>,
    /// stream x peripheral address register
    s5par: ReadWrite<u32>,
    /// stream x memory 0 address register
    s5m0ar: ReadWrite<u32>,
    /// stream x memory 1 address register
    s5m1ar: ReadWrite<u32>,
    /// stream x FIFO control register
    s5fcr: ReadWrite<u32, S5FCR::Register>,
    /// stream x configuration register
    s6cr: ReadWrite<u32, S6CR::Register>,
    /// stream x number of data register
    s6ndtr: ReadWrite<u32>,
    /// stream x peripheral address register
    s6par: ReadWrite<u32>,
    /// stream x memory 0 address register
    s6m0ar: ReadWrite<u32>,
    /// stream x memory 1 address register
    s6m1ar: ReadWrite<u32>,
    /// stream x FIFO control register
    s6fcr: ReadWrite<u32, S6FCR::Register>,
    /// stream x configuration register
    s7cr: ReadWrite<u32, S7CR::Register>,
    /// stream x number of data register
    s7ndtr: ReadWrite<u32>,
    /// stream x peripheral address register
    s7par: ReadWrite<u32>,
    /// stream x memory 0 address register
    s7m0ar: ReadWrite<u32>,
    /// stream x memory 1 address register
    s7m1ar: ReadWrite<u32>,
    /// stream x FIFO control register
    s7fcr: ReadWrite<u32, S7FCR::Register>,
}

register_bitfields![u32,
    LISR [
        /// Stream x transfer complete interrupt flag (x = 3..0)
        TCIF3 OFFSET(27) NUMBITS(1) [],
        /// Stream x half transfer interrupt flag (x=3..0)
        HTIF3 OFFSET(26) NUMBITS(1) [],
        /// Stream x transfer error interrupt flag (x=3..0)
        TEIF3 OFFSET(25) NUMBITS(1) [],
        /// Stream x direct mode error interrupt flag (x=3..0)
        DMEIF3 OFFSET(24) NUMBITS(1) [],
        /// Stream x FIFO error interrupt flag (x=3..0)
        FEIF3 OFFSET(22) NUMBITS(1) [],
        /// Stream x transfer complete interrupt flag (x = 3..0)
        TCIF2 OFFSET(21) NUMBITS(1) [],
        /// Stream x half transfer interrupt flag (x=3..0)
        HTIF2 OFFSET(20) NUMBITS(1) [],
        /// Stream x transfer error interrupt flag (x=3..0)
        TEIF2 OFFSET(19) NUMBITS(1) [],
        /// Stream x direct mode error interrupt flag (x=3..0)
        DMEIF2 OFFSET(18) NUMBITS(1) [],
        /// Stream x FIFO error interrupt flag (x=3..0)
        FEIF2 OFFSET(16) NUMBITS(1) [],
        /// Stream x transfer complete interrupt flag (x = 3..0)
        TCIF1 OFFSET(11) NUMBITS(1) [],
        /// Stream x half transfer interrupt flag (x=3..0)
        HTIF1 OFFSET(10) NUMBITS(1) [],
        /// Stream x transfer error interrupt flag (x=3..0)
        TEIF1 OFFSET(9) NUMBITS(1) [],
        /// Stream x direct mode error interrupt flag (x=3..0)
        DMEIF1 OFFSET(8) NUMBITS(1) [],
        /// Stream x FIFO error interrupt flag (x=3..0)
        FEIF1 OFFSET(6) NUMBITS(1) [],
        /// Stream x transfer complete interrupt flag (x = 3..0)
        TCIF0 OFFSET(5) NUMBITS(1) [],
        /// Stream x half transfer interrupt flag (x=3..0)
        HTIF0 OFFSET(4) NUMBITS(1) [],
        /// Stream x transfer error interrupt flag (x=3..0)
        TEIF0 OFFSET(3) NUMBITS(1) [],
        /// Stream x direct mode error interrupt flag (x=3..0)
        DMEIF0 OFFSET(2) NUMBITS(1) [],
        /// Stream x FIFO error interrupt flag (x=3..0)
        FEIF0 OFFSET(0) NUMBITS(1) []
    ],
    HISR [
        /// Stream x transfer complete interrupt flag (x=7..4)
        TCIF7 OFFSET(27) NUMBITS(1) [],
        /// Stream x half transfer interrupt flag (x=7..4)
        HTIF7 OFFSET(26) NUMBITS(1) [],
        /// Stream x transfer error interrupt flag (x=7..4)
        TEIF7 OFFSET(25) NUMBITS(1) [],
        /// Stream x direct mode error interrupt flag (x=7..4)
        DMEIF7 OFFSET(24) NUMBITS(1) [],
        /// Stream x FIFO error interrupt flag (x=7..4)
        FEIF7 OFFSET(22) NUMBITS(1) [],
        /// Stream x transfer complete interrupt flag (x=7..4)
        TCIF6 OFFSET(21) NUMBITS(1) [],
        /// Stream x half transfer interrupt flag (x=7..4)
        HTIF6 OFFSET(20) NUMBITS(1) [],
        /// Stream x transfer error interrupt flag (x=7..4)
        TEIF6 OFFSET(19) NUMBITS(1) [],
        /// Stream x direct mode error interrupt flag (x=7..4)
        DMEIF6 OFFSET(18) NUMBITS(1) [],
        /// Stream x FIFO error interrupt flag (x=7..4)
        FEIF6 OFFSET(16) NUMBITS(1) [],
        /// Stream x transfer complete interrupt flag (x=7..4)
        TCIF5 OFFSET(11) NUMBITS(1) [],
        /// Stream x half transfer interrupt flag (x=7..4)
        HTIF5 OFFSET(10) NUMBITS(1) [],
        /// Stream x transfer error interrupt flag (x=7..4)
        TEIF5 OFFSET(9) NUMBITS(1) [],
        /// Stream x direct mode error interrupt flag (x=7..4)
        DMEIF5 OFFSET(8) NUMBITS(1) [],
        /// Stream x FIFO error interrupt flag (x=7..4)
        FEIF5 OFFSET(6) NUMBITS(1) [],
        /// Stream x transfer complete interrupt flag (x=7..4)
        TCIF4 OFFSET(5) NUMBITS(1) [],
        /// Stream x half transfer interrupt flag (x=7..4)
        HTIF4 OFFSET(4) NUMBITS(1) [],
        /// Stream x transfer error interrupt flag (x=7..4)
        TEIF4 OFFSET(3) NUMBITS(1) [],
        /// Stream x direct mode error interrupt flag (x=7..4)
        DMEIF4 OFFSET(2) NUMBITS(1) [],
        /// Stream x FIFO error interrupt flag (x=7..4)
        FEIF4 OFFSET(0) NUMBITS(1) []
    ],
    LIFCR [
        /// Stream x clear transfer complete interrupt flag (x = 3..0)
        CTCIF3 OFFSET(27) NUMBITS(1) [],
        /// Stream x clear half transfer interrupt flag (x = 3..0)
        CHTIF3 OFFSET(26) NUMBITS(1) [],
        /// Stream x clear transfer error interrupt flag (x = 3..0)
        CTEIF3 OFFSET(25) NUMBITS(1) [],
        /// Stream x clear direct mode error interrupt flag (x = 3..0)
        CDMEIF3 OFFSET(24) NUMBITS(1) [],
        /// Stream x clear FIFO error interrupt flag (x = 3..0)
        CFEIF3 OFFSET(22) NUMBITS(1) [],
        /// Stream x clear transfer complete interrupt flag (x = 3..0)
        CTCIF2 OFFSET(21) NUMBITS(1) [],
        /// Stream x clear half transfer interrupt flag (x = 3..0)
        CHTIF2 OFFSET(20) NUMBITS(1) [],
        /// Stream x clear transfer error interrupt flag (x = 3..0)
        CTEIF2 OFFSET(19) NUMBITS(1) [],
        /// Stream x clear direct mode error interrupt flag (x = 3..0)
        CDMEIF2 OFFSET(18) NUMBITS(1) [],
        /// Stream x clear FIFO error interrupt flag (x = 3..0)
        CFEIF2 OFFSET(16) NUMBITS(1) [],
        /// Stream x clear transfer complete interrupt flag (x = 3..0)
        CTCIF1 OFFSET(11) NUMBITS(1) [],
        /// Stream x clear half transfer interrupt flag (x = 3..0)
        CHTIF1 OFFSET(10) NUMBITS(1) [],
        /// Stream x clear transfer error interrupt flag (x = 3..0)
        CTEIF1 OFFSET(9) NUMBITS(1) [],
        /// Stream x clear direct mode error interrupt flag (x = 3..0)
        CDMEIF1 OFFSET(8) NUMBITS(1) [],
        /// Stream x clear FIFO error interrupt flag (x = 3..0)
        CFEIF1 OFFSET(6) NUMBITS(1) [],
        /// Stream x clear transfer complete interrupt flag (x = 3..0)
        CTCIF0 OFFSET(5) NUMBITS(1) [],
        /// Stream x clear half transfer interrupt flag (x = 3..0)
        CHTIF0 OFFSET(4) NUMBITS(1) [],
        /// Stream x clear transfer error interrupt flag (x = 3..0)
        CTEIF0 OFFSET(3) NUMBITS(1) [],
        /// Stream x clear direct mode error interrupt flag (x = 3..0)
        CDMEIF0 OFFSET(2) NUMBITS(1) [],
        /// Stream x clear FIFO error interrupt flag (x = 3..0)
        CFEIF0 OFFSET(0) NUMBITS(1) []
    ],
    HIFCR [
        /// Stream x clear transfer complete interrupt flag (x = 7..4)
        CTCIF7 OFFSET(27) NUMBITS(1) [],
        /// Stream x clear half transfer interrupt flag (x = 7..4)
        CHTIF7 OFFSET(26) NUMBITS(1) [],
        /// Stream x clear transfer error interrupt flag (x = 7..4)
        CTEIF7 OFFSET(25) NUMBITS(1) [],
        /// Stream x clear direct mode error interrupt flag (x = 7..4)
        CDMEIF7 OFFSET(24) NUMBITS(1) [],
        /// Stream x clear FIFO error interrupt flag (x = 7..4)
        CFEIF7 OFFSET(22) NUMBITS(1) [],
        /// Stream x clear transfer complete interrupt flag (x = 7..4)
        CTCIF6 OFFSET(21) NUMBITS(1) [],
        /// Stream x clear half transfer interrupt flag (x = 7..4)
        CHTIF6 OFFSET(20) NUMBITS(1) [],
        /// Stream x clear transfer error interrupt flag (x = 7..4)
        CTEIF6 OFFSET(19) NUMBITS(1) [],
        /// Stream x clear direct mode error interrupt flag (x = 7..4)
        CDMEIF6 OFFSET(18) NUMBITS(1) [],
        /// Stream x clear FIFO error interrupt flag (x = 7..4)
        CFEIF6 OFFSET(16) NUMBITS(1) [],
        /// Stream x clear transfer complete interrupt flag (x = 7..4)
        CTCIF5 OFFSET(11) NUMBITS(1) [],
        /// Stream x clear half transfer interrupt flag (x = 7..4)
        CHTIF5 OFFSET(10) NUMBITS(1) [],
        /// Stream x clear transfer error interrupt flag (x = 7..4)
        CTEIF5 OFFSET(9) NUMBITS(1) [],
        /// Stream x clear direct mode error interrupt flag (x = 7..4)
        CDMEIF5 OFFSET(8) NUMBITS(1) [],
        /// Stream x clear FIFO error interrupt flag (x = 7..4)
        CFEIF5 OFFSET(6) NUMBITS(1) [],
        /// Stream x clear transfer complete interrupt flag (x = 7..4)
        CTCIF4 OFFSET(5) NUMBITS(1) [],
        /// Stream x clear half transfer interrupt flag (x = 7..4)
        CHTIF4 OFFSET(4) NUMBITS(1) [],
        /// Stream x clear transfer error interrupt flag (x = 7..4)
        CTEIF4 OFFSET(3) NUMBITS(1) [],
        /// Stream x clear direct mode error interrupt flag (x = 7..4)
        CDMEIF4 OFFSET(2) NUMBITS(1) [],
        /// Stream x clear FIFO error interrupt flag (x = 7..4)
        CFEIF4 OFFSET(0) NUMBITS(1) []
    ],
    S0CR [
        /// Channel selection
        CHSEL OFFSET(25) NUMBITS(3) [],
        /// Memory burst transfer configuration
        MBURST OFFSET(23) NUMBITS(2) [],
        /// Peripheral burst transfer configuration
        PBURST OFFSET(21) NUMBITS(2) [],
        /// Current target (only in double buffer mode)
        CT OFFSET(19) NUMBITS(1) [],
        /// Double buffer mode
        DBM OFFSET(18) NUMBITS(1) [],
        /// Priority level
        PL OFFSET(16) NUMBITS(2) [],
        /// Peripheral increment offset size
        PINCOS OFFSET(15) NUMBITS(1) [],
        /// Memory data size
        MSIZE OFFSET(13) NUMBITS(2) [],
        /// Peripheral data size
        PSIZE OFFSET(11) NUMBITS(2) [],
        /// Memory increment mode
        MINC OFFSET(10) NUMBITS(1) [],
        /// Peripheral increment mode
        PINC OFFSET(9) NUMBITS(1) [],
        /// Circular mode
        CIRC OFFSET(8) NUMBITS(1) [],
        /// Data transfer direction
        DIR OFFSET(6) NUMBITS(2) [],
        /// Peripheral flow controller
        PFCTRL OFFSET(5) NUMBITS(1) [],
        /// Transfer complete interrupt enable
        TCIE OFFSET(4) NUMBITS(1) [],
        /// Half transfer interrupt enable
        HTIE OFFSET(3) NUMBITS(1) [],
        /// Transfer error interrupt enable
        TEIE OFFSET(2) NUMBITS(1) [],
        /// Direct mode error interrupt enable
        DMEIE OFFSET(1) NUMBITS(1) [],
        /// Stream enable / flag stream ready when read low
        EN OFFSET(0) NUMBITS(1) []
    ],
    S0FCR [
        /// FIFO error interrupt enable
        FEIE OFFSET(7) NUMBITS(1) [],
        /// FIFO status
        FS OFFSET(3) NUMBITS(3) [],
        /// Direct mode disable
        DMDIS OFFSET(2) NUMBITS(1) [],
        /// FIFO threshold selection
        FTH OFFSET(0) NUMBITS(2) []
    ],
    S1CR [
        /// Channel selection
        CHSEL OFFSET(25) NUMBITS(3) [],
        /// Memory burst transfer configuration
        MBURST OFFSET(23) NUMBITS(2) [],
        /// Peripheral burst transfer configuration
        PBURST OFFSET(21) NUMBITS(2) [],
        /// ACK
        ACK OFFSET(20) NUMBITS(1) [],
        /// Current target (only in double buffer mode)
        CT OFFSET(19) NUMBITS(1) [],
        /// Double buffer mode
        DBM OFFSET(18) NUMBITS(1) [],
        /// Priority level
        PL OFFSET(16) NUMBITS(2) [],
        /// Peripheral increment offset size
        PINCOS OFFSET(15) NUMBITS(1) [],
        /// Memory data size
        MSIZE OFFSET(13) NUMBITS(2) [],
        /// Peripheral data size
        PSIZE OFFSET(11) NUMBITS(2) [],
        /// Memory increment mode
        MINC OFFSET(10) NUMBITS(1) [],
        /// Peripheral increment mode
        PINC OFFSET(9) NUMBITS(1) [],
        /// Circular mode
        CIRC OFFSET(8) NUMBITS(1) [],
        /// Data transfer direction
        DIR OFFSET(6) NUMBITS(2) [],
        /// Peripheral flow controller
        PFCTRL OFFSET(5) NUMBITS(1) [],
        /// Transfer complete interrupt enable
        TCIE OFFSET(4) NUMBITS(1) [],
        /// Half transfer interrupt enable
        HTIE OFFSET(3) NUMBITS(1) [],
        /// Transfer error interrupt enable
        TEIE OFFSET(2) NUMBITS(1) [],
        /// Direct mode error interrupt enable
        DMEIE OFFSET(1) NUMBITS(1) [],
        /// Stream enable / flag stream ready when read low
        EN OFFSET(0) NUMBITS(1) []
    ],
    S1FCR [
        /// FIFO error interrupt enable
        FEIE OFFSET(7) NUMBITS(1) [],
        /// FIFO status
        FS OFFSET(3) NUMBITS(3) [],
        /// Direct mode disable
        DMDIS OFFSET(2) NUMBITS(1) [],
        /// FIFO threshold selection
        FTH OFFSET(0) NUMBITS(2) []
    ],
    S2CR [
        /// Channel selection
        CHSEL OFFSET(25) NUMBITS(3) [],
        /// Memory burst transfer configuration
        MBURST OFFSET(23) NUMBITS(2) [],
        /// Peripheral burst transfer configuration
        PBURST OFFSET(21) NUMBITS(2) [],
        /// ACK
        ACK OFFSET(20) NUMBITS(1) [],
        /// Current target (only in double buffer mode)
        CT OFFSET(19) NUMBITS(1) [],
        /// Double buffer mode
        DBM OFFSET(18) NUMBITS(1) [],
        /// Priority level
        PL OFFSET(16) NUMBITS(2) [],
        /// Peripheral increment offset size
        PINCOS OFFSET(15) NUMBITS(1) [],
        /// Memory data size
        MSIZE OFFSET(13) NUMBITS(2) [],
        /// Peripheral data size
        PSIZE OFFSET(11) NUMBITS(2) [],
        /// Memory increment mode
        MINC OFFSET(10) NUMBITS(1) [],
        /// Peripheral increment mode
        PINC OFFSET(9) NUMBITS(1) [],
        /// Circular mode
        CIRC OFFSET(8) NUMBITS(1) [],
        /// Data transfer direction
        DIR OFFSET(6) NUMBITS(2) [],
        /// Peripheral flow controller
        PFCTRL OFFSET(5) NUMBITS(1) [],
        /// Transfer complete interrupt enable
        TCIE OFFSET(4) NUMBITS(1) [],
        /// Half transfer interrupt enable
        HTIE OFFSET(3) NUMBITS(1) [],
        /// Transfer error interrupt enable
        TEIE OFFSET(2) NUMBITS(1) [],
        /// Direct mode error interrupt enable
        DMEIE OFFSET(1) NUMBITS(1) [],
        /// Stream enable / flag stream ready when read low
        EN OFFSET(0) NUMBITS(1) []
    ],
    S2FCR [
        /// FIFO error interrupt enable
        FEIE OFFSET(7) NUMBITS(1) [],
        /// FIFO status
        FS OFFSET(3) NUMBITS(3) [],
        /// Direct mode disable
        DMDIS OFFSET(2) NUMBITS(1) [],
        /// FIFO threshold selection
        FTH OFFSET(0) NUMBITS(2) []
    ],
    S3CR [
        /// Channel selection
        CHSEL OFFSET(25) NUMBITS(3) [],
        /// Memory burst transfer configuration
        MBURST OFFSET(23) NUMBITS(2) [],
        /// Peripheral burst transfer configuration
        PBURST OFFSET(21) NUMBITS(2) [],
        /// ACK
        ACK OFFSET(20) NUMBITS(1) [],
        /// Current target (only in double buffer mode)
        CT OFFSET(19) NUMBITS(1) [],
        /// Double buffer mode
        DBM OFFSET(18) NUMBITS(1) [],
        /// Priority level
        PL OFFSET(16) NUMBITS(2) [],
        /// Peripheral increment offset size
        PINCOS OFFSET(15) NUMBITS(1) [],
        /// Memory data size
        MSIZE OFFSET(13) NUMBITS(2) [],
        /// Peripheral data size
        PSIZE OFFSET(11) NUMBITS(2) [],
        /// Memory increment mode
        MINC OFFSET(10) NUMBITS(1) [],
        /// Peripheral increment mode
        PINC OFFSET(9) NUMBITS(1) [],
        /// Circular mode
        CIRC OFFSET(8) NUMBITS(1) [],
        /// Data transfer direction
        DIR OFFSET(6) NUMBITS(2) [],
        /// Peripheral flow controller
        PFCTRL OFFSET(5) NUMBITS(1) [],
        /// Transfer complete interrupt enable
        TCIE OFFSET(4) NUMBITS(1) [],
        /// Half transfer interrupt enable
        HTIE OFFSET(3) NUMBITS(1) [],
        /// Transfer error interrupt enable
        TEIE OFFSET(2) NUMBITS(1) [],
        /// Direct mode error interrupt enable
        DMEIE OFFSET(1) NUMBITS(1) [],
        /// Stream enable / flag stream ready when read low
        EN OFFSET(0) NUMBITS(1) []
    ],
    S3FCR [
        /// FIFO error interrupt enable
        FEIE OFFSET(7) NUMBITS(1) [],
        /// FIFO status
        FS OFFSET(3) NUMBITS(3) [],
        /// Direct mode disable
        DMDIS OFFSET(2) NUMBITS(1) [],
        /// FIFO threshold selection
        FTH OFFSET(0) NUMBITS(2) []
    ],
    S4CR [
        /// Channel selection
        CHSEL OFFSET(25) NUMBITS(3) [],
        /// Memory burst transfer configuration
        MBURST OFFSET(23) NUMBITS(2) [],
        /// Peripheral burst transfer configuration
        PBURST OFFSET(21) NUMBITS(2) [],
        /// ACK
        ACK OFFSET(20) NUMBITS(1) [],
        /// Current target (only in double buffer mode)
        CT OFFSET(19) NUMBITS(1) [],
        /// Double buffer mode
        DBM OFFSET(18) NUMBITS(1) [],
        /// Priority level
        PL OFFSET(16) NUMBITS(2) [],
        /// Peripheral increment offset size
        PINCOS OFFSET(15) NUMBITS(1) [],
        /// Memory data size
        MSIZE OFFSET(13) NUMBITS(2) [],
        /// Peripheral data size
        PSIZE OFFSET(11) NUMBITS(2) [],
        /// Memory increment mode
        MINC OFFSET(10) NUMBITS(1) [],
        /// Peripheral increment mode
        PINC OFFSET(9) NUMBITS(1) [],
        /// Circular mode
        CIRC OFFSET(8) NUMBITS(1) [],
        /// Data transfer direction
        DIR OFFSET(6) NUMBITS(2) [],
        /// Peripheral flow controller
        PFCTRL OFFSET(5) NUMBITS(1) [],
        /// Transfer complete interrupt enable
        TCIE OFFSET(4) NUMBITS(1) [],
        /// Half transfer interrupt enable
        HTIE OFFSET(3) NUMBITS(1) [],
        /// Transfer error interrupt enable
        TEIE OFFSET(2) NUMBITS(1) [],
        /// Direct mode error interrupt enable
        DMEIE OFFSET(1) NUMBITS(1) [],
        /// Stream enable / flag stream ready when read low
        EN OFFSET(0) NUMBITS(1) []
    ],
    S4FCR [
        /// FIFO error interrupt enable
        FEIE OFFSET(7) NUMBITS(1) [],
        /// FIFO status
        FS OFFSET(3) NUMBITS(3) [],
        /// Direct mode disable
        DMDIS OFFSET(2) NUMBITS(1) [],
        /// FIFO threshold selection
        FTH OFFSET(0) NUMBITS(2) []
    ],
    S5CR [
        /// Channel selection
        CHSEL OFFSET(25) NUMBITS(3) [],
        /// Memory burst transfer configuration
        MBURST OFFSET(23) NUMBITS(2) [],
        /// Peripheral burst transfer configuration
        PBURST OFFSET(21) NUMBITS(2) [],
        /// ACK
        ACK OFFSET(20) NUMBITS(1) [],
        /// Current target (only in double buffer mode)
        CT OFFSET(19) NUMBITS(1) [],
        /// Double buffer mode
        DBM OFFSET(18) NUMBITS(1) [],
        /// Priority level
        PL OFFSET(16) NUMBITS(2) [],
        /// Peripheral increment offset size
        PINCOS OFFSET(15) NUMBITS(1) [],
        /// Memory data size
        MSIZE OFFSET(13) NUMBITS(2) [],
        /// Peripheral data size
        PSIZE OFFSET(11) NUMBITS(2) [],
        /// Memory increment mode
        MINC OFFSET(10) NUMBITS(1) [],
        /// Peripheral increment mode
        PINC OFFSET(9) NUMBITS(1) [],
        /// Circular mode
        CIRC OFFSET(8) NUMBITS(1) [],
        /// Data transfer direction
        DIR OFFSET(6) NUMBITS(2) [],
        /// Peripheral flow controller
        PFCTRL OFFSET(5) NUMBITS(1) [],
        /// Transfer complete interrupt enable
        TCIE OFFSET(4) NUMBITS(1) [],
        /// Half transfer interrupt enable
        HTIE OFFSET(3) NUMBITS(1) [],
        /// Transfer error interrupt enable
        TEIE OFFSET(2) NUMBITS(1) [],
        /// Direct mode error interrupt enable
        DMEIE OFFSET(1) NUMBITS(1) [],
        /// Stream enable / flag stream ready when read low
        EN OFFSET(0) NUMBITS(1) []
    ],
    S5FCR [
        /// FIFO error interrupt enable
        FEIE OFFSET(7) NUMBITS(1) [],
        /// FIFO status
        FS OFFSET(3) NUMBITS(3) [],
        /// Direct mode disable
        DMDIS OFFSET(2) NUMBITS(1) [],
        /// FIFO threshold selection
        FTH OFFSET(0) NUMBITS(2) []
    ],
    S6CR [
        /// Channel selection
        CHSEL OFFSET(25) NUMBITS(3) [],
        /// Memory burst transfer configuration
        MBURST OFFSET(23) NUMBITS(2) [],
        /// Peripheral burst transfer configuration
        PBURST OFFSET(21) NUMBITS(2) [],
        /// ACK
        ACK OFFSET(20) NUMBITS(1) [],
        /// Current target (only in double buffer mode)
        CT OFFSET(19) NUMBITS(1) [],
        /// Double buffer mode
        DBM OFFSET(18) NUMBITS(1) [],
        /// Priority level
        PL OFFSET(16) NUMBITS(2) [],
        /// Peripheral increment offset size
        PINCOS OFFSET(15) NUMBITS(1) [],
        /// Memory data size
        MSIZE OFFSET(13) NUMBITS(2) [],
        /// Peripheral data size
        PSIZE OFFSET(11) NUMBITS(2) [],
        /// Memory increment mode
        MINC OFFSET(10) NUMBITS(1) [],
        /// Peripheral increment mode
        PINC OFFSET(9) NUMBITS(1) [],
        /// Circular mode
        CIRC OFFSET(8) NUMBITS(1) [],
        /// Data transfer direction
        DIR OFFSET(6) NUMBITS(2) [],
        /// Peripheral flow controller
        PFCTRL OFFSET(5) NUMBITS(1) [],
        /// Transfer complete interrupt enable
        TCIE OFFSET(4) NUMBITS(1) [],
        /// Half transfer interrupt enable
        HTIE OFFSET(3) NUMBITS(1) [],
        /// Transfer error interrupt enable
        TEIE OFFSET(2) NUMBITS(1) [],
        /// Direct mode error interrupt enable
        DMEIE OFFSET(1) NUMBITS(1) [],
        /// Stream enable / flag stream ready when read low
        EN OFFSET(0) NUMBITS(1) []
    ],
    S6FCR [
        /// FIFO error interrupt enable
        FEIE OFFSET(7) NUMBITS(1) [],
        /// FIFO status
        FS OFFSET(3) NUMBITS(3) [],
        /// Direct mode disable
        DMDIS OFFSET(2) NUMBITS(1) [],
        /// FIFO threshold selection
        FTH OFFSET(0) NUMBITS(2) []
    ],
    S7CR [
        /// Channel selection
        CHSEL OFFSET(25) NUMBITS(3) [],
        /// Memory burst transfer configuration
        MBURST OFFSET(23) NUMBITS(2) [],
        /// Peripheral burst transfer configuration
        PBURST OFFSET(21) NUMBITS(2) [],
        /// ACK
        ACK OFFSET(20) NUMBITS(1) [],
        /// Current target (only in double buffer mode)
        CT OFFSET(19) NUMBITS(1) [],
        /// Double buffer mode
        DBM OFFSET(18) NUMBITS(1) [],
        /// Priority level
        PL OFFSET(16) NUMBITS(2) [],
        /// Peripheral increment offset size
        PINCOS OFFSET(15) NUMBITS(1) [],
        /// Memory data size
        MSIZE OFFSET(13) NUMBITS(2) [],
        /// Peripheral data size
        PSIZE OFFSET(11) NUMBITS(2) [],
        /// Memory increment mode
        MINC OFFSET(10) NUMBITS(1) [],
        /// Peripheral increment mode
        PINC OFFSET(9) NUMBITS(1) [],
        /// Circular mode
        CIRC OFFSET(8) NUMBITS(1) [],
        /// Data transfer direction
        DIR OFFSET(6) NUMBITS(2) [],
        /// Peripheral flow controller
        PFCTRL OFFSET(5) NUMBITS(1) [],
        /// Transfer complete interrupt enable
        TCIE OFFSET(4) NUMBITS(1) [],
        /// Half transfer interrupt enable
        HTIE OFFSET(3) NUMBITS(1) [],
        /// Transfer error interrupt enable
        TEIE OFFSET(2) NUMBITS(1) [],
        /// Direct mode error interrupt enable
        DMEIE OFFSET(1) NUMBITS(1) [],
        /// Stream enable / flag stream ready when read low
        EN OFFSET(0) NUMBITS(1) []
    ],
    S7FCR [
        /// FIFO error interrupt enable
        FEIE OFFSET(7) NUMBITS(1) [],
        /// FIFO status
        FS OFFSET(3) NUMBITS(3) [],
        /// Direct mode disable
        DMDIS OFFSET(2) NUMBITS(1) [],
        /// FIFO threshold selection
        FTH OFFSET(0) NUMBITS(2) []
    ]
];

/// The DMA stream number. What other microcontrollers refer to as "channel",
/// STM32F446RE refers to as "streams". STM32F446RE has eight streams. A stream
/// transfers data between memory and peripheral.
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum StreamId {
    Stream0 = 0,
    Stream1 = 1,
    Stream2 = 2,
    Stream3 = 3,
    Stream4 = 4,
    Stream5 = 5,
    Stream6 = 6,
    Stream7 = 7,
}

/// Each stream can be selected among up to eight channel requests. This is
/// basically STM32F446RE's way of selecting the peripheral for the stream.
/// Nevertheless, the use of the term channel here is confusing. Table 28
/// describes the mapping between stream, channel, and peripherals.
#[allow(dead_code)]
#[repr(u32)]
enum ChannelId {
    Channel0 = 0b000,
    Channel1 = 0b001,
    Channel2 = 0b010,
    Channel3 = 0b011,
    Channel4 = 0b100,
    Channel5 = 0b101,
    Channel6 = 0b110,
    Channel7 = 0b111,
}

/// DMA transfer direction. Section 9.5.5
#[allow(dead_code)]
#[repr(u32)]
enum Direction {
    PeripheralToMemory = 0b00,
    MemoryToPeripheral = 0b01,
    MemoryToMemory = 0b10,
}

/// DMA data size. Section 9.5.5
#[allow(dead_code)]
#[repr(u32)]
enum Size {
    Byte = 0b00,
    HalfWord = 0b01,
    Word = 0b10,
}

struct Msize(Size);
struct Psize(Size);

/// DMA transfer mode. Section 9.5.10
#[allow(dead_code)]
#[repr(u32)]
enum FifoSize {
    Quarter = 0b00,
    Half = 0b01,
    ThreeFourths = 0b10,
    Full = 0b11,
}

#[allow(dead_code)]
enum TransferMode {
    Direct,
    Fifo(FifoSize),
}

#[derive(Copy, Clone, PartialEq)]
pub enum DmaPeripheral {
    Dma1Peripheral(dma1::Dma1Peripheral),
    Dma2Peripheral(dma2::Dma2Peripheral),
}

impl From<dma1::Dma1Peripheral> for DmaPeripheral {
    fn from(dma1: dma1::Dma1Peripheral) -> Self {
        Self::Dma1Peripheral(dma1)
    }
}

impl From<dma2::Dma2Peripheral> for DmaPeripheral {
    fn from(dma2: dma2::Dma2Peripheral) -> Self {
        Self::Dma2Peripheral(dma2)
    }
}

#[derive(Clone, Copy)]
pub enum Stream<'a> {
    Dma1Stream(&'a dma1::Stream<'static>),
    Dma2Stream(&'a dma2::Stream<'static>),
}

impl<'a> From<&'a dma1::Stream<'static>> for Stream<'a> {
    fn from(dma1: &'a dma1::Stream<'static>) -> Self {
        Self::Dma1Stream(dma1)
    }
}

impl<'a> From<&'a dma2::Stream<'static>> for Stream<'a> {
    fn from(dma2: &'a dma2::Stream<'static>) -> Self {
        Self::Dma2Stream(dma2)
    }
}

macro_rules! stream_fn {
    { $name:ident; $return_type:ty } => {
            pub fn $name(&self) -> $return_type {
                match self {
                    Self::Dma1Stream(stream) => stream.$name(),
                    Self::Dma2Stream(stream) => stream.$name(),
                }
            }
    }
}

impl<'a> Stream<'a> {
    stream_fn! { return_buffer; Option<&'static mut [u8]> }
    stream_fn! { abort_transfer; (Option<&'static mut [u8]>, u32) }

    pub fn do_transfer(&self, buf: &'static mut [u8], len: usize) {
        match self {
            Self::Dma1Stream(stream) => stream.do_transfer(buf, len),
            Self::Dma2Stream(stream) => stream.do_transfer(buf, len),
        }
    }
}
