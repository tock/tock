#include <stdio.h>
#include <stdbool.h>

#include <tock.h>
#include <timer.h>
#include <isl29035.h>
#include <tmp006.h>
#include <tsl2561.h>
#include <lps25hb.h>
#include <si7021.h>
#include <FXOS8700CQ.h>

static bool isl29035 = false;
static bool tmp006 = false;
static bool tsl2561 = false;
static bool lps25hb = false;
static bool si7021 = false;
static bool fxos8700cq = false;

static void timer_fired(__attribute__ ((unused)) int arg0,
                 __attribute__ ((unused)) int arg1,
                 __attribute__ ((unused)) int arg2,
                 __attribute__ ((unused)) void* ud) {
  int light;
  int16_t tmp006_temp;
  int tsl2561_lux;
  int lps25hb_pressure;
  int si7021_temp;
  unsigned si7021_humi;
  int fxos8700cq_x, fxos8700cq_y, fxos8700cq_z;

  if (isl29035)   light = isl29035_read_light_intensity();
  if (tmp006)     tmp006_read_sync(&tmp006_temp);
  if (tsl2561)    tsl2561_lux = tsl2561_get_lux_sync();
  if (lps25hb)    lps25hb_pressure = lps25hb_get_pressure_sync();
  if (si7021)     si7021_get_temperature_humidity_sync(&si7021_temp, &si7021_humi);
  if (fxos8700cq) FXOS8700CQ_read_acceleration_sync(&fxos8700cq_x, &fxos8700cq_y, &fxos8700cq_z);


  if (isl29035)   printf("ISL29035:   Light Intensity: %d\n", light);
  if (tmp006)     printf("TMP006:     Temperature:     %d\n", tmp006_temp);
  if (tsl2561)    printf("TSL2561:    Light:           %d lux\n", tsl2561_lux);
  if (lps25hb)    printf("LPS25HB:    Pressure:        %d\n", lps25hb_pressure);
  if (si7021)     printf("SI7021:     Temperature:     %d deg C\n", si7021_temp/100);
  if (si7021)     printf("SI7021:     Humidity:        %u%%\n", si7021_humi/100);
  if (fxos8700cq) printf("FXOS8700CQ: X:               %d\n", fxos8700cq_x);
  if (fxos8700cq) printf("FXOS8700CQ: Y:               %d\n", fxos8700cq_y);
  if (fxos8700cq) printf("FXOS8700CQ: Z:               %d\n", fxos8700cq_z);

  printf("\n");
}

int main(void) {
  printf("[Sensors] Starting Sensors App.\n");
  printf("[Sensors] All available sensors on the platform will be sampled.\n");

  isl29035 = driver_exists(DRIVER_NUM_ISL29035);
  tmp006 = driver_exists(DRIVER_NUM_TMP006);
  tsl2561 = driver_exists(DRIVER_NUM_TSL2561);
  lps25hb = driver_exists(DRIVER_NUM_LPS25HB);
  si7021 = driver_exists(DRIVER_NUM_SI7021);
  fxos8700cq = driver_exists(DRIVER_NUM_FXO);

  // Setup periodic timer to sample the sensors.
  timer_subscribe(timer_fired, NULL);
  timer_start_repeating(1000);

  return 0;
}
