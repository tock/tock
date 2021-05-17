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

pub const SPI_DEVICERXF: u32 = 65;
pub const SPI_DEVICERXLVL: u32 = 66;
pub const SPI_DEVICETXLVL: u32 = 67;
pub const SPI_DEVICERXERR: u32 = 68;
pub const SPI_DEVICERXOVERFLOW: u32 = 69;
pub const SPI_DEVICETXUNDERFLOW: u32 = 70;
pub const SPI_HOST0ERROR: u32 = 71;
pub const SPI_HOST0SPIEVENT: u32 = 72;
pub const SPI_HOST1ERROR: u32 = 73;
pub const SPI_HOST1SPIEVENT: u32 = 74;

pub const I2C0_FMTWATERMARK: u32 = 75;
pub const I2C0_RXWATERMARK: u32 = 76;
pub const I2C0_FMTOVERFLOW: u32 = 77;
pub const I2C0_RXOVERFLOW: u32 = 78;
pub const I2C0_NAK: u32 = 79;
pub const I2C0_SCLINTERFERENCE: u32 = 80;
pub const I2C0_SDAINTERFERENCE: u32 = 81;
pub const I2C0_STRETCHTIMEOUT: u32 = 82;
pub const I2C0_SDAUNSTABLE: u32 = 83;
pub const I2C0_TRANSCOMPLETE: u32 = 84;
pub const I2C0_TXEMPTY: u32 = 85;
pub const I2C0_TXNONEMPTY: u32 = 86;
pub const I2C0_TXOVERFLOW: u32 = 87;
pub const I2C0_ACQOVERFLOW: u32 = 88;
pub const I2C0_ACKSTOP: u32 = 89;
pub const I2C0_HOSTTIMEOUT: u32 = 90;

pub const I2C1_FMTWATERMARK: u32 = 91;
pub const I2C1_RXWATERMARK: u32 = 92;
pub const I2C1_FMTOVERFLOW: u32 = 93;
pub const I2C1_RXOVERFLOW: u32 = 94;
pub const I2C1_NAK: u32 = 95;
pub const I2C1_SCLINTERFERENCE: u32 = 96;
pub const I2C1_SDAINTERFERENCE: u32 = 97;
pub const I2C1_STRETCHTIMEOUT: u32 = 98;
pub const I2C1_SDAUNSTABLE: u32 = 99;
pub const I2C1_TRANSCOMPLETE: u32 = 100;
pub const I2C1_TXEMPTY: u32 = 101;
pub const I2C1_TXNONEMPTY: u32 = 102;
pub const I2C1_TXOVERFLOW: u32 = 103;
pub const I2C1_ACQOVERFLOW: u32 = 104;
pub const I2C1_ACKSTOP: u32 = 105;
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
pub const I2C2_TRANSCOMPLETE: u32 = 116;
pub const I2C2_TXEMPTY: u32 = 117;
pub const I2C2_TXNONEMPTY: u32 = 118;
pub const I2C2_TXOVERFLOW: u32 = 119;
pub const I2C2_ACQOVERFLOW: u32 = 120;
pub const I2C2_ACKSTOP: u32 = 121;
pub const I2C2_HOSTTIMEOUT: u32 = 122;

pub const PATTGENDONECH0: u32 = 123;
pub const PATTGENDONECH1: u32 = 124;

pub const RVTIMERTIMEREXPIRED0_0: u32 = 125;

pub const USBDEV_PKTRECEIVED: u32 = 126;
pub const USBDEV_PKTSENT: u32 = 127;
pub const USBDEV_DISCONNECTED: u32 = 128;
pub const USBDEV_HOSTLOST: u32 = 129;
pub const USBDEV_LINKRESET: u32 = 130;
pub const USBDEV_LINKSUSPEND: u32 = 131;
pub const USBDEV_LINKRESUME: u32 = 132;
pub const USBDEV_AVEMPTY: u32 = 133;
pub const USBDEV_RXFULL: u32 = 134;
pub const USBDEV_AVOVERFLOW: u32 = 135;
pub const USBDEV_LINKINERR: u32 = 136;
pub const USBDEV_RXCRCERR: u32 = 137;
pub const USBDEV_RXPIDERR: u32 = 138;
pub const USBDEV_RXBITSTUFFERR: u32 = 139;
pub const USBDEV_FRAME: u32 = 140;
pub const USBDEV_CONNECTED: u32 = 141;
pub const USBDEV_LINKOUTERR: u32 = 142;

pub const OTP_CTRLOTPOPERATIONDONE: u32 = 143;
pub const OTP_CTRLOTPERROR: u32 = 144;

pub const ALERTHANDLERCLASSA: u32 = 145;
pub const ALERTHANDLERCLASSB: u32 = 146;
pub const ALERTHANDLERCLASSC: u32 = 147;
pub const ALERTHANDLERCLASSD: u32 = 148;

pub const PWRMGRAONWAKEUP: u32 = 149;

pub const ADCCTRLAONDEBUGCABLE: u32 = 150;
pub const AONTIMERAONWKUPTIMEREXPIRED: u32 = 151;
pub const AONTIMERAONWDOGTIMERBARK: u32 = 152;

pub const FLASHCTRL_PROGEMPTY: u32 = 153;
pub const FLASHCTRL_PROGLVL: u32 = 154;
pub const FLASHCTRL_RDFULL: u32 = 155;
pub const FLASHCTRL_RDLVL: u32 = 156;
pub const FLASHCTRL_OPDONE: u32 = 157;

pub const HMAC_HMACDONE: u32 = 158;
pub const HMAC_FIFOEMPTY: u32 = 159;
pub const HMAC_HMACERR: u32 = 160;

pub const KMAC_KMACDONE: u32 = 161;
pub const KMAC_FIFOEMPTY: u32 = 162;
pub const KMAC_KMACERR: u32 = 163;

pub const KEYMGR_OPDONE: u32 = 164;

pub const CSRNG_CSCMDREQDONE: u32 = 165;
pub const CSRNG_CSENTROPYREQ: u32 = 166;
pub const CSRNG_CSHWINSTEXC: u32 = 167;
pub const CSRNG_CSFATALERR: u32 = 168;

pub const ENTROPYSRC_ESENTROPYVALID: u32 = 169;
pub const ENTROPYSRC_ESHEALTHTESTFAILED: u32 = 170;
pub const ENTROPYSRC_ESFATALERR: u32 = 171;

pub const EDN0EDN_CMDREQDONE: u32 = 172;
pub const EDN0EDN_FATALERR: u32 = 173;
pub const EDN1EDN_CMDREQDONE: u32 = 174;
pub const EDN1EDN_FATALERR: u32 = 175;

pub const OTBN_DONE: u32 = 176;

pub const LAST: u32 = 176;
