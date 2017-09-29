#include "gpio_async.h"
#include "tock.h"

#define CONCAT_PORT_PIN(port, pin) (((pin & 0xFF) << 8) | (port & 0xFF))
#define CONCAT_PORT_PIN_DATA(port, pin, data) (((data & 0xFFFF) << 16) | ((pin & 0xFF) << 8) | (port & 0xFF))


struct gpio_async_data {
  bool fired;
  int value;
  int callback_type;
};

static struct gpio_async_data result = { .fired = false };

// Internal callback for faking synchronous reads
static void gpio_async_cb(__attribute__ ((unused)) int callback_type,
                          __attribute__ ((unused)) int value,
                          __attribute__ ((unused)) int unused,
                          void* ud) {
  struct gpio_async_data* myresult = (struct gpio_async_data*) ud;
  myresult->callback_type = callback_type;
  myresult->value         = value;
  myresult->fired         = true;
}


int gpio_async_set_callback (subscribe_cb callback, void* callback_args) {
  return subscribe(DRIVER_NUM_GPIO_ASYNC, 0, callback, callback_args);
}

int gpio_async_make_output(uint32_t port, uint8_t pin) {
  return command(DRIVER_NUM_GPIO_ASYNC, 1, CONCAT_PORT_PIN(port, pin), 0);
}

int gpio_async_set(uint32_t port, uint8_t pin) {
  return command(DRIVER_NUM_GPIO_ASYNC, 2, CONCAT_PORT_PIN(port, pin), 0);
}

int gpio_async_clear(uint32_t port, uint8_t pin) {
  return command(DRIVER_NUM_GPIO_ASYNC, 3, CONCAT_PORT_PIN(port, pin), 0);
}

int gpio_async_toggle(uint32_t port, uint8_t pin) {
  return command(DRIVER_NUM_GPIO_ASYNC, 4, CONCAT_PORT_PIN(port, pin), 0);
}

int gpio_async_make_input(uint32_t port, uint8_t pin, GPIO_InputMode_t pin_config) {
  return command(DRIVER_NUM_GPIO_ASYNC, 5, CONCAT_PORT_PIN_DATA(port, pin, pin_config), 0);
}

int gpio_async_read(uint32_t port, uint8_t pin) {
  return command(DRIVER_NUM_GPIO_ASYNC, 6, CONCAT_PORT_PIN(port, pin), 0);
}

int gpio_async_enable_interrupt(uint32_t port, uint8_t pin, GPIO_InterruptMode_t irq_config) {
  return command(DRIVER_NUM_GPIO_ASYNC, 7, CONCAT_PORT_PIN_DATA(port, pin, irq_config), 0);
}

int gpio_async_disable_interrupt(uint32_t port, uint8_t pin) {
  return command(DRIVER_NUM_GPIO_ASYNC, 8, CONCAT_PORT_PIN(port, pin), 0);
}

int gpio_async_disable(uint32_t port, uint8_t pin) {
  return command(DRIVER_NUM_GPIO_ASYNC, 9, CONCAT_PORT_PIN(port, pin), 0);
}

int gpio_async_interrupt_callback(subscribe_cb callback, void* callback_args) {
  return subscribe(DRIVER_NUM_GPIO_ASYNC, 1, callback, callback_args);
}



int gpio_async_make_output_sync(uint32_t port, uint8_t pin) {
  int err;
  result.fired = false;

  err = gpio_async_set_callback(gpio_async_cb, (void*) &result);
  if (err < 0) return err;

  err = gpio_async_make_output(port, pin);
  if (err < 0) return err;

  // Wait for the callback.
  yield_for(&result.fired);

  return result.value;
}

int gpio_async_set_sync(uint32_t port, uint8_t pin) {
  int err;
  result.fired = false;

  err = gpio_async_set_callback(gpio_async_cb, (void*) &result);
  if (err < 0) return err;

  err = gpio_async_set(port, pin);
  if (err < 0) return err;

  // Wait for the callback.
  yield_for(&result.fired);

  return result.value;
}

int gpio_async_clear_sync(uint32_t port, uint8_t pin) {
  int err;
  result.fired = false;

  err = gpio_async_set_callback(gpio_async_cb, (void*) &result);
  if (err < 0) return err;

  err = gpio_async_clear(port, pin);
  if (err < 0) return err;

  // Wait for the callback.
  yield_for(&result.fired);

  return result.value;
}

int gpio_async_toggle_sync(uint32_t port, uint8_t pin) {
  int err;
  result.fired = false;

  err = gpio_async_set_callback(gpio_async_cb, (void*) &result);
  if (err < 0) return err;

  err = gpio_async_toggle(port, pin);
  if (err < 0) return err;

  // Wait for the callback.
  yield_for(&result.fired);

  return result.value;
}

int gpio_async_make_input_sync(uint32_t port, uint8_t pin, GPIO_InputMode_t pin_config) {
  int err;
  result.fired = false;

  err = gpio_async_set_callback(gpio_async_cb, (void*) &result);
  if (err < 0) return err;

  err = gpio_async_make_input(port, pin, pin_config);
  if (err < 0) return err;

  // Wait for the callback.
  yield_for(&result.fired);

  return result.value;
}

int gpio_async_read_sync(uint32_t port, uint8_t pin) {
  int err;
  result.fired = false;

  err = gpio_async_set_callback(gpio_async_cb, (void*) &result);
  if (err < 0) return err;

  err = gpio_async_read(port, pin);
  if (err < 0) return err;

  // Wait for the callback.
  yield_for(&result.fired);

  return result.value;
}

int gpio_async_enable_interrupt_sync(uint32_t port, uint8_t pin, GPIO_InterruptMode_t irq_config) {
  int err;
  result.fired = false;

  err = gpio_async_set_callback(gpio_async_cb, (void*) &result);
  if (err < 0) return err;

  err = gpio_async_enable_interrupt(port, pin, irq_config);
  if (err < 0) return err;

  // Wait for the callback.
  yield_for(&result.fired);

  return result.value;
}

int gpio_async_disable_interrupt_sync(uint32_t port, uint8_t pin) {
  int err;
  result.fired = false;

  err = gpio_async_set_callback(gpio_async_cb, (void*) &result);
  if (err < 0) return err;

  err = gpio_async_disable_interrupt(port, pin);
  if (err < 0) return err;

  // Wait for the callback.
  yield_for(&result.fired);

  return result.value;
}

int gpio_async_disable_sync(uint32_t port, uint8_t pin) {
  int err;
  result.fired = false;

  err = gpio_async_set_callback(gpio_async_cb, (void*) &result);
  if (err < 0) return err;

  err = gpio_async_disable(port, pin);
  if (err < 0) return err;

  // Wait for the callback.
  yield_for(&result.fired);

  return result.value;
}
