//! This utility performs a simple test of usb functionality in Tock:
//! It writes bulk data into an eight-byte buffer on a connected
//! device and reads that data back from it, using nonstandard
//! "vendor" codes.
//!
//! This utility depends on the `libusb` crate, which in turn requires
//! that the cross-platform (Windows, OSX, Linux) library
//! [libusb](http://libusb.info/) is installed on the host machine.
//!
//! To run the test, load the app in `examples/tests/usb` onto a device
//! running Tock; this app will enable the device's USB controller and
//! instruct it to respond to requests.
//!
//! Then, connect the device to a host machine's USB port and run this
//! program.
//!
//! The expected output of this program (after listing USB devices) is:
//!
//! Bulk wrote 8 bytes: [222, 173, 190, 239, 0, 0, 0, 0]
//! Bulk read  8 bytes: [222, 173, 190, 239, 0, 0, 0, 0]
//! Bulk wrote 8 bytes: [222, 173, 190, 239, 0, 0, 0, 1]
//! Bulk read  8 bytes: [222, 173, 190, 239, 0, 0, 0, 1]
//! Bulk wrote 8 bytes: [222, 173, 190, 239, 0, 0, 0, 2]
//! Bulk read  8 bytes: [222, 173, 190, 239, 0, 0, 0, 2]
//! Bulk wrote 8 bytes: [222, 173, 190, 239, 0, 0, 0, 3]
//! Bulk read  8 bytes: [222, 173, 190, 239, 0, 0, 0, 3]
//! [ ... and continuing with increasing numbers in the rightmost four bytes ... ]

extern crate libusb;

use libusb::*;
use std::thread::sleep;
use std::time::Duration;

const VENDOR_ID: u16 = 0x6667;
const PRODUCT_ID: u16 = 0xabcd;

fn main() {
    let context = Context::new().expect("Creating context");

    println!("Searching for device ...");
    let device_list = context.devices().expect("Getting device list");
    let mut dev = None;
    for d in device_list.iter() {
        let descr = d.device_descriptor().expect("Getting device descriptor");
        let matches = descr.vendor_id() == VENDOR_ID && descr.product_id() == PRODUCT_ID;
        println!(
            "{} {:02}:{:02} Vendor:{:04x} Product:{:04x}",
            if matches { "->" } else { "  " },
            d.bus_number(),
            d.address(),
            descr.vendor_id(),
            descr.product_id()
        );

        if matches {
            dev = Some(d);
        }
    }

    let mut dh = dev.expect("Matching device not found")
        .open()
        .expect("Opening device");

    dh.set_active_configuration(0)
        .expect("Setting active configuration");

    dh.claim_interface(0).expect("Claiming interface");

    let mut i: u32 = 0;
    loop {
        {
            let endpoint = 2;
            let address = endpoint | 0 << 7; // OUT endpoint
            let buf = &[
                0xde,
                0xad,
                0xbe,
                0xef,
                (i >> 24 & 0xff) as u8,
                (i >> 16 & 0xff) as u8,
                (i >> 8 & 0xff) as u8,
                (i >> 0 & 0xff) as u8,
            ];

            let timeout = Duration::from_secs(3);

            let n = dh.write_bulk(address, buf, timeout).expect("write_bulk");
            println!("Bulk wrote {} bytes: {:?}", n, &buf[..n]);
        }
        {
            let endpoint = 1;
            let address = endpoint | 1 << 7; // IN endpoint
            let mut buf = &mut [0; 8];
            let timeout = Duration::from_secs(3);

            let n = dh.read_bulk(address, buf, timeout).expect("read_bulk");
            println!("Bulk read  {} bytes: {:?}", n, &buf[..n]);
        }

        i += 1;
        sleep(Duration::from_secs(1));
    }
}
