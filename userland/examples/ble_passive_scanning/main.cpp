#include <cstring>
#include <forward_list>
#include <simple_ble.h>
#include <stdio.h>
#include <string.h>

#include "advertisement.h"
#include "advertisement_list.h"

/*
 * BLE Demo Application
 * Passive scanner for Bluetooth Low Energy advertisements
 */


const int BUF_SIZE = 39;
static unsigned char scan[BUF_SIZE];
AdvertisementList list;

static void callback(int result, int len, __attribute__((unused)) int unused2,
                     __attribute__((unused)) void* ud)
{
  if (result == TOCK_SUCCESS) {
    if (Advertisement::validAdvertisement(scan, len)) {
      Advertisement adv(scan, len);

      // new device detected
      if (!list.containsDevice(adv)) {
        if (list.add(adv)) {
          list.printList();
        }
      }

      // FIXME: add this to get dynamic behavior i.e, update every time new advertisement is detected
      // but might it fill the print buffer, use at your own risk
      // else {
      //   list.newData(adv);
      // }
    }
  }
}

int main(void)
{
  printf("\rBLE Passive Scanner\r\n");

  // using the pre-configured adv interval
  int err = ble_start_passive_scan(scan, BUF_SIZE, callback);

  if (err < TOCK_SUCCESS) {
    printf("ble_start_passive_scan_wip, error: %s\r\n", tock_strerror(err));
  }
  return 0;
}
