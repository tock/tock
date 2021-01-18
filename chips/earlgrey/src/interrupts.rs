//! Named interrupts for the Earl Grey chip.

#![allow(dead_code)]

pub const PWRMGRWAKEUP: u32 = 0x50;

pub const NO_INTERRUPT: u32 = 0;

pub const GPIO_PIN0: u32 = 1;
pub const GPIO_PIN1: u32 = 2;
pub const GPIO_PIN2: u32 = 3;
pub const GPIO_PIN3: u32 = 4;
pub const GPIO_PIN4: u32 = 5;
pub const GPIO_PIN5: u32 = 6;
pub const GPIO_PIN6: u32 = 7;
pub const GPIO_PIN7: u32 = 8;
pub const GPIO_PIN8: u32 = 9;
pub const GPIO_PIN9: u32 = 10;
pub const GPIO_PIN10: u32 = 11;
pub const GPIO_PIN11: u32 = 12;
pub const GPIO_PIN12: u32 = 13;
pub const GPIO_PIN13: u32 = 14;
pub const GPIO_PIN14: u32 = 15;
pub const GPIO_PIN15: u32 = 16;
pub const GPIO_PIN16: u32 = 17;
pub const GPIO_PIN17: u32 = 18;
pub const GPIO_PIN18: u32 = 19;
pub const GPIO_PIN19: u32 = 20;
pub const GPIO_PIN20: u32 = 21;
pub const GPIO_PIN21: u32 = 22;
pub const GPIO_PIN22: u32 = 23;
pub const GPIO_PIN23: u32 = 24;
pub const GPIO_PIN24: u32 = 25;
pub const GPIO_PIN25: u32 = 26;
pub const GPIO_PIN26: u32 = 27;
pub const GPIO_PIN27: u32 = 28;
pub const GPIO_PIN28: u32 = 29;
pub const GPIO_PIN29: u32 = 30;
pub const GPIO_PIN30: u32 = 31;
pub const GPIO_PIN31: u32 = 32;

pub const UART_TX_WATERMARK: u32 = 33;
pub const UART_RX_WATERMARK: u32 = 34;
pub const UART_TX_EMPTY: u32 = 35;
pub const UART_RX_OVERFLOW: u32 = 36;
pub const UART_RX_FRAME_ERR: u32 = 37;
pub const UART_RX_BREAK_ERR: u32 = 38;
pub const UART_RX_TIMEOUT: u32 = 39;
pub const UART_RX_PARITY_ERR: u32 = 40;

pub const SPI_RXF: u32 = 41;
pub const SPI_RXLVL: u32 = 42;
pub const SPI_TXLVL: u32 = 43;
pub const SPI_RXERR: u32 = 44;
pub const SPI_RXOVERFLOW: u32 = 45;
pub const SPI_TXUNDERFLOW: u32 = 46;

pub const FLASH_PROG_EMPTY: u32 = 47;
pub const FLASH_PROG_LVL: u32 = 48;
pub const FLASH_RD_FULL: u32 = 49;
pub const FLASH_RD_LVL: u32 = 50;
pub const FLASH_OP_DONE: u32 = 51;
pub const FLASH_OP_ERROR: u32 = 52;

pub const HMAC_HMAC_DONE: u32 = 53;
pub const HMAC_FIFO_EMPTY: u32 = 54;
pub const HMAC_HMAC_ERR: u32 = 55;

pub const ALERT_CLASSA: u32 = 56;
pub const ALERT_CLASSB: u32 = 57;
pub const ALERT_CLASSC: u32 = 58;
pub const ALERT_CLASSD: u32 = 59;

pub const NMI_ESC0: u32 = 60;
pub const NMI_ESC1: u32 = 61;
pub const NMI_ESC2: u32 = 62;

pub const USBDEV_PKT_RECEIVED: u32 = 63;
pub const USBDEV_PKT_SENT: u32 = 64;
pub const USBDEV_DISCONNECTED: u32 = 65;
pub const USBDEV_HOST_LOST: u32 = 66;
pub const USBDEV_LINK_RESET: u32 = 67;
pub const USBDEV_LINK_SUSPEND: u32 = 68;
pub const USBDEV_LINK_RESUME: u32 = 69;
pub const USBDEV_AV_EMPTY: u32 = 70;
pub const USBDEV_RX_FULL: u32 = 71;
pub const USBDEV_AV_OVERFLOW: u32 = 72;
pub const USBDEV_LINK_IN_ERR: u32 = 73;
pub const USBDEV_RX_CRC_ERR: u32 = 74;
pub const USBDEV_RX_PID_ERR: u32 = 75;
pub const USBDEV_RX_BITSTUFF_ERR: u32 = 76;
pub const USBDEV_FRAME: u32 = 77;
pub const USBDEV_CONNECTED: u32 = 78;
pub const USBDEV_LINK_OUT_ERR: u32 = 79;

pub const PWRMGR_WAKEUP: u32 = 80;

pub const OTBN_DONE: u32 = 81;

pub const KEYMGR_OP_DONE: u32 = 82;
pub const KEYMGR_ERR: u32 = 83;

pub const KMAC_KMAC_DONE: u32 = 84;
pub const KMAC_FIFO_EMPTY: u32 = 85;
pub const KMAC_KMAC_ERR: u32 = 86;

pub const PATTGEN_DONE_CH0: u32 = 87;
pub const PATTGEN_DONE_CH1: u32 = 88;

pub const I2C_FMT_WATERMARK: u32 = 89;
pub const I2C_RX_WATERMARK: u32 = 90;
pub const I2C_FMT_OVERFLOW: u32 = 91;
pub const I2C_RX_OVERFLOW: u32 = 92;
pub const I2C_NAK: u32 = 93;
pub const I2C_SCL_INTERFERENCE: u32 = 94;
pub const I2C_SDA_INTERFERENCE: u32 = 95;
pub const I2C_STRETCH_TIMEOUT: u32 = 96;
pub const I2C_SDA_UNSTABLE: u32 = 97;
pub const I2C_TRANS_COMPLETE: u32 = 98;
pub const I2C_TX_EMPTY: u32 = 99;
pub const I2C_TX_NONEMPTY: u32 = 100;
pub const I2C_TX_OVERFLOW: u32 = 101;
pub const I2C_ACQ_OVERFLOW: u32 = 102;
pub const I2C_ACK_STOP: u32 = 103;
