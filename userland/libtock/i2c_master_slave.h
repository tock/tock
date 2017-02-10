#pragma once

#include "tock.h"

#define DRIVER_NUM_I2CMASTERSLAVE 13

#define CB_SLAVE_READ_REQUEST   2
#define CB_SLAVE_READ_COMPLETE  4
#define CB_SLAVE_WRITE          3
#define CB_MASTER_READ          1
#define CB_MASTER_WRITE         0

int i2c_master_slave_set_callback (subscribe_cb callback, void* callback_args);
int i2c_master_slave_set_master_write_buffer(uint8_t* buffer, uint32_t len);
int i2c_master_slave_set_master_read_buffer(uint8_t* buffer, uint32_t len);
int i2c_master_slave_set_slave_read_buffer(uint8_t* buffer, uint32_t len);
int i2c_master_slave_set_slave_write_buffer(uint8_t* buffer, uint32_t len);
int i2c_master_slave_write(uint8_t address, uint8_t length);
int i2c_master_slave_read(uint16_t address, uint16_t len);
int i2c_master_slave_listen(void);
int i2c_master_slave_set_slave_address(uint8_t address);

int i2c_master_slave_write_sync(uint8_t address, uint8_t length);
int i2c_master_slave_read_sync(uint16_t address, uint16_t len);
