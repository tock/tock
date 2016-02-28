#ifndef _GPIO_H
#define _GPIO_H

#include <unistd.h>
#include "tock.h"

#define GPIO_DRIVER_NUM 1
#define LED_0 PC10

#ifdef __cplusplus
extern "C" {
#endif

//XXX: This is platform dependent and needs to leave
//XXX: Also this should be fixed to be only the exposed IO
//XXX: Also build an LED driver that is separate from the GPIO driver
typedef enum {
  PC10=0,
  PC19,
  PC13,
  PA17,
  PC20,
  PA19,
  PA14,
  PA16,
  PA13,
  PA11,
  PA10,
  PA12,
  PC09,
} GPIO_PIN_t;

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
int gpio_enable_interrupts(GPIO_Pin_t pin, GPIO_InputMode_t pin_config,
    GPIO_InterruptMode_t irq_config);
int gpio_disable_interrupts(GPIO_Pin_t pin);
int gpio_disable(GPIO_Pin_t pin);
int gpio_interrupt_callback(subscribe_cb callback, void* callback_args);

#ifdef __cplusplus
}
#endif

#endif // _GPIO_H
