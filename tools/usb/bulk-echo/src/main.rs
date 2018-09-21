//! This testing utility is designed to communicate with a device
//! running Tock.  In particular, it interacts with the usbc_client
//! capsule.  The USB controller and the client capsule must be enabled
//! from userspace; the application in `libtock-c/examples/tests/usb/`
//! will do this.
//!
//! NOTE: A more flexible and performant variant of this utility can be
//! found in `tools/usb/bulk-echo-fast/`, but does not run on all platforms.
//! (Notably, not on Windows).  That utility is preferred if you can run it.
//!
//! This utility sends its stdin to a Bulk OUT endpoint on the attached
//! USB device, which then echos all data back to the PC via a
//! Bulk IN endpoint, and this utility will then send it to stdout:
//!
//!   stdin  >___                  ___< Bulk IN endpoint  <--\
//!              \                /                           | Tock usbc_client
//!                [this utility]                             | capsule echoes data
//!   stdout <___/                \___> Bulk OUT endpoint -->/
//!
//! Thus, a useful test of the USB software on Tock is to pipe a file of data
//! through the path show above, and confirm that the output is the same as the input.
//! The `test.sh` script in this directory does that.
//!
//! Note that a USB bus reset (which you can cause by reconnection) may be necessary
//! to properly initialize the state of the echo buffer on the device before
//! running this utility.
//!
//! This utility depends on the `libusb` crate, which in turn requires
//! that the cross-platform (Windows, OSX, Linux) library
//! [libusb](http://libusb.info/) is installed on the host machine.

extern crate libusb;

use libusb::{Context, Error};
#[allow(unused_imports)]
use std::io::{stderr, stdin, stdout, Read, Write};
use std::time::Duration;

const VENDOR_ID: u16 = 0x6667;
const PRODUCT_ID: u16 = 0xabcd;

macro_rules! debug {
    [ $( $arg:expr ),+ ] => {
        {}

        /*
        {
            write!(stderr(), $( $arg ),+).expect("write");
            write!(stderr(), "\n").expect("write");
        }
        */
    };
}

fn main() {
    let context = Context::new().expect("Creating context");
    let device_list = context.devices().expect("Getting device list");
    let mut dev = None;
    for d in device_list.iter() {
        let descr = d.device_descriptor().expect("Getting device descriptor");
        let matches = descr.vendor_id() == VENDOR_ID && descr.product_id() == PRODUCT_ID;
        if matches {
            dev = Some(d);
        }
    }
    let mut dh = dev.expect("Matching device not found").open().expect(
        "Opening device",
    );
    // dh.reset().expect("Reset");
    dh.set_active_configuration(0).expect(
        "Setting active configuration",
    );
    dh.claim_interface(0).expect("Claiming interface");

    // Unfortunately libusb doesn't provide an asynchronous interface,
    // so we'll make do here with blocking calls with short timeouts.
    // (Note that an async interface *is* available for the underlying
    // libusb C library.)

    let input_buf = &mut [0; 8];
    let mut input_buflen = 0;
    let mut out_bytes = 0;
    let mut in_bytes = 0;
    let mut stdin_closed = false;
    while !stdin_closed || in_bytes < out_bytes {
        if input_buflen == 0 {
            // Fill the buffer from stdin
            let n = stdin().read(input_buf).expect("read");
            if n == 0 {
                stdin_closed = true;
                debug!(
                    "[ {} out, {} in] End of input ... waiting to drain device",
                    out_bytes,
                    in_bytes
                );
            } else {
                input_buflen = n;
            }
        }

        if input_buflen > 0 {
            // Write it out to the device
            let endpoint = 2;
            let address = endpoint | 0 << 7; // OUT endpoint
            let timeout = Duration::from_secs(1);
            match dh.write_bulk(address, &input_buf[0..input_buflen], timeout) {
                Ok(n) => {
                    if n != input_buflen {
                        panic!("short write");
                    }
                    debug!(
                        "[ {} out, {} in] Bulk wrote {} bytes: {:?}",
                        out_bytes,
                        in_bytes,
                        n,
                        &input_buf[..n]
                    );
                    out_bytes += n;
                    input_buflen = 0;
                }
                Err(Error::Timeout) => {
                    debug!("write timeout");
                }
                _ => panic!("write_bulk"),
            }
        }

        if in_bytes < out_bytes {
            // Read some data back from the device
            let endpoint = 1;
            let address = endpoint | 1 << 7; // IN endpoint
            let timeout = Duration::from_secs(3);
            let buf = &mut [0; 8];
            match dh.read_bulk(address, buf, timeout) {
                Ok(n) => {
                    debug!(
                        "[ {} out, {} in] Bulk read  {} bytes: {:?}",
                        out_bytes,
                        in_bytes,
                        n,
                        &buf[..n]
                    );
                    in_bytes += n;

                    // Send it to stdout
                    stdout().write_all(&buf[..n]).expect("write");
                }
                Err(Error::Timeout) => {
                    debug!("read timeout");
                }
                _ => panic!("read_bulk"),
            }
        }
    }

    debug!("[ {} out, {} in] Done", out_bytes, in_bytes);
}
