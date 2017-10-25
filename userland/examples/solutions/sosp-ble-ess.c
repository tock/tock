/* XXX this "solution" should be fleshed out and report measurements from more sensors */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <timer.h>
#include <console.h>
#include <ambient_light.h>
#include <ipc.h>

int _svc_num = 0;

char ipc_buf[64] __attribute__((aligned(64)));

typedef enum {
  SENSOR_TEMPERATURE = 0,
  SENSOR_IRRADIANCE = 1,
  SENSOR_HUMIDITY = 2,
} sensor_type_e;

typedef struct {
  int type;  // sensor type
  int value; // sensor reading
} sensor_update_t;

static void ipc_callback(__attribute__ ((unused)) int pid,
                         __attribute__ ((unused)) int len,
                         __attribute__ ((unused)) int arg2,
                         __attribute__ ((unused)) void* ud) {
  printf("Updated BLE characteristic.\n");
}


char buf[300];

static void print_complete(int a __attribute__((unused)),
                           int b __attribute__((unused)),
                           int c __attribute__((unused)),
                           void* d __attribute__((unused)))
{
  // The message has been printed to the console
}

int main(void)
{
  int n = snprintf(buf, sizeof(buf), "From tock app: \"%s\"\n", "Hello, World!");
  putnstr_async(buf, n, print_complete, NULL);

  _svc_num = ipc_discover("org.tockos.services.ble-ess");
  if (_svc_num < 0) {
    printf("No BLE ESS service installed.\n");
    return -1;
  }

  printf("Found BLE ESS service (%i)\n", _svc_num);

  delay_ms(1500);

  sensor_update_t *update = (sensor_update_t*) ipc_buf;
  ipc_register_client_cb(_svc_num, ipc_callback, update);

  update->type = SENSOR_HUMIDITY;
  update->value = 185;
  ipc_share(_svc_num, ipc_buf, 64);

  ipc_notify_svc(_svc_num);

  return 0;
}
