/*
 * This testing utility is designed to communicate with a device
 * running Tock.  In particular, it interacts with the usbc_client
 * capsule.  The USB controller and the client capsule must be enabled
 * from userspace; the application in `userland/examples/tests/usb/`
 * will do this.
 *
 * This utility sends its stdin to a Bulk OUT endpoint on the attached
 * USB device, which then echos all data back to the PC via a
 * Bulk IN endpoint, and this utility will then send it to stdout:
 *
 *   stdin  >___                  ___< Bulk IN endpoint  <--\
 *              \                /                           | Tock usbc_client
 *                [this utility]                             | capsule echoes data
 *   stdout <___/                \___> Bulk OUT endpoint -->/
 *
 * Thus, a useful test of the USB software on Tock is to pipe a file of data
 * through the path show above, and confirm that the output is the same as the input.
 * The `test.sh` script in this directory does that.
 *
 * Note that a USB bus reset (which you can cause by reconnection) may be necessary
 * to properly initialize the state of the echo buffer on the device before
 * running this utility.  Passing "-r" as this first argument to this utility
 * will cause it to perform a bus reset via libusb, but for some reason this
 * then causes the subsequent libusb_open() call to fail.
 *
 * This utility requires that the cross-platform (Windows, OSX, Linux) library
 * [libusb](http://libusb.info/) is installed on the host machine.  (Tested
 * with libusb 1.0.22.)
 *
 * NOTE: This code uses libusb interfaces (get_pollfds) not available on Windows.
 * A less-performant but cross-platform variant of this utility is available in
 * tools/usb/bulk-echo.
 */
#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <unistd.h>
#include <poll.h>
#include <error.h>
#include <sys/time.h>
#include "libusb.h"

typedef int bool;
static const bool false = 0;
static const bool true = 1;

static size_t bytes_out = 0;
static size_t bytes_in = 0;

static const size_t max_poll_fds = 10;
static struct pollfd fds[max_poll_fds];
static const int timeout_never = -1;
static size_t stdin_fdi;

// Choose odd buffer size here to stimulate bugs
static const size_t input_bufsz = 103;
static unsigned char input_buf[input_bufsz];
static size_t input_buflen = 0;
static size_t input_buf_avail(void);
static bool input_buf_locked = false;
static size_t read_input(void);

static bool reading_in = false;

static const uint16_t TARGET_VENDOR_ID = 0x6667;
static const uint16_t TARGET_PRODUCT_ID = 0xabcd;

unsigned char endpoint_bulk_in = 1 | 1 << 7;
unsigned char endpoint_bulk_out = 2 | 0 << 7;

static libusb_device_handle *zorp;

void open_device(void);

static struct timeval timeval_zero = { 0, 0 };

static bool input_closed = false;
void submit_transfers(void);
void handle_events(void);

#ifdef LOGGING
#define LOG_STRING(msg) "[ buf %4lu | device %s%s | %4lu out, %4lu in ] " msg "\n"
#define LOG_ARGS \
    input_buflen, \
    input_buf_locked ? "w" : " ", \
    reading_in ? "r" : " ", \
    bytes_out, bytes_in
#define log(fmt, ...) \
    fprintf (stderr, LOG_STRING(fmt), LOG_ARGS, ##__VA_ARGS__)
#else
#define log(fmt, ...) 
#endif

int main(int argc, char **argv) {
    int r;

    bool do_reset = 0;
    if (argc == 2 && !strcmp(argv[1], "-r")) {
        do_reset = 1;
    }

    r = libusb_init(NULL);
    if (r < 0)
        error(1, r, "libusb_init");

    open_device();

    if (do_reset) {
        log("Reset");
        r = libusb_reset_device(zorp);
        switch (r) {
            case 0:
                break;
            case LIBUSB_ERROR_NOT_FOUND:
                libusb_close(zorp);
                open_device();
                break;
            default:
                error(1, 0, "reset: %s", libusb_error_name(r));
        }
    }

    if ((r = libusb_set_configuration(zorp, 0)) != 0)
        error(1, 0, "set_configuration");
    if ((r = libusb_claim_interface(zorp, 0)) != 0)
        error(1, 0, "claim_interface");

    log("Start");

    while (!input_closed || bytes_in < bytes_out) {
        submit_transfers();
        handle_events();
    }

    log("Done");
    return 0;
}

void open_device(void) {
    libusb_device **devs;
    int r;
    ssize_t cnt;

    cnt = libusb_get_device_list(NULL, &devs);
    if (cnt < 0)
        error(1, (int) cnt, "libusb_get_device_list");

    libusb_device *dev;
    int i = 0;
    while ((dev = devs[i++]) != NULL) {
        struct libusb_device_descriptor desc;
        int r = libusb_get_device_descriptor(dev, &desc);
        if (r < 0)
            error(1, r, "failed to get device descriptor");

        if (desc.idVendor == TARGET_VENDOR_ID &&
            desc.idProduct == TARGET_PRODUCT_ID)
            break;
    }

    if (dev == NULL)
        error(1, 0, "Couldn't find target device");

    r = libusb_open(dev, &zorp);
    if (r != 0)
        error(1, 0, "open: %s", libusb_error_name(r));

    bool unref_devices = true;
    libusb_free_device_list(devs, unref_devices);
}

void LIBUSB_CALL write_done(struct libusb_transfer *transfer) {
    switch (transfer->status) {
        case LIBUSB_TRANSFER_COMPLETED:
            if (transfer->actual_length != transfer->length) {
                error(1, 0, "short write");
            }
            log("Wrote %d bytes to device", transfer->actual_length);

            input_buflen = 0;
            input_buf_locked = false;
            bytes_out += transfer->actual_length;
            break;
        default:
            error(1, 0, "bad transfer status: %s", libusb_error_name(transfer->status));
    }

    libusb_free_transfer(transfer);
}

// It seems non-multiples-of-8 cause trouble here ... not sure why
static const size_t return_buf_sz = 80;
static unsigned char return_buf[return_buf_sz];

void LIBUSB_CALL read_done(struct libusb_transfer *transfer) {
    switch (transfer->status) {
        case LIBUSB_TRANSFER_COMPLETED:
            log("Read %d bytes from device", transfer->actual_length);

            fwrite(return_buf, transfer->actual_length, 1, stdout);
            bytes_in += transfer->actual_length;
            reading_in = false;
            break;
        default:
            error(1, 0, "bad transfer status: %s", libusb_error_name(transfer->status));
    }

    libusb_free_transfer(transfer);
}

void submit_transfers(void) {
    if (!input_buf_locked && input_buflen > 0) {
        // Write input buf to device

        int iso_packets = 0;
        struct libusb_transfer* transfer = libusb_alloc_transfer(iso_packets);
        libusb_fill_bulk_transfer(transfer, zorp, endpoint_bulk_out,
                                  input_buf, input_buflen, write_done, NULL, 0);

        log("-> Write %d bytes to device", transfer->length);

        // Don't fiddle with input buffer while libusb is trying to send it
        input_buf_locked = true;

        if (libusb_submit_transfer(transfer))
            error(1, 0, "submit");
    }

    if (!reading_in) {
        // Read data back from device

        int iso_packets = 0;
        struct libusb_transfer* transfer = libusb_alloc_transfer(iso_packets);
        libusb_fill_bulk_transfer(transfer, zorp, endpoint_bulk_in,
                                  return_buf, return_buf_sz, read_done, NULL, 0);

        log("-> Read from device");

        if (libusb_submit_transfer(transfer))
            error(1, 0, "submit");
        reading_in = true;
    }
}

void handle_events(void) {
    nfds_t nfds = 0;

    // Add stdin fd
    bool poll_stdin = !input_closed && !input_buf_locked && input_buf_avail() > 0;
    if (poll_stdin) {
        if (nfds + 1 > max_poll_fds) {
          error(1, 0, "too many fds");
        }
        fds[nfds].fd = 0;
        fds[nfds].events = POLLIN;
        fds[nfds].revents = 0;
        stdin_fdi = nfds;
        nfds++;
    }

    // Add libusb fds
    const struct libusb_pollfd **all_usb_fds = libusb_get_pollfds(NULL);
    if (all_usb_fds == NULL) {
        error(1, 0, "libusb_get_pullfds");
    }
    for (const struct libusb_pollfd **usb_fds = all_usb_fds; *usb_fds != NULL; usb_fds++) {
        const struct libusb_pollfd *pollfd = *usb_fds;
        if (nfds + 1 > max_poll_fds) {
          error(1, 0, "too many fds");
        }
        fds[nfds].fd = pollfd->fd;
        fds[nfds].events = pollfd->events;
        fds[nfds].revents = 0;
        nfds++;
    }
    libusb_free_pollfds(all_usb_fds);

    if (nfds == 0) {
        // Nothing to wait for
        error(1, 0, "Deadlocked");
    }

    // Poll for ready fds
    int nfds_active = poll(fds, nfds, timeout_never);
    if (nfds_active < 0) {
        error(1, nfds_active, "poll");
    }

    // Check if stdin ready
    if (poll_stdin) {
        if (fds[stdin_fdi].revents != 0) {
            if (read_input() == 0) {
              input_closed = true;
            }
            nfds_active--;
        }
    }

    if (nfds_active > 0) {
        // libusb must be ready

        int r = libusb_handle_events_timeout(NULL, &timeval_zero);
        if (r != 0) {
            error(1, 0, "libusb_handle_events: %s", libusb_error_name(r));
        }
    }
}

// Read input from stdin

static size_t input_buf_avail(void) {
    return input_bufsz - input_buflen;
}

static size_t read_input(void) {
    size_t to_read = input_bufsz - input_buflen;
    log("-> Input up to %ld bytes", to_read);
    ssize_t r = read(0, input_buf + input_buflen, to_read);
    if (r < 0) {
        error(1, r, "read");
    }
    else {
        log("Input %ld bytes", r);

        input_buflen += r;
    }
    return r;
}
