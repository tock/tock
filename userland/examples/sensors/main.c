#include <stdbool.h>
#include <stdio.h>

#include <ambient_light.h>
#include <humidity.h>
#include <lps25hb.h>
#include <ninedof.h>
#include <temperature.h>
#include <timer.h>
#include <tmp006.h>
#include <tock.h>
#include <tsl2561.h>

static bool isl29035 = false;
// TODO: modify tmp006 to comply with the generic temperature interface
// and then remove tmp006!!!
static bool tmp006      = false;
static bool tsl2561     = false;
static bool lps25hb     = false;
static bool temperature = false;
static bool humidity    = false;
static bool ninedof     = false;

static void timer_fired(__attribute__ ((unused)) int arg0,
                        __attribute__ ((unused)) int arg1,
                        __attribute__ ((unused)) int arg2,
                        __attribute__ ((unused)) void* ud) {
  int light = 0;
  int16_t tmp006_temp  = 0;
  int tsl2561_lux      = 0;
  int lps25hb_pressure = 0;
  int temp = 0;
  unsigned humi = 0;
  int ninedof_x = 0, ninedof_y = 0, ninedof_z = 0;

  /* *INDENT-OFF* */
  if (isl29035)     light = ambient_light_read_intensity();
  if (tmp006)       tmp006_read_sync(&tmp006_temp);
  if (tsl2561)      tsl2561_lux = tsl2561_get_lux_sync();
  if (lps25hb)      lps25hb_pressure = lps25hb_get_pressure_sync();
  if (temperature)  temperature_read_sync(&temp);
  if (humidity)     humidity_read_sync(&humi);
  if (ninedof)      ninedof_read_acceleration_sync(&ninedof_x, &ninedof_y, &ninedof_z);

  if (isl29035)       printf("ISL29035:   Light Intensity: %d\n", light);
  if (tmp006)         printf("TMP006:     Temperature:     %d\n", tmp006_temp);
  if (tsl2561)        printf("TSL2561:    Light:           %d lux\n", tsl2561_lux);
  if (lps25hb)        printf("LPS25HB:    Pressure:        %d\n", lps25hb_pressure);
  if (temp)           printf("Temperature:                 %d deg C\n", temp/100);
  if (humi)           printf("Humidity:                    %u%%\n", humi/100);
  if (ninedof)        printf("FXOS8700CQ: X:               %d\n", ninedof_x);
  if (ninedof)        printf("FXOS8700CQ: Y:               %d\n", ninedof_y);
  if (ninedof)        printf("FXOS8700CQ: Z:               %d\n", ninedof_z);
  /* *INDENT-ON* */

  printf("\n");
}

int main(void) {
  printf("[Sensors] Starting Sensors App.\n");
  printf("[Sensors] All available sensors on the platform will be sampled.\n");

  isl29035    = driver_exists(DRIVER_NUM_AMBIENT_LIGHT);
  tmp006      = driver_exists(DRIVER_NUM_TMP006);
  tsl2561     = driver_exists(DRIVER_NUM_TSL2561);
  lps25hb     = driver_exists(DRIVER_NUM_LPS25HB);
  temperature = driver_exists(DRIVER_NUM_TEMPERATURE);
  humidity    = driver_exists(DRIVER_NUM_HUMIDITY);
  ninedof     = driver_exists(DRIVER_NUM_NINEDOF);

  // Setup periodic timer to sample the sensors.
  static tock_timer_t timer;
  timer_every(1000, timer_fired, NULL, &timer);

  return 0;
}
