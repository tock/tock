// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Named interrupts for the Earl Grey chip.

#![allow(dead_code)]

pub const NONE: u32 = 0;

pub const UART0_TX_WATERMARK: u32 = 1;
pub const UART0_RX_WATERMARK: u32 = 2;
pub const UART0_TX_EMPTY: u32 = 3;
pub const UART0_RX_OVERFLOW: u32 = 4;
pub const UART0_RX_FRAMEERR: u32 = 5;
pub const UART0_RX_BREAKERR: u32 = 6;
pub const UART0_RX_TIMEOUT: u32 = 7;
pub const UART0_RX_PARITYERR: u32 = 8;

pub const UART1_TX_WATERMARK: u32 = 9;
pub const UART1_RX_WATERMARK: u32 = 10;
pub const UART1_TX_EMPTY: u32 = 11;
pub const UART1_RX_OVERFLOW: u32 = 12;
pub const UART1_RX_FRAMEERR: u32 = 13;
pub const UART1_RX_BREAKERR: u32 = 14;
pub const UART1_RX_TIMEOUT: u32 = 15;
pub const UART1_RX_PARITYERR: u32 = 16;

pub const UART2_TX_WATERMARK: u32 = 17;
pub const UART2_RX_WATERMARK: u32 = 18;
pub const UART2_TX_EMPTY: u32 = 19;
pub const UART2_RX_OVERFLOW: u32 = 20;
pub const UART2_RX_FRAMEERR: u32 = 21;
pub const UART2_RX_BREAKERR: u32 = 22;
pub const UART2_RX_TIMEOUT: u32 = 23;
pub const UART2_RX_PARITYERR: u32 = 24;

pub const UART3_TX_WATERMARK: u32 = 25;
pub const UART3_RX_WATERMARK: u32 = 26;
pub const UART3_TX_EMPTY: u32 = 27;
pub const UART3_RX_OVERFLOW: u32 = 28;
pub const UART3_RX_FRAMEERR: u32 = 29;
pub const UART3_RX_BREAKERR: u32 = 30;
pub const UART3_RX_TIMEOUT: u32 = 31;
pub const UART3_RX_PARITYERR: u32 = 32;

pub const GPIO_PIN0: u32 = 33;
pub const GPIO_PIN1: u32 = 34;
pub const GPIO_PIN2: u32 = 35;
pub const GPIO_PIN3: u32 = 36;
pub const GPIO_PIN4: u32 = 37;
pub const GPIO_PIN5: u32 = 38;
pub const GPIO_PIN6: u32 = 39;
pub const GPIO_PIN7: u32 = 40;
pub const GPIO_PIN8: u32 = 41;
pub const GPIO_PIN9: u32 = 42;
pub const GPIO_PIN10: u32 = 43;
pub const GPIO_PIN11: u32 = 44;
pub const GPIO_PIN12: u32 = 45;
pub const GPIO_PIN13: u32 = 46;
pub const GPIO_PIN14: u32 = 47;
pub const GPIO_PIN15: u32 = 48;
pub const GPIO_PIN16: u32 = 49;
pub const GPIO_PIN17: u32 = 50;
pub const GPIO_PIN18: u32 = 51;
pub const GPIO_PIN19: u32 = 52;
pub const GPIO_PIN20: u32 = 53;
pub const GPIO_PIN21: u32 = 54;
pub const GPIO_PIN22: u32 = 55;
pub const GPIO_PIN23: u32 = 56;
pub const GPIO_PIN24: u32 = 57;
pub const GPIO_PIN25: u32 = 58;
pub const GPIO_PIN26: u32 = 59;
pub const GPIO_PIN27: u32 = 60;
pub const GPIO_PIN28: u32 = 61;
pub const GPIO_PIN29: u32 = 62;
pub const GPIO_PIN30: u32 = 63;
pub const GPIO_PIN31: u32 = 64;

pub const SPI_DEVICE_GENERICRXFULL: u32 = 65;
pub const SPI_DEVICE_GENERICRXWATERMARK: u32 = 66;
pub const SPI_DEVICE_GENERICTXWATERMARK: u32 = 67;
pub const SPI_DEVICE_GENERICRXERROR: u32 = 68;
pub const SPI_DEVICE_GENERICRXOVERFLOW: u32 = 69;
pub const SPI_DEVICE_GENERICTXUNDERFLOW: u32 = 70;
pub const SPI_DEVICE_UPLOADCMDFIFONOTEMPTY: u32 = 71;
pub const SPI_DEVICE_UPLOADPAYLOADNOTEMPTY: u32 = 72;
pub const SPI_DEVICE_UPLOADPAYLOADOVERFLOW: u32 = 73;
pub const SPI_DEVICE_READBUFWATERMARK: u32 = 74;
pub const SPI_DEVICE_READBUFFLIP: u32 = 75;
pub const SPI_DEVICE_TPMHEADERNOTEMPTY: u32 = 76;

pub const I2C0_FMTWATERMARK: u32 = 77;
pub const I2C0_RXWATERMARK: u32 = 78;
pub const I2C0_FMTOVERFLOW: u32 = 79;
pub const I2C0_RXOVERFLOW: u32 = 80;
pub const I2C0_NAK: u32 = 81;
pub const I2C0_SCLINTERFERENCE: u32 = 82;
pub const I2C0_SDAINTERFERENCE: u32 = 83;
pub const I2C0_STRETCHTIMEOUT: u32 = 84;
pub const I2C0_SDAUNSTABLE: u32 = 85;
pub const I2C0_CMDCOMPLETE: u32 = 86;
pub const I2C0_TXSTRETCH: u32 = 87;
pub const I2C0_TXOVERFLOW: u32 = 88;
pub const I2C0_ACQFULL: u32 = 89;
pub const I2C0_UNEXPSTOP: u32 = 90;
pub const I2C0_HOSTTIMEOUT: u32 = 91;

pub const I2C1_FMTWATERMARK: u32 = 92;
pub const I2C1_RXWATERMARK: u32 = 93;
pub const I2C1_FMTOVERFLOW: u32 = 94;
pub const I2C1_RXOVERFLOW: u32 = 95;
pub const I2C1_NAK: u32 = 96;
pub const I2C1_SCLINTERFERENCE: u32 = 97;
pub const I2C1_SDAINTERFERENCE: u32 = 98;
pub const I2C1_STRETCHTIMEOUT: u32 = 99;
pub const I2C1_SDAUNSTABLE: u32 = 100;
pub const I2C1_CMDCOMPLETE: u32 = 101;
pub const I2C1_TXSTRETCH: u32 = 102;
pub const I2C1_TXOVERFLOW: u32 = 103;
pub const I2C1_ACQFULL: u32 = 104;
pub const I2C1_UNEXPSTOP: u32 = 105;
pub const I2C1_HOSTTIMEOUT: u32 = 106;

pub const I2C2_FMTWATERMARK: u32 = 107;
pub const I2C2_RXWATERMARK: u32 = 108;
pub const I2C2_FMTOVERFLOW: u32 = 109;
pub const I2C2_RXOVERFLOW: u32 = 110;
pub const I2C2_NAK: u32 = 111;
pub const I2C2_SCLINTERFERENCE: u32 = 112;
pub const I2C2_SDAINTERFERENCE: u32 = 113;
pub const I2C2_STRETCHTIMEOUT: u32 = 114;
pub const I2C2_SDAUNSTABLE: u32 = 115;
pub const I2C2_CMDCOMPLETE: u32 = 116;
pub const I2C2_TXSTRETCH: u32 = 117;
pub const I2C2_TXOVERFLOW: u32 = 118;
pub const I2C2_ACQFULL: u32 = 119;
pub const I2C2_UNEXPSTOP: u32 = 120;
pub const I2C2_HOSTTIMEOUT: u32 = 121;

pub const PATTGENDONECH0: u32 = 122;
pub const PATTGENDONECH1: u32 = 123;

pub const RVTIMERTIMEREXPIRED0_0: u32 = 124;

pub const OTPCTRL_OTPOPERATIONDONE: u32 = 125;
pub const OTPCTRL_OTPERROR: u32 = 126;

pub const ALERTHANDLER_CLASSA: u32 = 127;
pub const ALERTHANDLER_CLASSB: u32 = 128;
pub const ALERTHANDLER_CLASSC: u32 = 129;
pub const ALERTHANDLER_CLASSD: u32 = 130;

pub const SPIHOST0_ERROR: u32 = 131;
pub const SPIHOST0_SPIEVENT: u32 = 132;

pub const SPIHOST1_ERROR: u32 = 133;
pub const SPIHOST1_SPIEVENT: u32 = 134;

pub const USBDEV_PKTRECEIVED: u32 = 135;
pub const USBDEV_PKTSENT: u32 = 136;
pub const USBDEV_DISCONNECTED: u32 = 137;
pub const USBDEV_HOSTLOST: u32 = 138;
pub const USBDEV_LINKRESET: u32 = 139;
pub const USBDEV_LINKSUSPEND: u32 = 140;
pub const USBDEV_LINKRESUME: u32 = 141;
pub const USBDEV_AVEMPTY: u32 = 142;
pub const USBDEV_RXFULL: u32 = 143;
pub const USBDEV_AVOVERFLOW: u32 = 144;
pub const USBDEV_LINKINERR: u32 = 145;
pub const USBDEV_RXCRCERR: u32 = 146;
pub const USBDEV_RXPIDERR: u32 = 147;
pub const USBDEV_RXBITSTUFFERR: u32 = 148;
pub const USBDEV_FRAME: u32 = 149;
pub const USBDEV_POWERED: u32 = 150;
pub const USBDEV_LINKOUTERR: u32 = 151;

pub const PWRMGRAONWAKEUP: u32 = 152;
pub const SYSRST_CTRL_AON_SYSRST_CTRL: u32 = 153;
pub const ADC_CTRL_AON_MATCH_DONE: u32 = 154;

pub const AON_TIMER_AON_WKUP_TIMER_EXPIRED: u32 = 155;
pub const AON_TIMER_AON_WDOG_TIMER_BARK: u32 = 156;

pub const SENSOR_CTRL_IO_STATUS_CHANGE: u32 = 157;
pub const SENSOR_CTRL_INIT_STATUS_CHANGE: u32 = 158;

pub const FLASHCTRL_PROGEMPTY: u32 = 159;
pub const FLASHCTRL_PROGLVL: u32 = 160;
pub const FLASHCTRL_RDFULL: u32 = 161;
pub const FLASHCTRL_RDLVL: u32 = 162;
pub const FLASHCTRL_OPDONE: u32 = 163;
pub const FLASHCTRL_CORRERR: u32 = 164;

pub const HMAC_HMACDONE: u32 = 165;
pub const HMAC_FIFOEMPTY: u32 = 166;
pub const HMAC_HMACERR: u32 = 167;

pub const KMAC_KMACDONE: u32 = 168;
pub const KMAC_FIFOEMPTY: u32 = 169;
pub const KMAC_KMACERR: u32 = 170;

pub const OTBN_DONE: u32 = 171;

pub const KEYMGR_OP_DONE: u32 = 172;

pub const CSRNG_CSCMDREQDONE: u32 = 173;
pub const CSRNG_CSENTROPYREQ: u32 = 174;
pub const CSRNG_CSHWINSTEXC: u32 = 175;
pub const CSRNG_CSFATALERR: u32 = 176;

pub const ENTROPY_SRC_ES_ENTROPY_VALID: u32 = 177;
pub const ENTROPY_SRC_ES_HEALTH_TEST_FAILED: u32 = 178;
pub const ENTROPY_SRC_ES_OBSERVE_FIFO_READY: u32 = 179;
pub const ENTROPY_SRC_ES_FATAL_ERR: u32 = 180;

pub const EDN0_EDN_CMD_REQ_DONE: u32 = 181;
pub const EDN0_EDN_FATAL_ERR: u32 = 182;
pub const EDN1_EDN_CMD_REQ_DONE: u32 = 183;
pub const EDN1_EDN_FATAL_ERR: u32 = 184;
// Last valid plic interrupt ID
pub const IRQ_ID_LAST: u32 = 184;
