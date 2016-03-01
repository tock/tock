/* vim: set sw=2 expandtab tw=80: */

#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include <stdlib.h>
#include <unistd.h>

#include <firestorm.h>

#include "nrf_error.h"

#include "simple_ble.h"
#include "simple_adv.h"

#include "nrf.h"

#include "delay.h"


void ble_address_set () {
    // ignore address setting for now, not sure if that works...
    __asm("nop;");
}



// Intervals for advertising and connections
char device_name[] = "FSTORM";
simple_ble_config_t ble_config = {
    .platform_id       = 0x00,              // used as 4th octect in device BLE address
    .device_id         = DEVICE_ID_DEFAULT,
    .adv_name          = NULL,
    .adv_interval      = MSEC_TO_UNITS(500, UNIT_0_625_MS),
    .min_conn_interval = MSEC_TO_UNITS(500, UNIT_1_25_MS),
    .max_conn_interval = MSEC_TO_UNITS(1000, UNIT_1_25_MS)
};



int main () {

    // Configure the LED for debugging
    gpio_enable_output(LED_0);
    gpio_clear(LED_0);

    // Set the device name in the struct this way to avoid errors with PIC code
    ble_config.adv_name = device_name;

    // Setup BLE
    simple_ble_init(&ble_config);

    // Advertise our name packet
    simple_adv_only_name();
}
