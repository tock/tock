#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include <stdlib.h>
#include <unistd.h>

#include <nordic_common.h>
#include <nrf_error.h>
#include <ble_advdata.h>

#include <simple_ble.h>
#include <simple_adv.h>
#include <eddystone.h>

#include <tmp006.h>
#include <isl29035.h>
#include <nrf51_serialization.h>

#include "nrf.h"
#include "env_sense_service.h"


/*******************************************************************************
 * BLE
 ******************************************************************************/

uint16_t conn_handle = BLE_CONN_HANDLE_INVALID;

// Intervals for advertising and connections
//char device_name[] = "FSTORM";
simple_ble_config_t ble_config = {
    .platform_id       = 0x00,              // used as 4th octect in device BLE address
    .device_id         = DEVICE_ID_DEFAULT,
    .adv_name          = "TOCK-BLE-ENV",
    .adv_interval      = MSEC_TO_UNITS(500, UNIT_0_625_MS),
    .min_conn_interval = MSEC_TO_UNITS(1000, UNIT_1_25_MS),
    .max_conn_interval = MSEC_TO_UNITS(1250, UNIT_1_25_MS)
};

// URL to advertise
const char eddystone_url[] = "goo.gl/8685Uw";

// Manufacturer specific data setup
#define UMICH_COMPANY_IDENTIFIER 0x02E0
#define BLE_APP_ID  0x15
#define BLE_APP_VERSION_NUM 0x00
uint8_t mdata[4] = {BLE_APP_ID, BLE_APP_VERSION_NUM, 0xFF, 0xFF};
ble_advdata_manuf_data_t mandata = {
  .company_identifier = UMICH_COMPANY_IDENTIFIER,
  .data.p_data = mdata,
  .data.size   = sizeof(mdata)
};

__attribute__ ((const))
void ble_address_set (void) {
  // nop
}

void ble_evt_user_handler (ble_evt_t* p_ble_evt) {
    ble_gap_conn_params_t conn_params;
    memset(&conn_params, 0, sizeof(conn_params));
    conn_params.min_conn_interval = ble_config.min_conn_interval;
    conn_params.max_conn_interval = ble_config.max_conn_interval;
    conn_params.slave_latency     = SLAVE_LATENCY;
    conn_params.conn_sup_timeout  = CONN_SUP_TIMEOUT;

    switch (p_ble_evt->header.evt_id) {
        case BLE_GAP_EVT_CONN_PARAM_UPDATE:
            // just update them right now
            sd_ble_gap_conn_param_update(0, &conn_params);
            break;
    }
}

void ble_evt_connected(ble_evt_t* p_ble_evt) {
    UNUSED_PARAMETER(p_ble_evt);
    printf("Connected to central\n");

    ble_common_evt_t *common = (ble_common_evt_t*)&p_ble_evt->evt;
    conn_handle = common->conn_handle;
}

void ble_evt_disconnected(ble_evt_t* p_ble_evt) {
    UNUSED_PARAMETER(p_ble_evt);
    printf("Disconnected from central\n");
    conn_handle = BLE_CONN_HANDLE_INVALID;
}

void ble_error (uint32_t error_code) {
    printf("BLE ERROR: Code = %d\n", (int)error_code);
}

void services_init (void) {
  env_sense_service_init();
}

/*******************************************************************************
 * Sensing callbacks
 ******************************************************************************/

// Temperature read callback
static void temp_callback (int temp_value, int error_code, int unused, void* ud) {
    UNUSED_PARAMETER(error_code);
    UNUSED_PARAMETER(unused);
    UNUSED_PARAMETER(ud);

    int temp_reading = (int16_t)temp_value * 100;
    printf("Temp reading = %d\n", (int)temp_reading);

    env_sense_update_temperature(conn_handle, temp_reading);

    int lux = isl29035_read_light_intensity();
    printf("Light (lux) reading = %d\n", lux);

    // precision of 0.1 watts/m2, assuming sunlight efficacy of 93 lumens per watt.
    uint16_t irradiance = lux * 10 / 93;
    env_sense_update_irradiance(conn_handle, irradiance);
}

/*******************************************************************************
 * MAIN
 ******************************************************************************/

int main (void) {
    printf("Starting BLE serialization example\n");

    // Setup BLE
    conn_handle = simple_ble_init(&ble_config)->conn_handle;

    ble_advdata_t srdata;
    memset(&srdata, 0, sizeof(srdata));

    srdata.name_type = BLE_ADVDATA_FULL_NAME;
    srdata.p_manuf_specific_data = &mandata;
    ble_uuid_t PHYSWEB_SERVICE_UUID[] = {{0x181A, BLE_UUID_TYPE_BLE}};
    ble_advdata_uuid_list_t service_list = {
      .uuid_cnt = 1,
      .p_uuids = PHYSWEB_SERVICE_UUID
    };
    srdata.uuids_complete = service_list;

    // And update advertising data
    eddystone_adv(eddystone_url, &srdata);

    // Setup reading from the temperature sensor
    tmp006_start_sampling(0x2, temp_callback, NULL);
}

