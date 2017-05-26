#pragma once

#include <tock.h>

#define DRIVER_RADIO  33
#define RX            0
#define TX            1
#define CH            2
#define BLE_ADV_START 3
#define BLE_ADV_STOP  4
#define SET_NAME      5
#define SET_DATA      6


#ifdef __cplusplus
extern "C" {
#endif

int subscribe_rx(subscribe_cb callback, void *ud);
int subscribe_tx(subscribe_cb callback, void *ud);
int tx_data(const char* packet, unsigned char len);
int rx_data(const char* packet, unsigned char len);
int read_data(const char* packet, subscribe_cb callback, unsigned char len);
int set_channel(int ch_num);

int start_ble_advertisement(const char* name, const char* data);
int stop_ble_advertisement(void);
#ifdef __cplusplus
}
#endif
