#pragma once

#include <tock.h>

#define DRIVER_NUM_BUTTON 9

#ifdef __cplusplus
extern "C" {
#endif

int button_subscribe(subscribe_cb callback, void *ud);
int button_enable_interrupt(int pin_num);
int button_disable_interrupt(int pin_num);
int button_read(int pin_num);
int button_count(void);


#ifdef __cplusplus
}
#endif

