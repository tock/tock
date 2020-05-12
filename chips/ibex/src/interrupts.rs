//! Named interrupts for the Ibex core.

#![allow(dead_code)]

pub const USBDEV_CONNECTED: u32 = 0x4f;
pub const USBDEV_FRAME: u32 = 0x4e;
pub const USBDEV_RX_BITSTUFF_ERR: u32 = 0x4d;
pub const USBDEV_RX_PID_ERR: u32 = 0x4c;
pub const USBDEV_RX_CRC_ERR: u32 = 0x4b;
pub const USBDEV_LINK_IN_ERR: u32 = 0x4a;
pub const USBDEV_AV_OVERFLOW: u32 = 0x49;
pub const USBDEV_RX_FULL: u32 = 0x48;
pub const USBDEV_AV_EMPTY: u32 = 0x47;
pub const USBDEV_LINK_RESUME: u32 = 0x46;
pub const USBDEV_LINK_SUSPEND: u32 = 0x45;
pub const USBDEV_LINK_RESET: u32 = 0x44;
pub const USBDEV_HOST_LOST: u32 = 0x43;
pub const USBDEV_DISCONNECTED: u32 = 0x42;
pub const USBDEV_PKT_SENT: u32 = 0x41;
pub const USBDEV_PKT_RECEIVED: u32 = 0x40;

pub const NMI_ESC3: u32 = 0x3f;
pub const NMI_ESC2: u32 = 0x3e;
pub const NMI_ESC1: u32 = 0x3d;
pub const NMI_ESC0: u32 = 0x3c;

pub const ALERT_CLASSD: u32 = 0x3b;
pub const ALERT_CLASSC: u32 = 0x3a;
pub const ALERT_CLASSB: u32 = 0x39;
pub const ALERT_CLASSA: u32 = 0x38;

pub const HMAC_HMAC_ERR: u32 = 0x37;
pub const HMAC_FIFO_FULL: u32 = 0x36;
pub const HMAC_HMAC_DONE: u32 = 0x35;

pub const FLASH_OP_ERROR: u32 = 0x34;
pub const FLASH_OP_DONE: u32 = 0x33;
pub const FLASH_RD_LVL: u32 = 0x32;
pub const FLASH_RD_FULL: u32 = 0x31;
pub const FLASH_PROG_LVL: u32 = 0x30;
pub const FLASH_PROG_EMPTY: u32 = 0x2f;

pub const SPI_TXUNDERFLOW: u32 = 0x2e;
pub const SPI_RXOVERFLOW: u32 = 0x2d;
pub const SPI_RXERR: u32 = 0x2c;
pub const SPI_TXLVL: u32 = 0x2b;
pub const SPI_RXLVL: u32 = 0x2a;
pub const SPI_RXF: u32 = 0x29;

pub const UART_RX_PARITY_ERR: u32 = 0x28;
pub const UART_RX_TIMEOUT: u32 = 0x27;
pub const UART_RX_BREAK_ERR: u32 = 0x26;
pub const UART_RX_FRAME_ERR: u32 = 0x25;
pub const UART_RX_OVERFLOW: u32 = 0x24;
pub const UART_TX_EMPTY: u32 = 0x23;
pub const UART_RX_WATERMARK: u32 = 0x22;
pub const UART_TX_WATERMARK: u32 = 0x21;

pub const GPIO_PIN31: u32 = 0x20;
pub const GPIO_PIN30: u32 = 0x1f;
pub const GPIO_PIN29: u32 = 0x1e;
pub const GPIO_PIN28: u32 = 0x1d;
pub const GPIO_PIN27: u32 = 0x1c;
pub const GPIO_PIN26: u32 = 0x1b;
pub const GPIO_PIN25: u32 = 0x1a;
pub const GPIO_PIN24: u32 = 0x19;
pub const GPIO_PIN23: u32 = 0x18;
pub const GPIO_PIN22: u32 = 0x17;
pub const GPIO_PIN21: u32 = 0x16;
pub const GPIO_PIN20: u32 = 0x15;
pub const GPIO_PIN19: u32 = 0x14;
pub const GPIO_PIN18: u32 = 0x13;
pub const GPIO_PIN17: u32 = 0x12;
pub const GPIO_PIN16: u32 = 0x11;
pub const GPIO_PIN15: u32 = 0x10;
pub const GPIO_PIN14: u32 = 0x0f;
pub const GPIO_PIN13: u32 = 0x0e;
pub const GPIO_PIN12: u32 = 0x0d;
pub const GPIO_PIN11: u32 = 0x0c;
pub const GPIO_PIN10: u32 = 0x0b;
pub const GPIO_PIN9: u32 = 0x0a;
pub const GPIO_PIN8: u32 = 0x09;
pub const GPIO_PIN7: u32 = 0x08;
pub const GPIO_PIN6: u32 = 0x07;
pub const GPIO_PIN5: u32 = 0x06;
pub const GPIO_PIN4: u32 = 0x05;
pub const GPIO_PIN3: u32 = 0x04;
pub const GPIO_PIN2: u32 = 0x03;
pub const GPIO_PIN1: u32 = 0x02;
pub const GPIO_PIN0: u32 = 0x01;

// Per the PLIC docs: ID 0 is reserved and represents no interrupt.
pub const NO_INTERRUPT: u32 = 0x00;
