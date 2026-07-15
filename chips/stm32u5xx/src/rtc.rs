// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

// RTC Driver for the STM32U545RE-Q
// Referance Document:
// RM0456 Reference Manual: https://www.st.com/resource/en/reference_manual/rm0456-stm32u5-series-armbased-32bit-mcus-stmicroelectronics.pdf

use super::rcc;
use core::cell::Cell;
use kernel::ErrorCode;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::date_time;
use kernel::hil::date_time::{DateTimeClient, DateTimeValues, DayOfWeek, Month};
use kernel::utilities::StaticRef;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};
use kernel::utilities::registers::{ReadOnly, WriteOnly, interfaces::Writeable, register_structs};
use kernel::utilities::registers::{ReadWrite, register_bitfields};

register_structs! {
pub  RtcRegisters{

    /// The RTC_TR is the calendar time shadow register. This register must be written in initialization mode only.
    (0x00 => rtc_tr: ReadWrite<u32, RTC_TR::Register>),

    /// The RTC_DR is the calendar date shadow register. This register must be written in initialization mode only.
    (0x04 => rtc_dr: ReadWrite<u32, RTC_DR::Register>),

    /// RTC subsecond register
    (0x08 => rtc_ssr: ReadOnly<u32, RTC_SSR::Register>),

    /// RTC initialization control and status regiter.
    (0x0C => rtc_icsr: ReadWrite<u32, RTC_ICSR::Register>),

    /// RTC prescaler register
    (0x10 => rtc_prer: ReadWrite<u32,RTC_PRER::Register>),

    /// RTC wake-up timer register
    (0x14 => rtc_wutr: ReadWrite<u32,RTC_WUTR::Register>),

    /// RTC control register
    (0x18 => rtc_cr: ReadWrite<u32,RTC_CR::Register>),

    /// RTC privilege mode control register. This register can be written only when the APB access is privileged.
    (0x1C => rtc_privcfgr: ReadWrite<u32,RTC_PRIVCFGR::Register>),

    /// RTC secure configuration register. This register can be written only when the APB access is secure.
    (0x20 => rtc_seccfgr: ReadWrite<u32, RTC_SECCFGR::Register>),

    /// RTC write protection register
    (0x24 => rtc_wpr: WriteOnly<u32,RTC_WPR::Register>),

    /// RTC calibration register
    (0x28 => rtc_calr: ReadWrite<u32, RTC_CALR::Register>),

    /// RTC shift control register
    (0x2C => rtc_shiftr: WriteOnly<u32,RTC_SHIFTR::Register>),

    /// RTC timestamp time register
    (0x30 => rtc_tstr: ReadOnly<u32, RTC_TSTR::Register>),

    /// RTC timestamp data register
    (0x34 => rtc_tsdr: ReadOnly<u32,RTC_TSDR::Register>),

    /// RTC timestamp subsecond register
    (0x38 => rtc_tsssr: ReadOnly<u32,RTC_TSSSR::Register>),

    (0x3C => _padding),

    /// RTC alarm A register.This register can be written only when ALRAE is reset in RTC_CR register.
    (0x40 => rtc_alrmar: ReadWrite<u32,RTC_ALRMAR::Register>),

    /// RTC alarm A subsecond register.
    (0x44 => rtc_alrmassr: ReadWrite<u32,RTC_ALRMASSR::Register>),

    /// RTC alarm B register
    (0x48 => rtc_alrmbr: ReadWrite<u32,RTC_ALRMBR::Register>),

    /// RTC alarm B subsecond register
    (0x4C => rtc_alrmbssr: ReadWrite<u32,RTC_ALRMBSSR::Register>),

    /// RTC status register
    (0x50 => rtc_sr: ReadOnly<u32, RTC_SR::Register>),

    /// RTC nonsecure masked interrupt status register
    (0x54 => rtc_misr: ReadOnly<u32,RTC_MISR::Register>),

    /// RTC secure masked interrupt status register
    (0x58 => rtc_smisr: ReadOnly<u32,RTC_SMISR::Register>),

    /// RTC status clear register
    (0x5C => rtc_scr: WriteOnly<u32,RTC_SCR::Register>),

    (0x60 => _padding1),
    (0x64 => _padding2),
    (0x68 => _padding3),
    (0x6C => _padding4),

    /// RTC alarm A binary mode register.This register can be written only when ALRAE is reset in RTC_CR register.
    (0x70 => rtc_alrabinr: ReadWrite<u32,RTC_ALRABINR::Register>),

    /// RTC alarm B binary mode register.This register can be written only when ALRBE is reset in RTC_CR register.
    (0x74 => rtc_alrbbinr: ReadWrite<u32,RTC_ALRBBINR::Register>),

    (0x78 => @END),
}
}
register_bitfields![u32,
RTC_TR[
    /// AM/PM notation. 0: AM or 24-hour format, 1: PM
    PM OFFSET(22) NUMBITS(1) [],
    /// Hour tens in BCD format
    HT OFFSET(20) NUMBITS(2) [],
    /// Hour units in BCD format
    HU OFFSET(16) NUMBITS(4) [],
    /// Minute tens in BCD format
    MNT OFFSET(12) NUMBITS(3) [],
    /// Minute units in BCD format
    MNU OFFSET(8) NUMBITS(4) [],
    /// Second tens in BCD format
    ST OFFSET(4) NUMBITS(3) [],
    /// Second units in BCD format
    SU OFFSET(0) NUMBITS(4) [],
],
RTC_DR[
    /// Year tens in BCD format
    YT OFFSET(20) NUMBITS(4) [],
    /// Year units in BCD format
    YU OFFSET(16) NUMBITS(4) [],
    /// Week day units
    WDU OFFSET(13) NUMBITS(3) [],
    /// Month tens in BCD format
    MT OFFSET(12) NUMBITS(1) [],
    /// Month units in BCD format
    MU OFFSET(8) NUMBITS(4) [],
    /// Date tens in BCD format
    DT OFFSET(4) NUMBITS(1) [],
    /// Date units in BCD format
    DU OFFSET(0) NUMBITS(4) [],
],

RTC_SSR[
    /// Sub second value / synchronous binary counter LSB values
    SS OFFSET(0) NUMBITS(16) [],
],

RTC_ICSR[
    /// Wake-up timer write flag
    WUTWF OFFSET(2) NUMBITS(1) [],
    /// Shift operation pending
    SHPF OFFSET(3) NUMBITS(1) [],
    /// Initalization status flag
    INITS OFFSET(4) NUMBITS(1) [],
    /// Registers synchronization flag
    RSF OFFSET(5) NUMBITS(1) [],
    /// Initialization flag
    INITF OFFSET(6) NUMBITS(1) [],
    /// Initialization mode
    INIT OFFSET(7) NUMBITS(1) [],
    /// Binary mode
    BIN OFFSET(8) NUMBITS(2) [],
    /// BCD update
    BCDU OFFSET(10) NUMBITS(3) [],
    /// Recalibration pending flag
    RECALPF OFFSET(16) NUMBITS(1) [],
],

RTC_PRER[
    /// Synchronous prescaler factor
    PREDIV_S OFFSET(0) NUMBITS(15) [],
    /// Asynchronous prescaler factor
    PREDIV_A OFFSET(16) NUMBITS(7) [],
],
RTC_WUTR[
    /// Wake-up auto-reload output clear value
    WUTOCLR OFFSET(16) NUMBITS(16) [],
    /// Wake-up auto-reload value bits
    WUT OFFSET(0) NUMBITS(16) [],
],
RTC_CR[
    /// RTC_OUT2 output enalbe
    OUT2EN OFFSET(31) NUMBITS(1) [],
    /// TAMPALRM output type
    TAMPALRM_TYPE OFFSET(30) NUMBITS(1) [],
    /// TAMPALRM pull-up enable
    TAMPALRM_PU OFFSET(29) NUMBITS(1) [],
    /// Alarm B flag automatic clear
    ALRBFCLR OFFSET(28) NUMBITS(1) [],
    /// Alarm A flag automatic clear
    ALRAFCLR OFFSET(27) NUMBITS(1) [],
    ///Tamper detection output enable on TAMPALRM
    TAMPOE OFFSET(26) NUMBITS(1) [],
    /// Activate timestamp on tamper detection event
    TAMPTS OFFSET(25) NUMBITS(1) [],
    /// Timestamp on internal event enable
    ITSE OFFSET(24) NUMBITS(1) [],
    /// Calibration output enable
    COE OFFSET(23) NUMBITS(1) [],
    /// Output selection
    OSEL OFFSET(21) NUMBITS(2) [],
    /// Output polarity
    POL OFFSET(20) NUMBITS(1) [],
    /// Calibration output selection
    COSEL OFFSET(19) NUMBITS(1) [],
    /// Backup
    BKP OFFSET(18) NUMBITS(1) [],
    /// Subtract 1 hour (winter time change)
    SUB1H OFFSET(17) NUMBITS(1) [],
    /// Add 1 hour (sumer time change)
    AD1H OFFSET(16)  NUMBITS(1) [],
    /// Timestampt interrupt enable
    TSIE OFFSET(15) NUMBITS(1) [],
    /// Wake-up timer interrupt enable
    WUTIE OFFSET(14) NUMBITS(1) [],
    /// Alarm B interrupt enable
    ALRBIE OFFSET(13) NUMBITS(1) [],
    /// Alarm A interrupt enable
    ALRAIE OFFSET(12) NUMBITS(1) [],
    /// Timestamp enable
    TSE OFFSET(11) NUMBITS(1) [],
    /// Wake-up timer enable
    WUTE OFFSET(10) NUMBITS(1) [],
    /// Alarm B enable
    ALRBE OFFSET(9) NUMBITS(1) [],
    /// Alarm A enable
    ALRAE OFFSET(8) NUMBITS(1) [],
    /// SSR underflow interrupt enable
    SSRUIE OFFSET(7) NUMBITS(1) [],
    /// Hour Format ( 0 -> 24h , 1 -> AM/PM)
    FMT OFFSET(6) NUMBITS(1) [],
    /// Bypas the shadow registers
    BYPSHAD OFFSET(5) NUMBITS(1) [],
    /// RTC_REFIN reference clock detection enable (50 or 60 Hz)
    REFCKON OFFSET(4) NUMBITS(1) [],
    /// Timestamp event active edge
    TSEDGE OFFSET(3) NUMBITS(1) [],
    /// ck_wut wake-up clock selection
    WUCKSEL OFFSET(0) NUMBITS(2) [],

],
RTC_PRIVCFGR[
    /// RTC privilege protecction
    PRIV OFFSET(15) NUMBITS(1) [],
    /// Initialization privilege protection
    INITPRIV OFFSET(14) NUMBITS(1) [],
    /// Shift register, daylight saving, calibration and reference clock privilege protection
    CALPRIV OFFSET(13) NUMBITS(1) [],
    /// Timestamp privilege protection
    TSPRIV OFFSET(3) NUMBITS(1) [],
    /// Wake-up timer privilege protection
    WUTPRIV OFFSET(2) NUMBITS(1) [],
    /// Alarm B privilege protection
    ALRBPRIV OFFSET(1) NUMBITS(1) [],
    /// Alarm A and SSR underflow privilege protection
    ALRAPRIV OFFSET(0) NUMBITS(1) [],
],
RTC_SECCFGR[
    /// RTC global protection
    SEC OFFSET(15) NUMBITS(1) [],
    /// Initialization protection
    INITSEC OFFSET(14) NUMBITS(1) [],
    /// Shift register, daylight saving, calibration and reference clock protection
    CALSEC OFFSET(13) NUMBITS(1) [],
    /// Timestamp protection
    TSSEC OFFSET(3) NUMBITS(1) [],
    /// Wake-up timer protection
    WUTSEC OFFSET(2) NUMBITS(1) [],
    /// Alarm B protection
    ALRBSEC OFFSET(1) NUMBITS(1) [],
    /// Alarm A and SSR underflow protection
    ALRASEC OFFSET(0) NUMBITS(1) [],
],
RTC_WPR[
    /// Write protection key
    KEY OFFSET(0) NUMBITS(8) [],
],
RTC_CALR[
    /// Increase frequenccey of RTC by 488.5 ppm
    CALP OFFSET(15)  NUMBITS(1) [],
    /// use an 8-second calibration cycle period
    CALW8 OFFSET(14) NUMBITS(1) [],
    /// Use a 16-second calibration cycle period
    CALW16 OFFSET(13) NUMBITS(1) [],
    /// RTC low-power mode
    LPCAL OFFSET(12) NUMBITS(1) [],
    /// Calibration minus
    CALM OFFSET(0) NUMBITS(9) [],
],
RTC_SHIFTR[
    /// Add one second
    ADD1S OFFSET(31) NUMBITS(1) [],
    /// Subtract a fraction of a second  = SUBFS / (PREDIV_S + 1)
    SUBFS OFFSET(0) NUMBITS(15)[],
],
RTC_TSTR[
    /// AM/PM notation (0 is 24h)
    PM OFFSET(22) NUMBITS(1) [],
    /// Hour tens in BCD format
    HT OFFSET(20) NUMBITS(2)[],
    /// Hour units in BCD format
    HU OFFSET(16) NUMBITS(4)[],
    /// Minute tens in BCD format
    MNT OFFSET(12) NUMBITS(3)[],
    /// Minute units in BCD format
    MNU OFFSET(8) NUMBITS(4)[],
    /// Second tens in BCD format
    ST OFFSET(4) NUMBITS(3) [],
    /// Second units in BCD format
    SU OFFSET(0) NUMBITS(4) [],
],

RTC_TSDR[
    /// Week day units
    WDU OFFSET(13) NUMBITS(3) [],
    /// Month tens in BCD format
    MT OFFSET(12) NUMBITS(1) [],
    /// Month units in BCD format
    MU OFFSET(8) NUMBITS(4) [],
    /// Day tens in BCD format
    DT OFFSET(4) NUMBITS(2)[],
    /// Day units in BCD format
    DU OFFSET(0) NUMBITS(4)[],
],
RTC_TSSSR[
    /// Subsecond value/synchronous binary counter values
    SS OFFSET(0) NUMBITS(32) []
],
RTC_ALRMAR[
    /// Alarm A date mask
    MSK4 OFFSET(31) NUMBITS(1) [],
    /// Week day selection
    WDSEL OFFSET(30) NUMBITS(1) [],
    /// Date tens in BCD format
    DT OFFSET(28) NUMBITS(2) [],
    /// Date units or day in BCD format
    DU OFFSET(24) NUMBITS(4) [],
    /// Alarm A hours mask
    MSK3 OFFSET(23) NUMBITS(1) [],
    /// AM/PM notation
    PM OFFSET(22) NUMBITS(1) [],
    /// Hour tens in BCD format
    HT OFFSET(20) NUMBITS(2) [],
    /// Hour units in BCD format
    HU OFFSET(16) NUMBITS(4) [],
    /// Alarm A minutes mask
    MSK2 OFFSET(15) NUMBITS(1) [],
    /// Minute tens in BCD format
    MNT OFFSET(12) NUMBITS(3) [],
    ///Minute units in BCD format
    MNU OFFSET(8) NUMBITS(4) [],
    /// Alarm A seconds mask
    MSK1 OFFSET(7) NUMBITS(1) [],
    /// Second tens in BCD format
    ST OFFSET(4) NUMBITS(3) [],
    /// Second units in BCD format
    SU OFFSET(0) NUMBITS(4) [],
],
RTC_ALRMASSR[
    /// Clear synchronous counter on alarm
    SSCLR OFFSET(31) NUMBITS(1) [],
    /// Mask the MSBs starting at this bit
    MASKSS OFFSET(24) NUMBITS(5)[],
    /// Subseconds value
    SS OFFSET(0) NUMBITS(15)[],
],
RTC_ALRMBR[
    /// Alarm B date mask
    MSK4 OFFSET(31) NUMBITS(1) [],
    /// Week day selection
    WDSEL OFFSET(30) NUMBITS(1)[],
    /// Data tens in BCD format
    DT OFFSET(28) NUMBITS(2) [],
    /// Date units or day in BCD format
    DU OFFSET(24) NUMBITS(4) [],
    /// Alarm B hours mask
    MSK3 OFFSET(23) NUMBITS(1) [],
    /// AM/PM notation
    PM OFFSET(22) NUMBITS(1) [],
    /// Hour tens in BCD format
    HT OFFSET(20) NUMBITS(2) [],
    /// Hour units in BCD foramt
    HU OFFSET(16) NUMBITS(4) [],
    /// Alarm B minutes mask
    MSK2 OFFSET(15) NUMBITS(1) [],
    /// Minute tens in BCD fomrmat
    MNT OFFSET(12) NUMBITS(3) [],
    /// Minute units in BCD format
    MNU OFFSET(8) NUMBITS(4)  [],
    /// Alarm B seconds mask
    MSK1 OFFSET(7) NUMBITS(1) [],
    /// Second tens in BCD format
    ST OFFSET(4) NUMBITS(3) [],
    /// Second units in BCD format
    SU OFFSET(0) NUMBITS(4) [],
],
RTC_ALRMBSSR[
    /// Clear synchronous counter on alarm
    SSCLR OFFSET(31) NUMBITS(1) [],
    /// Mask the MSBs starting at this bit
    MASKSS OFFSET(24) NUMBITS(5) [],
    /// Subseconds value
    SS OFFSET(0) NUMBITS(15) [],
],
RTC_SR [
    /// SSR underflow flag
    SSRUF OFFSET(6) NUMBITS(1) [],
    /// Internal timestamp flag
    ITSF OFFSET(5) NUMBITS(1) [],
    /// Timestamp overflow flag
    TSOVF OFFSET(4) NUMBITS(1) [],
    /// Timestamp flag
    TSF OFFSET(3) NUMBITS(1) [],
    ///Wake-up timer flag
    WUTF OFFSET(2) NUMBITS(1) [],
    /// Alarm B flag
    ALRBF OFFSET(1) NUMBITS(1) [],
    /// Alarm A flag
    ALRAF OFFSET(0) NUMBITS(1) [],
],
RTC_MISR [
    /// SSR underflow nonsecure masked flag
    SSRUMF OFFSET(6) NUMBITS(1) [],
    /// Internal timestamp nonsecure masked flag
    ITSMF OFFSET(5) NUMBITS(1) [],
    /// Timestamp overflow nonsecure masked flag
    TSOVMF OFFSET(4) NUMBITS(1) [],
    /// Timestamp nonsecure masked flag
    TSMF OFFSET(3) NUMBITS(1) [],
    /// Wake-up timer nonsecure masked flag
    WUTMF OFFSET(2) NUMBITS(1) [],
    /// Alarm B nonsecure masked flag
    ALRBMF OFFSET(1) NUMBITS(1) [],
    /// Alarm A masked flag
    ALRAMF OFFSET(0) NUMBITS(1) [],
],
RTC_SMISR [
    /// SSR underflow secure masked flag
    SSRUMF OFFSET(6) NUMBITS(1) [],
    /// Internal timestamp interrupt secure masked flag
    ITSMF OFFSET(5) NUMBITS(1) [],
    /// Timestamp overflow interrupt secure masked flag
    TSOVMF OFFSET(4) NUMBITS(1) [],
    /// Timestamp interrupt secure masked flag
    TSMF OFFSET(3) NUMBITS(1) [],
    /// Wake-up timer interrupt secure masked flag
    WUTMF OFFSET(2) NUMBITS(1) [],
    /// Alarm B interrupt secure masked flag
    ALRBMF OFFSET(1) NUMBITS(1) [],
    /// Alarm A interrupt secure masked flag
    ALRAMF OFFSET(0) NUMBITS(1) [],
],
RTC_SCR [
    /// Clear SSR underflow flag
    CSSRUF OFFSET(6) NUMBITS(1) [],
    /// Clear internal timestamp flag
    CITSF OFFSET(5) NUMBITS(1) [],
    /// Clear timestamp overflow flag
    CTSOVF OFFSET(4) NUMBITS(1) [],
    /// Clear timestamp overflow flag
    CTSF OFFSET(3)  NUMBITS(1) [],
    /// Clear wake-up timer flag
    CWUTF OFFSET(2) NUMBITS(1) [],
    /// Clear alarm B flag
    CALRBF OFFSET(1) NUMBITS(1) [],
    /// Clear alarm A flag
    CALRAF OFFSET(0) NUMBITS(1) [],
],
RTC_ALRABINR[
    /// Synchronous counter alarm value
    SS OFFSET(0) NUMBITS(32) [],
],
RTC_ALRBBINR[
    /// Synchronous counter alarm value
    SS OFFSET(0) NUMBITS(32) [],
],
];

register_structs! {
pub PwrRegisters {
    (0x00 => _padding),
    /// PWR disable backup domain register
    (0x28 => pwr_dbpr: ReadWrite<u32, PWR_DBPR::Register>),
    (0x2C => @END),
}
}
register_bitfields![u32,
    PWR_DBPR [
        /// Disable backup domain write protection
        DBP OFFSET(0) NUMBITS(1) []
    ],
];

// Real Time Clock
// RM0456, Table 6. Memoy map and peripheral register boundary addresses - Page 145
const RTC_BASE: StaticRef<RtcRegisters> =
    unsafe { StaticRef::new(0x46007800 as *const RtcRegisters) };
// Power Control
// RM0456, Table 6. Memoy map and peripheral register boundary addresses - Page 145
const PWR_BASE: StaticRef<PwrRegisters> =
    unsafe { StaticRef::new(0x46020800 as *const PwrRegisters) };

#[derive(Clone, Copy)]
enum DeferredCallTask {
    Get,
    Set,
}

pub struct Rtc<'a> {
    registers: StaticRef<RtcRegisters>,
    rcc: &'a rcc::Rcc,
    client: OptionalCell<&'a dyn date_time::DateTimeClient>,
    time: Cell<DateTimeValues>,

    deferred_call: DeferredCall,
    deferred_call_task: OptionalCell<DeferredCallTask>,
}

impl DeferredCallClient for Rtc<'_> {
    fn handle_deferred_call(&self) {
        self.deferred_call_task.take().map(|value| match value {
            DeferredCallTask::Get => self
                .client
                .map(|client| client.get_date_time_done(Ok(self.time.get()))),
            DeferredCallTask::Set => self.client.map(|client| client.set_date_time_done(Ok(()))),
        });
    }
    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}

impl<'a> Rtc<'a> {
    pub fn new(rcc: &'a rcc::Rcc) -> Rtc<'a> {
        Rtc {
            registers: RTC_BASE,
            rcc,
            client: OptionalCell::empty(),
            time: Cell::new(DateTimeValues {
                year: 0,
                month: Month::January,
                day: 1,
                day_of_week: DayOfWeek::Sunday,
                hour: 0,
                minute: 0,
                seconds: 0,
            }),
            deferred_call: DeferredCall::new(),
            deferred_call_task: OptionalCell::empty(),
        }
    }
    pub fn initialize_clock(&self) -> Result<(), ErrorCode> {
        // Initialize the RTC clock source and enables peripheral access.
        // Why this specific sequence:
        // 1. Enable the PWR clock: The Power controller manages access to the backup domain
        // 2. We need to disable the Backup Domain's Write Protection since the write
        //    protection resets to default state every time we boot up the board. We
        //    want no write protection on the Backup Domain so we can access the RTC config
        //    registers and time registers.
        // 3. Enable the APB3 bus clock so our code can send commands to the RTC registers
        // 4. Turn on the LSI Oscillator, which is a slow internal clock we use for low power consumption.
        //    In ultra low power mode the HSE/HSI are disabled. We use LSI to keep track of time regardless
        //    if the board is in stand-by, lower power mode or not.
        // 5. After the LSI Oscillator stabilizes we select it as the RTC clock source. There can be
        //    a situation where the oscillator doesn't stabilize, in that case my approach is to not hang the kernel
        //    and just timeout.
        let pwr = PWR_BASE;
        self.rcc.enable_ahb3_pwrclk();
        pwr.pwr_dbpr.modify(PWR_DBPR::DBP::SET);
        self.rcc.enable_apb3_bus_clk();

        // Enable LSI oscillator.
        self.rcc.enable_lsi();
        // Magic number large enough to prevent kernel hanging if the oscillator fails to stabilize
        let mut cycle_counter = 100000;
        while !self.rcc.is_lsi_ready() && cycle_counter > 0 {
            cycle_counter -= 1;
        }
        if cycle_counter <= 0 {
            return Err(ErrorCode::FAIL);
        }
        self.rcc.select_rtc_source_lsi();
        self.rcc.enable_rtc();
        Ok(())
    }
    #[inline(never)]
    // This function is marked as #[inline(never)] in order to aid with the debugging process when
    // disabling board memory protection
    /// Bypass write protection.
    fn bypass_write_protection(&self) {
        // Unlock write acces to RTC_WPR to be able to access RTC calibration and initialization registers
        // Writing an incorrect key reactivated the write protection
        // RM0456 - Section 63.3.11, Page 2596
        self.registers.rtc_wpr.write(RTC_WPR::KEY.val(0xCA));
        self.registers.rtc_wpr.write(RTC_WPR::KEY.val(0x53));
    }
    #[inline(never)]
    // This function is marked as #[inline(never)] in order to aid with the debugging process when
    // enabling board memory protection
    /// Reactivate write protection.
    fn enable_write_protection(&self) {
        // Just write a random value to activate write protection.
        self.registers.rtc_wpr.write(RTC_WPR::KEY.val(0x42));
    }

    pub fn enter_init_mode(&self) -> Result<(), ErrorCode> {
        // We need to bypass the hardware write protection before we can modify
        // any of the RTC control registers.
        // We then request the RTC to enter initialization mode.
        self.bypass_write_protection();
        self.registers.rtc_icsr.modify(RTC_ICSR::INIT::SET);
        let mut cycle_counter = 100000;
        while self.registers.rtc_icsr.read(RTC_ICSR::INITF) != 1 && cycle_counter > 0 {
            cycle_counter -= 1;
        }
        if cycle_counter <= 0 {
            return Err(ErrorCode::FAIL);
        }

        Ok(())
    }
    pub fn init_mode(&self) -> Result<(), ErrorCode> {
        self.enter_init_mode()?;
        // Run clock at 1 Hz
        // The formula is f_RTC = f_CLK / ((PREDIV_A + 1) * (PREDIV_S + 1))
        // 32.768 kHz / (128 * 256) = 1 Hz.
        // It is recommended to max out PREDIV_A which is an asynchronous prescaler.
        // Doing this reduces the power consumption since we minimize the frequency of the clock
        // that goes into the synchronous part of the RTC.
        self.registers.rtc_prer.modify(RTC_PRER::PREDIV_A.val(127));
        self.registers.rtc_prer.modify(RTC_PRER::PREDIV_S.val(255));

        // Enter BCD mode.
        self.registers.rtc_icsr.modify(RTC_ICSR::BIN::CLEAR);

        // Set time format to 24h. We first clear the AM/PM notation bit.
        self.registers.rtc_tr.write(RTC_TR::PM::CLEAR);
        self.registers.rtc_cr.modify(RTC_CR::FMT::CLEAR);

        let datetime = date_time::DateTimeValues {
            year: 0,
            month: Month::January,
            day: 1,
            day_of_week: DayOfWeek::Monday,

            hour: 0,
            minute: 0,
            seconds: 0,
        };
        self.date_time_setup(datetime)?;

        self.exit_init_mode()?;
        Ok(())
    }
    pub fn exit_init_mode(&self) -> Result<(), ErrorCode> {
        // To exit initialization mode we first need to request exiting it
        // and then re-enable write protection so we don't accidentaly change anything.
        self.registers.rtc_icsr.modify(RTC_ICSR::INIT::CLEAR);
        let mut cycle_counter = 100000;
        while self.registers.rtc_icsr.read(RTC_ICSR::INITF) != 0 && cycle_counter > 0 {
            cycle_counter -= 1;
        }
        if cycle_counter <= 0 {
            return Err(ErrorCode::FAIL);
        }

        self.enable_write_protection();
        Ok(())
    }

    fn date_time_setup(&self, datetime: date_time::DateTimeValues) -> Result<(), ErrorCode> {
        // The hardware RTC registers store time and date values in BCD format, not integers.
        // Since the digits are represented in BCD format we divide by 10 to get the tens digit and
        // use modulo the get the units digits for the respective bitfield.
        let month_num = Rtc::month_into_u32(datetime.month);
        let dotw_num = Rtc::dotw_into_u32(datetime.day_of_week);

        if !(datetime.day >= 1 && datetime.day <= 31) {
            return Err(ErrorCode::INVAL);
        }
        if !(datetime.hour <= 23) {
            return Err(ErrorCode::INVAL);
        }
        if !(datetime.minute <= 59) {
            return Err(ErrorCode::INVAL);
        }
        if !(datetime.seconds <= 59) {
            return Err(ErrorCode::INVAL);
        }
        self.registers.rtc_dr.modify(
            RTC_DR::YT.val((datetime.year % 100) as u32 / 10)
                + RTC_DR::YU.val((datetime.year % 100) as u32 % 10)
                + RTC_DR::MT.val(month_num / 10)
                + RTC_DR::MU.val(month_num % 10)
                + RTC_DR::DT.val(datetime.day as u32 / 10)
                + RTC_DR::DU.val(datetime.day as u32 % 10)
                + RTC_DR::WDU.val(dotw_num),
        );

        self.registers.rtc_tr.modify(
            RTC_TR::HT.val(datetime.hour as u32 / 10)
                + RTC_TR::HU.val(datetime.hour as u32 % 10)
                + RTC_TR::MNT.val(datetime.minute as u32 / 10)
                + RTC_TR::MNU.val(datetime.minute as u32 % 10)
                + RTC_TR::ST.val(datetime.seconds as u32 / 10)
                + RTC_TR::SU.val(datetime.seconds as u32 % 10),
        );

        Ok(())
    }

    fn dotw_try_from_u32(dotw: u32) -> Result<DayOfWeek, ErrorCode> {
        match dotw {
            1 => Ok(DayOfWeek::Monday),
            2 => Ok(DayOfWeek::Tuesday),
            3 => Ok(DayOfWeek::Wednesday),
            4 => Ok(DayOfWeek::Thursday),
            5 => Ok(DayOfWeek::Friday),
            6 => Ok(DayOfWeek::Saturday),
            7 => Ok(DayOfWeek::Sunday),
            _ => Err(ErrorCode::INVAL),
        }
    }

    fn dotw_into_u32(dotw: DayOfWeek) -> u32 {
        match dotw {
            DayOfWeek::Monday => 1,
            DayOfWeek::Tuesday => 2,
            DayOfWeek::Wednesday => 3,
            DayOfWeek::Thursday => 4,
            DayOfWeek::Friday => 5,
            DayOfWeek::Saturday => 6,
            DayOfWeek::Sunday => 7,
        }
    }

    fn month_try_from_u32(month_num: u32) -> Result<Month, ErrorCode> {
        match month_num {
            1 => Ok(Month::January),
            2 => Ok(Month::February),
            3 => Ok(Month::March),
            4 => Ok(Month::April),
            5 => Ok(Month::May),
            6 => Ok(Month::June),
            7 => Ok(Month::July),
            8 => Ok(Month::August),
            9 => Ok(Month::September),
            10 => Ok(Month::October),
            11 => Ok(Month::November),
            12 => Ok(Month::December),
            _ => Err(ErrorCode::INVAL),
        }
    }

    fn month_into_u32(month: Month) -> u32 {
        match month {
            Month::January => 1,
            Month::February => 2,
            Month::March => 3,
            Month::April => 4,
            Month::May => 5,
            Month::June => 6,
            Month::July => 7,
            Month::August => 8,
            Month::September => 9,
            Month::October => 10,
            Month::November => 11,
            Month::December => 12,
        }
    }
}

impl<'a> date_time::DateTime<'a> for Rtc<'a> {
    fn get_date_time(&self) -> Result<(), ErrorCode> {
        // Start an async read of current date and time:
        // 1. Verify if the driver isn't busy.
        // 2. Read the calendar registers (TR and DR) and converts their BCD values into integers.
        // 3. The time and date are then stored in the time cell and it is later retrieved via the deferred call
        //    handler and passed to the client.
        // 4. Set a deferred call task and schedules it to notify client.
        match self.deferred_call_task.take() {
            Some(DeferredCallTask::Set) => {
                self.deferred_call_task.insert(Some(DeferredCallTask::Set));
                return Err(ErrorCode::BUSY);
            }
            Some(DeferredCallTask::Get) => {
                self.deferred_call_task.insert(Some(DeferredCallTask::Get));
                return Err(ErrorCode::ALREADY);
            }
            _ => (),
        }

        let month_num =
            self.registers.rtc_dr.read(RTC_DR::MT) * 10 + self.registers.rtc_dr.read(RTC_DR::MU);
        let month_name = Rtc::month_try_from_u32(month_num)?;

        let dotw_num = self.registers.rtc_dr.read(RTC_DR::WDU);
        let dotw_name = Rtc::dotw_try_from_u32(dotw_num)?;

        let datetime = date_time::DateTimeValues {
            hour: (self.registers.rtc_tr.read(RTC_TR::HT) * 10
                + self.registers.rtc_tr.read(RTC_TR::HU)) as u8,
            minute: (self.registers.rtc_tr.read(RTC_TR::MNT) * 10
                + self.registers.rtc_tr.read(RTC_TR::MNU)) as u8,
            seconds: (self.registers.rtc_tr.read(RTC_TR::ST) * 10
                + self.registers.rtc_tr.read(RTC_TR::SU)) as u8,

            year: (self.registers.rtc_dr.read(RTC_DR::YT) * 10
                + self.registers.rtc_dr.read(RTC_DR::YU)) as u16,
            month: month_name,
            day: (self.registers.rtc_dr.read(RTC_DR::DT) * 10
                + self.registers.rtc_dr.read(RTC_DR::DU)) as u8,
            day_of_week: dotw_name,
        };

        self.time.replace(datetime);

        self.deferred_call_task.insert(Some(DeferredCallTask::Get));
        self.deferred_call.set();

        Ok(())
    }

    fn set_date_time(&self, date_time: date_time::DateTimeValues) -> Result<(), ErrorCode> {
        // Start an async write to update RTC date and time registers:
        // 1. Verify if the driver isn't busy.
        // 2. Unlock the wp and enters init mode (which is necessary for writing to RTC date/time registers).
        // 3. Convert the provided date and time values to BCD format and writes them to the registers (TR and DR).
        // 4. Exit initialization mode, resume clk and reactivate wp.
        // 5. Set a deferred call task and schedule it to notify client.
        match self.deferred_call_task.take() {
            Some(DeferredCallTask::Set) => {
                self.deferred_call_task.insert(Some(DeferredCallTask::Set));
                return Err(ErrorCode::ALREADY);
            }
            Some(DeferredCallTask::Get) => {
                self.deferred_call_task.insert(Some(DeferredCallTask::Get));
                return Err(ErrorCode::BUSY);
            }
            _ => (),
        }

        self.enter_init_mode()?;
        self.date_time_setup(date_time)?;
        self.exit_init_mode()?;

        self.deferred_call_task.insert(Some(DeferredCallTask::Set));
        self.deferred_call.set();
        Ok(())
    }

    fn set_client(&self, client: &'a dyn DateTimeClient) {
        self.client.set(client);
    }
}
