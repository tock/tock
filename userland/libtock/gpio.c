#include "gpio.h"

int gpio_enable_output(GPIO_Pin_t pin) {
  return command(GPIO_DRIVER_NUM, 1, pin);
}

int gpio_set(GPIO_Pin_t pin) {
  return command(GPIO_DRIVER_NUM, 2, pin);
}

int gpio_clear(GPIO_Pin_t pin) {
  return command(GPIO_DRIVER_NUM, 3, pin);
}

int gpio_toggle(GPIO_Pin_t pin) {
  return command(GPIO_DRIVER_NUM, 4, pin);
}

int gpio_enable_input(GPIO_Pin_t pin, GPIO_InputMode_t pin_config) {
  uint32_t data = ((pin_config & 0xFF) << 8) | (pin & 0xFF);
  return command(GPIO_DRIVER_NUM, 5, data);
}

int gpio_read(GPIO_Pin_t pin) {
  return command(GPIO_DRIVER_NUM, 6, pin);
}

int gpio_enable_interrupt(GPIO_Pin_t pin, GPIO_InputMode_t pin_config,
    GPIO_InterruptMode_t irq_config) {
  uint32_t data = ((irq_config & 0xFF) << 16) | ((pin_config & 0xFF) << 8) | (pin & 0xFF);
  return command(GPIO_DRIVER_NUM, 7, data);
}

int gpio_disable_interrupt(GPIO_Pin_t pin) {
  return command(GPIO_DRIVER_NUM, 8, pin);
}

int gpio_disable(GPIO_Pin_t pin) {
  return command(GPIO_DRIVER_NUM, 9, pin);
}

int gpio_interrupt_callback(subscribe_cb callback, void* callback_args) {
  return subscribe(GPIO_DRIVER_NUM, 0, callback, callback_args);
}

