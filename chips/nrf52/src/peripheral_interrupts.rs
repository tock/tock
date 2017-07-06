// https://github.com/apache/mynewt-core/blob/master/hw/mcu/nordic/src/ext/nRF5_SDK_11.0.0_89a8197/components/device/nrf52.h  

/*
  POWER_CLOCK_IRQn              =   0,              /*!<   0  POWER_CLOCK                                                      */
  RADIO_IRQn                    =   1,              /*!<   1  RADIO                                                            */
  UARTE0_UART0_IRQn             =   2,              /*!<   2  UARTE0_UART0                                                     */
  SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0_IRQn=   3,      /*!<   3  SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0                                */
  SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1_IRQn=   4,      /*!<   4  SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1                                */
  NFCT_IRQn                     =   5,              /*!<   5  NFCT                                                             */
  GPIOTE_IRQn                   =   6,              /*!<   6  GPIOTE                                                           */
  SAADC_IRQn                    =   7,              /*!<   7  SAADC                                                            */
  TIMER0_IRQn                   =   8,              /*!<   8  TIMER0                                                           */
  TIMER1_IRQn                   =   9,              /*!<   9  TIMER1                                                           */
  TIMER2_IRQn                   =  10,              /*!<  10  TIMER2                                                           */
  RTC0_IRQn                     =  11,              /*!<  11  RTC0                                                             */
  TEMP_IRQn                     =  12,              /*!<  12  TEMP                                                             */
  RNG_IRQn                      =  13,              /*!<  13  RNG                                                              */
  ECB_IRQn                      =  14,              /*!<  14  ECB                                                              */
  CCM_AAR_IRQn                  =  15,              /*!<  15  CCM_AAR                                                          */
  WDT_IRQn                      =  16,              /*!<  16  WDT                                                              */
  RTC1_IRQn                     =  17,              /*!<  17  RTC1                                                             */
  QDEC_IRQn                     =  18,              /*!<  18  QDEC                                                             */
  COMP_LPCOMP_IRQn              =  19,              /*!<  19  COMP_LPCOMP                                                      */
  SWI0_EGU0_IRQn                =  20,              /*!<  20  SWI0_EGU0                                                        */
  SWI1_EGU1_IRQn                =  21,              /*!<  21  SWI1_EGU1                                                        */
  SWI2_EGU2_IRQn                =  22,              /*!<  22  SWI2_EGU2                                                        */
  SWI3_EGU3_IRQn                =  23,              /*!<  23  SWI3_EGU3                                                        */
  SWI4_EGU4_IRQn                =  24,              /*!<  24  SWI4_EGU4                                                        */
  SWI5_EGU5_IRQn                =  25,              /*!<  25  SWI5_EGU5                                                        */
  TIMER3_IRQn                   =  26,              /*!<  26  TIMER3                                                           */
  TIMER4_IRQn                   =  27,              /*!<  27  TIMER4                                                           */
  PWM0_IRQn                     =  28,              /*!<  28  PWM0                                                             */
  PDM_IRQn                      =  29,              /*!<  29  PDM                                                              */
  MWU_IRQn                      =  32,              /*!<  32  MWU                                                              */
  PWM1_IRQn                     =  33,              /*!<  33  PWM1                                                             */
  PWM2_IRQn                     =  34,              /*!<  34  PWM2                                                             */
  SPIM2_SPIS2_SPI2_IRQn         =  35,              /*!<  35  SPIM2_SPIS2_SPI2                                                 */
  RTC2_IRQn                     =  36,              /*!<  36  RTC2                                                             */
  I2S_IRQn                      =  37,              /*!<  37  I2S                                                              */
  FPU_IRQn                      =  38               /*!<  38  FPU                                                              */
*/


#[allow(non_camel_case_types,dead_code)]
#[derive(Copy,Clone)]
pub enum NvicIdx {
    POWER_CLOCK = 0,
    RADIO = 1,
    UART0 = 2,
    SPI0_TWI0 = 3,
    SPI1_TWI1 = 4,
    NFCT = 5,
    GPIOTE = 6,
    ADC = 7,
    TIMER0 = 8,
    TIMER1 = 9,
    TIMER2 = 10,
    RTC0 = 11,
    TEMP = 12,
    RNG = 13,
    ECB = 14,
    CCM_AAR = 15,
    WDT = 16,
    RTC1 = 17,
    QDEC = 18,
    LPCOMP = 19,
    SWI0 = 20,
    SWI1 = 21,
    SWI2 = 22,
    SWI3 = 23,
    SWI4 = 24,
    SWI5 = 25,
    TIMER3 = 26,
    TIMER4 = 27,
    PWM0 = 28,
    PDM = 29,
    MWU = 32,
    PWM1 = 33,
    PWM2 = 34,
    SPIM2_SPIS2_SPI2 = 35,
    RTC2 = 36,
    I2S = 37,
    FPU = 38,
}
