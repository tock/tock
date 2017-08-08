#pragma once

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

#define DRIVER_NUM_BUTTON 0x3

int button_subscribe(subscribe_cb callback, void *ud);
int button_enable_interrupt(int pin_num);
int button_disable_interrupt(int pin_num);
int button_read(int pin_num);
int button_count(void);


#ifdef __cplusplus
}
#endif

