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
//char device_name[] = "FSTORM";
simple_ble_config_t ble_config = {
    .platform_id       = 0x00,              // used as 4th octect in device BLE address
    .device_id         = DEVICE_ID_DEFAULT,
    .adv_name          = "FSTORM",
    .adv_interval      = MSEC_TO_UNITS(500, UNIT_0_625_MS),
    .min_conn_interval = MSEC_TO_UNITS(500, UNIT_1_25_MS),
    .max_conn_interval = MSEC_TO_UNITS(1000, UNIT_1_25_MS)
};

// URL to advertise
char eddystone_url[] = "goo.gl/8685Uw";

// Manufacturer specific data setup
#define UMICH_COMPANY_IDENTIFIER 0x02E0
#define BLE_APP_ID  0x15
#define BLE_APP_VERSION_NUM 0x00
ble_advdata_manuf_data_t mandata;
uint8_t mdata[4] = {BLE_APP_ID, BLE_APP_VERSION_NUM, 0x99, 0xbe};

// Sensor data service
static simple_ble_service_t sensor_service = {
    .uuid128 = {{0x1b, 0x98, 0x8e, 0xc4, 0xd0, 0xc4, 0x4a, 0x85,
                 0x91, 0x96, 0x95, 0x57, 0xf8, 0x02, 0xa0, 0x54}}};

    // characteristic to display temperature values
    static simple_ble_char_t temp_sensor_char = {.uuid16 = 0xf803};
    static int16_t temp_reading;

/*******************************************************************************
 * TEMPERATURE
 ******************************************************************************/


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


/*******************************************************************************
 * BLE
 ******************************************************************************/

void ble_error (uint32_t error_code) {
    char buf[64];
    snprintf(buf, 64, "BLE ERROR: Code = %d\n", error_code);
    putstr(buf);
}

void services_init (void) {
    // add sensor data service
    simple_ble_add_service(&sensor_service);

        // add characteristic for temperature
        temp_reading = 0xFFFF;
        simple_ble_add_stack_characteristic(1, 0, 0, 0, // read, write, notify, vlen
                2, (uint8_t*)&temp_reading,
                &sensor_service, &temp_sensor_char);
}

/*******************************************************************************
 * MAIN
 ******************************************************************************/

int main () {
    putstr("Starting BLE serialization example\n");
    putstr("Unplug/Replug to start app\n");

    // Configure the LED for debugging
    gpio_enable_output(LED_0);
    gpio_clear(LED_0);

    // Setup BLE
    simple_ble_init(&ble_config);

    // Init advertising data
    mandata.company_identifier = UMICH_COMPANY_IDENTIFIER;
    mandata.data.p_data = mdata;
    mandata.data.size   = 4;
    eddystone_with_manuf_adv(eddystone_url, &mandata);

    // Setup reading from the temperature sensor
    temperature_init();

    while (1) {
        // When this returns, we should have gotten a new temp reading
        wait();

        // Update manufacturer specific data with new temp reading
        putstr("Data!\n");
        simple_ble_stack_char_set(&temp_sensor_char, 2, temp_reading);
        mdata[2] = temp_reading & 0xff;
        mdata[3] = (temp_reading >> 8) & 0xff;

        // And update advertising data
        eddystone_with_manuf_adv(eddystone_url, &mandata);
    }
}
