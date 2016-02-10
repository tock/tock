#include <firestorm.h>
#include <tock.h>
#include <tmp006.h>

static int errno_val = 0;

// internal callback for faking synchronous reads
static CB_TYPE tmp006_cb(int r0, int r1, int r2, void* ud) {
    int16_t* result = (int16_t*)ud;
    *result = (int16_t)r0;

    errno_val = r1;

    // signal that the callback has completed
    return READTMP;
}

// enable TMP006, take a single reading, disable TMP006, return value to user
int tmp006_read_sync(int16_t* temp_reading) {
    uint32_t err_code = tmp006_read_async(temp_reading, tmp006_cb);
    if (err_code != ERR_NONE) {
        return err_code;
    }

    // wait for result
    wait_for(READTMP);

    // handle error codes
    if (errno_val != ERR_NONE) {
        return errno_val;
    }
    return ERR_NONE;
}

// enable TMP006, take a single reading, disable TMP006, callback with value
int tmp006_read_async(int16_t* temp_reading, subscribe_cb callback) {

    // subscribe to a single temp value callback
    //  also enables the temperature sensor for the duration of one sample
    return subscribe(2, 0, callback, temp_reading);
}

// enable TMP006, configure periodic sampling with interrupts, callback with value on interrupt
int tmp006_start_sampling(uint8_t period, subscribe_cb callback) {
    // set period for periodic temp readings
    uint32_t err_code = command(2, 0, period);
    if (err_code != ERR_NONE) {
        return err_code;
    }

    // subscribe to periodic temp value callbacks
    //  also enables the temperature sensor
    return subscribe(2, 1, callback, 0);
}

// disable TMP006
int tmp006_stop_sampling(void) {
    // unsubscribe from periodic temp value callbacks
    //  also disables the temperature sensor
    return command(2, 1, 0);
}

