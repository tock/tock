#![no_std]
#![no_main]
#![feature(const_fn,lang_items)]

extern crate capsules;
#[macro_use(static_init)]
extern crate kernel;
extern crate sam4l;

use kernel::hil::Controller;

mod io;

struct Imix {
    console: &'static capsules::console::Console<'static, sam4l::usart::USART>
}

impl kernel::Platform for Imix {
    fn with_driver<F, R>(&mut self, driver_num: usize, f: F) -> R
        where F: FnOnce(Option<&kernel::Driver>) -> R {
            match driver_num {
                0 => f(Some(self.console)),
                _ => f(None)
            }
    }
}

unsafe fn set_pin_primary_functions() {
    use sam4l::gpio::{PA, PB, PC};
    use sam4l::gpio::PeripheralFunction::{A, B, C, D, E};

    // Configuring pins for RF233
    // SPI
    PC[03].configure(Some(A)); // SPI NPCS0
    PC[02].configure(Some(A)); // SPI NPCS1
    PC[00].configure(Some(A)); // SPI NPCS2
    PC[01].configure(Some(A)); // SPI NPCS3 (RF233)
    PC[06].configure(Some(A)); // SPI CLK
    PC[04].configure(Some(A)); // SPI MISO
    PC[05].configure(Some(A)); // SPI MOSI
    // GIRQ line of RF233
    PA[20].enable();
    PA[20].disable_output();
    PA[20].disable_interrupt();
    // PA00 is RCLK
    // PC14 is RSLP
    // PC15 is RRST
    PC[14].enable();
    PC[14].disable_output();
    PC[15].enable();
    PC[15].disable_output();

    // Right column: Firestorm pin name
    // Left  column: SAM4L peripheral function
    // LI_INT   --  EIC EXTINT2
    PA[04].configure(Some(C));

    // EXTINT1  --  EIC EXTINT1
    PA[06].configure(Some(C));

    // PWM 0    --  GPIO pin
    PA[08].configure(None);

    // PWM 1    --  GPIO pin
    PC[16].configure(None);

    // PWM 2    --  GPIO pin
    PC[17].configure(None);

    // PWM 3    --  GPIO pin
    PC[18].configure(None);

    // AD5      --  ADCIFE AD1
    PA[05].configure(Some(A));

    // AD4      --  ADCIFE AD2
    PA[07].configure(Some(A));

    // AD3      --  ADCIFE AD3
    PB[02].configure(Some(A));

    // AD2      --  ADCIFE AD4
    PB[03].configure(Some(A));

    // AD1      --  ADCIFE AD5
    PB[04].configure(Some(A));

    // AD0      --  ADCIFE AD6
    PB[05].configure(Some(A));


    // BL_SEL   --  USART3 RTS
    PB[06].configure(Some(A));

    //          --  USART3 CTS
    PB[07].configure(Some(A));

    //          --  USART3 CLK
    PB[08].configure(Some(A));

    // PRI_RX   --  USART3 RX
    PB[09].configure(Some(A));

    // PRI_TX   --  USART3 TX
    PB[10].configure(Some(A));

    // U1_CTS   --  USART0 CTS
    PB[11].configure(Some(A));

    // U1_RTS   --  USART0 RTS
    PB[12].configure(Some(A));

    // U1_CLK   --  USART0 CLK
    PB[13].configure(Some(A));

    // U1_RX    --  USART0 RX
    PB[14].configure(Some(A));

    // U1_TX    --  USART0 TX
    PB[15].configure(Some(A));

    // STORMRTS --  USART2 RTS
    PC[07].configure(Some(B));

    // STORMCTS --  USART2 CTS
    PC[08].configure(Some(E));

    // STORMRX  --  USART2 RX
    PC[11].configure(Some(B));

    // STORMTX  --  USART2 TX
    PC[12].configure(Some(B));

    // STORMCLK --  USART2 CLK
    PA[18].configure(Some(A));

    // ESDA     --  TWIMS1 TWD
    PB[00].configure(Some(A));

    // ESCL     --  TWIMS1 TWCK
    PB[01].configure(Some(A));

    // SDA      --  TWIM2 TWD
    PA[21].configure(Some(E));

    // SCL      --  TWIM2 TWCK
    PA[22].configure(Some(E));

    // EPCLK    --  USBC DM
    PA[25].configure(Some(A));

    // EPDAT    --  USBC DP
    PA[26].configure(Some(A));

    // PCLK     --  PARC PCCK
    PC[21].configure(Some(D));
    // PCEN1    --  PARC PCEN1
    PC[22].configure(Some(D));
    // EPGP     --  PARC PCEN2
    PC[23].configure(Some(D));
    // PCD0     --  PARC PCDATA0
    PC[24].configure(Some(D));
    // PCD1     --  PARC PCDATA1
    PC[25].configure(Some(D));
    // PCD2     --  PARC PCDATA2
    PC[26].configure(Some(D));
    // PCD3     --  PARC PCDATA3
    PC[27].configure(Some(D));
    // PCD4     --  PARC PCDATA4
    PC[28].configure(Some(D));
    // PCD5     --  PARC PCDATA5
    PC[29].configure(Some(D));
    // PCD6     --  PARC PCDATA6
    PC[30].configure(Some(D));
    // PCD7     --  PARC PCDATA7
    PC[31].configure(Some(D));

    // P2       -- GPIO Pin
    PA[16].configure(None);
    // P3       -- GPIO Pin
    PA[12].configure(None);
    // P4       -- GPIO Pin
    PC[09].configure(None);
    // P5       -- GPIO Pin
    PA[10].configure(None);
    // P6       -- GPIO Pin
    PA[11].configure(None);
    // P7       -- GPIO Pin
    PA[19].configure(None);
    // P8       -- GPIO Pin
    PA[13].configure(None);

    // none     -- GPIO Pin
    PA[14].configure(None);

    // ACC_INT2 -- GPIO Pin
    PC[20].configure(None);
    // STORMINT -- GPIO Pin
    PA[17].configure(None);
    // TMP_DRDY -- GPIO Pin
    PA[09].configure(None);
    // ACC_INT1 -- GPIO Pin
    PC[13].configure(None);
    // ENSEN    -- GPIO Pin
    PC[19].configure(None);
    // LED0     -- GPIO Pin
    PC[10].configure(None);
}


#[no_mangle]
pub unsafe fn reset_handler() {
    sam4l::init();

    // Source 32Khz and 1Khz clocks from RC23K (SAM4L Datasheet 11.6.8)
    sam4l::bpm::set_ck32source(sam4l::bpm::CK32Source::RC32K);

    set_pin_primary_functions();

    let console = static_init!(
        capsules::console::Console<sam4l::usart::USART>,
        capsules::console::Console::new(&sam4l::usart::USART3,
                     &mut capsules::console::WRITE_BUF,
                     kernel::Container::create()),
        24);
    sam4l::usart::USART3.set_client(console);

    sam4l::usart::USART3.configure(sam4l::usart::USARTParams {
        baud_rate: 115200,
        data_bits: 8,
        parity: kernel::hil::uart::Parity::None,
        mode: kernel::hil::uart::Mode::Normal,
    });

    console.initialize();

    let mut imix = Imix {
        console: console
    };

    let mut chip = sam4l::chip::Sam4l::new();
    kernel::main(&mut imix, &mut chip, load_processes());
}

unsafe fn load_processes() -> &'static mut [Option<kernel::process::Process<'static>>] {
    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
    }

    const NUM_PROCS: usize = 1;

    #[link_section = ".app_memory"]
    static mut MEMORIES: [[u8; 8192]; NUM_PROCS] = [[0; 8192]; NUM_PROCS];

    static mut processes: [Option<kernel::process::Process<'static>>; NUM_PROCS] = [None];

    let mut addr = &_sapps as *const u8;
    for i in 0..NUM_PROCS {
        // The first member of the LoadInfo header contains the total size of each process image. A
        // sentinel value of 0 (invalid because it's smaller than the header itself) is used to
        // mark the end of the list of processes.
        let total_size = *(addr as *const usize);
        if total_size == 0 {
            break;
        }

        let process = &mut processes[i];
        let memory = &mut MEMORIES[i];
        *process = Some(kernel::process::Process::create(addr, total_size, memory));
        // TODO: panic if loading failed?

        addr = addr.offset(total_size as isize);
    }

    if *(addr as *const usize) != 0 {
        panic!("Exceeded maximum NUM_PROCS. {:#x}", *(addr as *const usize));
    }

    &mut processes
}

