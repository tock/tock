/*
 * Advertise a URL according to the Eddystone protocol.
 * https://github.com/google/eddystone/blob/master/protocol-specification.md
 */

// Global libraries
#include <stdint.h>

// Nordic libraries
#include "ble_advdata.h"

// nrf5x-base libraries
#include "simple_ble.h"
#include "eddystone.h"

// Define constants about this beacon.
#define DEVICE_NAME "nRFtest"
#define PHYSWEB_URL "goo.gl/aaaaaa"

// Intervals for advertising and connections
static simple_ble_config_t ble_config = {
    .platform_id       = 0x00,              // used as 4th octect in device BLE address
    .device_id         = DEVICE_ID_DEFAULT,
    .adv_name          = DEVICE_NAME,       // used in advertisements if there is room
    .adv_interval      = MSEC_TO_UNITS(500, UNIT_0_625_MS),
    .min_conn_interval = MSEC_TO_UNITS(500, UNIT_1_25_MS),
    .max_conn_interval = MSEC_TO_UNITS(1000, UNIT_1_25_MS)
};

// main is essentially two library calls to setup all of the Nordic SDK
// API calls.
int main(void) {

    // Setup BLE
    simple_ble_init(&ble_config);

    // Advertise a URL
    eddystone_adv(PHYSWEB_URL, NULL);

    while (1) {
        power_manage();
    }
}
