#include "gpio.h"

int gpio_enable_output(GPIO_Pin_t pin) {
  return command(GPIO_DRIVER_NUM, 1, pin, 0);
}

int gpio_set(GPIO_Pin_t pin) {
  return command(GPIO_DRIVER_NUM, 2, pin, 0);
}

int gpio_clear(GPIO_Pin_t pin) {
  return command(GPIO_DRIVER_NUM, 3, pin, 0);
}

int gpio_toggle(GPIO_Pin_t pin) {
  return command(GPIO_DRIVER_NUM, 4, pin, 0);
}

int gpio_enable_input(GPIO_Pin_t pin, GPIO_InputMode_t pin_config) {
  return command(GPIO_DRIVER_NUM, 5, pin, pin_config);
}

int gpio_read(GPIO_Pin_t pin) {
  return command(GPIO_DRIVER_NUM, 6, pin, 0);
}

int gpio_enable_interrupt(GPIO_Pin_t pin, GPIO_InterruptMode_t irq_config) {
  return command(GPIO_DRIVER_NUM, 7, pin, irq_config);
}

int gpio_disable_interrupt(GPIO_Pin_t pin) {
  return command(GPIO_DRIVER_NUM, 8, pin, 0);
}

int gpio_disable(GPIO_Pin_t pin) {
  return command(GPIO_DRIVER_NUM, 9, pin, 0);
}

int gpio_interrupt_callback(subscribe_cb callback, void* callback_args) {
  return subscribe(GPIO_DRIVER_NUM, 0, callback, callback_args);
}

