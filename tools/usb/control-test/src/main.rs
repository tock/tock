//! This utility performs a simple test of usb functionality in Tock:
//! It reads control data from a connected device and writes control
//! data back to it, using nonstandard "vendor" codes.
//!
//! This utility depends on the `libusb` crate, which in turn requires
//! that the cross-platform (Windows, OSX, Linux) library
//! [libusb](http://libusb.info/) is installed on the host machine.
//!
//! To run the test, load the app in `examples/tests/usb` onto a device
//! running Tock; this app will enable the device's USB controller and
//! instruct it to respond to control requests.
//!
//! Then, connect the device to a host machine's USB port and run this
//! program.
//!
//! The expected output of this program (after listing USB devices) is:
//!
//!     Received [10, 11, 12]
//!     Wrote [13, 14, 15]
//!
//! The program exits with status `0` if the above interactions occur
//! correctly.

extern crate libusb;

use libusb::*;
use std::time::Duration;

const VENDOR_ID: u16 = 0x6667;
const PRODUCT_ID: u16 = 0xabcd;

const EXPECT_BYTES: &'static [u8] = &[10, 11, 12];

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

    let mut dh = dev
        .expect("Matching device not found")
        .open()
        .expect("Opening device");

    dh.set_active_configuration(0)
        .expect("Setting active configuration");

    dh.claim_interface(0).expect("Claiming interface");

    {
        let request_type = request_type(Direction::In, RequestType::Vendor, Recipient::Device);
        let request = 1;
        let value = 0;
        let index = 0;
        let timeout = Duration::from_secs(3);
        let buf = &mut [0; 8];
        let n = dh
            .read_control(request_type, request, value, index, buf, timeout)
            .expect("read_control");
        let received = &buf[..n];

        println!("Received {:?}", received);

        if received != EXPECT_BYTES {
            panic!("Received data does not match expected pattern");
        }
    }

    {
        let request_type = request_type(Direction::Out, RequestType::Vendor, Recipient::Other);
        let request = 1;
        let value = 0;
        let index = 0;
        let timeout = Duration::from_secs(3);
        let buf = &[0xd, 0xe, 0xf];
        let n = dh
            .write_control(request_type, request, value, index, buf, timeout)
            .expect("write_control");

        println!("Wrote {:?}", &buf[..n]);
    }
}
