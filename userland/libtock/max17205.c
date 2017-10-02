#include "max17205.h"
#include "tock.h"

struct max17205_data {
  int value0;
  int value1;
  bool fired;
};

static struct max17205_data result = { .fired = false, .value0 = 0, .value1 = 0 };

// Internal callback for faking synchronous reads
static void max17205_cb(__attribute__ ((unused)) int callback_type,
                        int value0,
                        int value1,
                        void* ud) {
  struct max17205_data* data = (struct max17205_data*) ud;
  data->value0 = value0;
  data->value1 = value1;
  data->fired  = true;
}

int max17205_set_callback (subscribe_cb callback, void* callback_args) {
  return subscribe(DRIVER_NUM_MAX17205, 0, callback, callback_args);
}

int max17205_read_status(void) {
  return command(DRIVER_NUM_MAX17205, 1, 0, 0);
}

int max17205_read_soc(void) {
  return command(DRIVER_NUM_MAX17205, 2, 0, 0);
}

int max17205_read_voltage_current(void) {
  return command(DRIVER_NUM_MAX17205, 3, 0, 0);
}

int max17205_read_coulomb(void) {
  return command(DRIVER_NUM_MAX17205, 4, 0, 0);
}

int max17205_read_status_sync(uint16_t* status) {
  int err;
  result.fired = false;

  err = max17205_set_callback(max17205_cb, (void*) &result);
  if (err < 0) return err;

  err = max17205_read_status();
  if (err < 0) return err;

  // Wait for the callback.
  yield_for(&result.fired);

  *status = result.value0 & 0xFFFF;

  return 0;
}

int max17205_read_soc_sync(uint16_t* percent, uint16_t* soc_mah, uint16_t* soc_mah_full) {
  int err;
  result.fired = false;

  err = max17205_set_callback(max17205_cb, (void*) &result);
  if (err < 0) return err;

  err = max17205_read_soc();
  if (err < 0) return err;

  // Wait for the callback.
  yield_for(&result.fired);

  *percent      = result.value0 & 0xFFFF;
  *soc_mah      = (result.value1 & 0xFFFF0000) >> 16;
  *soc_mah_full = result.value1 & 0xFFFF;

  return 0;
}

int max17205_read_voltage_current_sync(uint16_t* voltage, uint16_t* current) {
  int err;
  result.fired = false;

  err = max17205_set_callback(max17205_cb, (void*) &result);
  if (err < 0) return err;

  err = max17205_read_voltage_current();
  if (err < 0) return err;

  // Wait for the callback.
  yield_for(&result.fired);

  *voltage = result.value0 & 0xFFFF;
  *current = result.value1 & 0xFFFF;

  return 0;
}

int max17205_read_coulomb_sync(uint16_t* coulomb) {
  int err;
  result.fired = false;

  err = max17205_set_callback(max17205_cb, (void*) &result);
  if (err < 0) return err;

  err = max17205_read_coulomb();
  if (err < 0) return err;

  // Wait for the callback.
  yield_for(&result.fired);

  *coulomb = result.value0 & 0xFFFF;

  return 0;
}

float max17205_get_voltage_mV(int vcount) {
  return vcount * 1.25;
}

float max17205_get_current_uA(int ccount) {
  return ccount * 108;
}

float max17205_get_percentage_mP(int percent) {
  return ((float)percent / 26000.0) * 100000.0;
}

float max17205_get_capacity_uAh(int cap) {
  return (float)cap * (5.0 / .01);
}
