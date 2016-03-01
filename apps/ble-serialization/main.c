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
#include "serialization.h"


// char txdone[] = "TX DONE!\r\n";
char txdone[] = "T\n";
char hello[] = "Done!\r\n";
char am[] = "am!\r\n";

CB_TYPE nop(int x, int y, int z, void *ud) { return ASYNC; }




ble_address_set() {
    __asm("nop;");
}






char device_name[] = "FSTORM";


// Intervals for advertising and connections
simple_ble_config_t ble_config = {
    .platform_id       = 0x00,              // used as 4th octect in device BLE address
    .device_id         = DEVICE_ID_DEFAULT,
    .adv_name          = NULL,
    .adv_interval      = MSEC_TO_UNITS(500, UNIT_0_625_MS),
    .min_conn_interval = MSEC_TO_UNITS(500, UNIT_1_25_MS),
    .max_conn_interval = MSEC_TO_UNITS(1000, UNIT_1_25_MS)
};


void main() {

    gpio_enable_output(LED_0);
    gpio_clear(LED_0);

    ble_config.adv_name = device_name;

    // gpio_toggle(LED_0);
    // delay_ms(200);
    // gpio_toggle(LED_0);
    // delay_ms(200);

    // gpio_toggle(LED_0);
    // delay_ms(200);
    // gpio_toggle(LED_0);
    // delay_ms(200);

    // gpio_toggle(LED_0);
    // delay_ms(200);
    // gpio_toggle(LED_0);
    // delay_ms(200);


  // putnstr_async(hello, sizeof(hello), nop, NULL);



    serialization_init();

  // nrf51822_serialization_write(tx, 10);



  // Setup BLE
    simple_ble_init(&ble_config);


    // gpio_set(LED_0);

    // Advertise because why not
    simple_adv_only_name();





}

