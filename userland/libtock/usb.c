#include "usb.h"

int usb_exists(void) {
  return command(DRIVER_NUM_USB, 0, 0, 0) >= 0;
}

int usb_subscribe(subscribe_cb callback, void *ud) {
  return subscribe(DRIVER_NUM_USB, 0, callback, ud);
}

int usb_enable_and_attach_async(void) {
  return command(DRIVER_NUM_USB, 1, 0, 0);
}

struct data {
  bool fired;
  int status;
};

static void callback(int status,
                     __attribute__((unused)) int v1,
                     __attribute__((unused)) int v2,
                     void *data)
{
  struct data *d = data;
  d->fired  = true;
  d->status = status;
}

int usb_enable_and_attach(void)
{
  int status;

  struct data d = { .fired = false };

  if ((status = usb_subscribe(callback, (void *) &d)) != TOCK_SUCCESS) {
    return status;
  }

  if ((status = usb_enable_and_attach_async()) != TOCK_SUCCESS) {
    return status;
  }

  yield_for(&d.fired);
  return d.status;
}
