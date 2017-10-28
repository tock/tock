#include <stdbool.h>
#include <stdio.h>

#include <timer.h>
#include <ambient_light.h>
#include <temperature.h>
#include <humidity.h>
#include <ninedof.h>
#include <led.h>
#include <ipc.h>

int _svc_num = 0;

char buf[64] __attribute__((aligned(64)));

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

int main(void)
{
  _svc_num = ipc_discover("org.tockos.services.ble-ess");
  if (_svc_num < 0) {
    printf("No BLE ESS service installed.\n");
    return -1;
  }

  printf("Found BLE ESS service (%i)\n", _svc_num);

  delay_ms(1500);

  sensor_update_t *update = (sensor_update_t*) buf;
  ipc_register_client_cb(_svc_num, ipc_callback, update);
  ipc_share(_svc_num, buf, 64);

  while (true) {
    int lux;
    ambient_light_read_intensity_sync(&lux);
    update->type = SENSOR_IRRADIANCE;
    update->value = lux;
    ipc_notify_svc(_svc_num);

    int temp;
    temperature_read_sync(&temp);
    update->type = SENSOR_TEMPERATURE;
    update->value = temp;
    ipc_notify_svc(_svc_num);

    unsigned humi;
    humidity_read_sync(&humi);
    update->type = SENSOR_HUMIDITY;
    update->value = humi;
    ipc_notify_svc(_svc_num);

    delay_ms(1000);
  }
}

