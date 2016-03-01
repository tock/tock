/* vim: set sw=2 expandtab tw=80: */

#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include <stdlib.h>
#include <unistd.h>

#include <firestorm.h>
#include <tmp006.h>

#include "nordic_common.h"
#include "nrf_error.h"
#include "ble_advdata.h"

#include "simple_ble.h"
#include "simple_adv.h"
#include "eddystone.h"

#include "nrf.h"

#include "delay.h"


void ble_address_set () {
    // ignore address setting for now, not sure if that works...
    __asm("nop;");
}


/*******************************************************************************
 * BLE
 ******************************************************************************/

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

// URL to advertise
char eddystone_url[] = "goo.gl/123abc";

// Manufacturer specific data setup
#define UMICH_COMPANY_IDENTIFIER 0x02E0
uint8_t mdata[2] = {0x99, 0xbe};

ble_advdata_manuf_data_t mandata;




/*******************************************************************************
 * TEMPERATURE
 ******************************************************************************/

int16_t temp_reading;

// callback to receive asynchronous data
CB_TYPE temp_callback (int temp_value, int error_code, int unused, void* callback_args) {
    UNUSED_PARAMETER(error_code);
    UNUSED_PARAMETER(unused);
    UNUSED_PARAMETER(callback_args);



    temp_reading = (int16_t) temp_value;


    return 0;
}

void temperature_init () {
    tmp006_start_sampling(0x2, temp_callback, NULL);
}






int main () {

    // Configure the LED for debugging
    gpio_enable_output(LED_0);
    gpio_clear(LED_0);

    // Set the device name in the struct this way to avoid errors with PIC code
    ble_config.adv_name = device_name;

    // Setup BLE
    simple_ble_init(&ble_config);

    mandata.company_identifier = UMICH_COMPANY_IDENTIFIER;
    mandata.data.p_data = mdata;
    mandata.data.size   = 2;

    eddystone_with_manuf_adv(eddystone_url, &mandata);
    // eddystone_with_manuf_adv(eddystone_url, &mandata);
    // eddystone_adv(eddystone_url, NULL);

    // Advertise our name packet
    // simple_adv_only_name();
    gpio_set(LED_0);
    temperature_init();

    while (1) {
        wait();

        // putstr("temp callback\n");





        // Update manufacturer specific data with new temp reading
        mdata[0] = temp_reading & 0xff;
        mdata[1] = (temp_reading >> 8) & 0xff;

        // And update advertising data
        eddystone_with_manuf_adv(eddystone_url, &mandata);
    }
}
