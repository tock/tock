#pragma once

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

#define DRIVER_NUM_MAX17205 0x80001

// Set a callback for the MAX17205 driver.
//
// The callback function should look like:
//
//     void callback (int callback_type, int data, int data2, void* callback_args)
//
// callback_type is one of:
//    0: Got the battery status. `data` is:
//          status
//    1: Got the state of charge. `data` is:
//          percent charged in %/255
//        and `data2` is the capacity and full capacity:
//          word 0 (u16): full capacity in 0.5mAh
//          word 1 (u16): current capacity in 0.5mAh
//     2: Got voltage and current. `data` is:
//          voltage in 1.25mV
//        and 'data2' is:
//          current in 156.25uA
//     3: A write operation finished.
int max17205_set_callback (subscribe_cb callback, void* callback_args);

// Get the current status of the battery
// Result is returned in callback.
int max17205_read_status(void);

// Get the current state of charge of the battery.
// Result is returned in callback.
int max17205_read_soc(void);

// Get the current voltage and current of the battery.
// Result is returned in callback.
int max17205_read_voltage_current(void);

//get current count on the coulomb counter
int max17205_read_coulomb (void);

//
// Synchronous Versions
//
int max17205_read_status_sync(uint16_t* state);
int max17205_read_soc_sync(uint16_t* percent, uint16_t* soc_mah, uint16_t* soc_mah_full);
int max17205_read_voltage_current_sync(uint16_t* voltage, uint16_t* current);
int max17205_read_coulomb_sync (uint16_t* coulomb);

float max17205_get_voltage_mV(int vcount) __attribute__((const));
float max17205_get_current_uA(int ccount) __attribute__((const));
float max17205_get_percentage_mP(int percent) __attribute__((const));
float max17205_get_capacity_uAh(int cap) __attribute__((const));

#ifdef __cplusplus
}
#endif
