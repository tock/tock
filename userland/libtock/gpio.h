#pragma once

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

#define GPIO_DRIVER_NUM 0x4

// GPIO pin enum is defined externally in platform headers
typedef uint32_t GPIO_Pin_t;

typedef enum {
  PullUp=0,
  PullDown,
  PullNone,
} GPIO_InputMode_t;

typedef enum {
  Change=0,
  RisingEdge,
  FallingEdge,
} GPIO_InterruptMode_t;

int gpio_enable_output(GPIO_Pin_t pin);
int gpio_set(GPIO_Pin_t pin);
int gpio_clear(GPIO_Pin_t pin);
int gpio_toggle(GPIO_Pin_t pin);
int gpio_enable_input(GPIO_Pin_t pin, GPIO_InputMode_t pin_config);
int gpio_read(GPIO_Pin_t pin);
int gpio_enable_interrupt(GPIO_Pin_t pin, GPIO_InputMode_t pin_config,
    GPIO_InterruptMode_t irq_config);
int gpio_disable_interrupt(GPIO_Pin_t pin);
int gpio_disable(GPIO_Pin_t pin);
int gpio_interrupt_callback(subscribe_cb callback, void* callback_args);

#ifdef __cplusplus
}
#endif
