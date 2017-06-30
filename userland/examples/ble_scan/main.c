#include <ble.h>
#include <led.h>
#include <stdio.h>
#include <string.h>
#include <timer.h>

/*
 * BLE Demo Application
 * Active scanner for BLE advertisements
 */

bool isDetected(void);
bool listFull(void);
bool validAdType(int);
void printList(void);

#define BUF_SIZE 39
#define MAX_DEVICES 50

struct Vector {
   unsigned char (*data)[BUF_SIZE];
   int size;
};

// global variables
unsigned char scan[BUF_SIZE] = {0};
unsigned char *data[MAX_DEVICES] = {0};
int last = 0;
struct Vector scan_list = {.data = &data[0], .size = 0};

static void callback(__attribute__((unused)) int unused0,
                     __attribute__((unused)) int unused1,
                     __attribute__((unused)) int unused2,
                     __attribute__((unused)) void* ud) {

    if(!listFull() && validAdType(scan[0]) && !isDetected()) {
        memcpy(scan_list.data[scan_list.size], scan, BUF_SIZE);
        scan_list.size += 1;
    }
    if (last != scan_list.size) {
        printList();
        last = scan_list.size;
    }
}

int main(void)
{
  int err;

  printf("BLE Scanner\r\n");

  // using the pre-configured adv interval
  err = ble_adv_scan(scan, BUF_SIZE, callback);
  if (err < 0) {
    printf("ble_adv_start error %d\r\n", err);
  }

  return 0;
}

bool isDetected(void) {
    for(int i = 0; i < scan_list.size; i++) {
        if(memcmp(scan_list.data[i], scan, BUF_SIZE) == 0) {
            return true;
        }
    }
    return false;
}

void printList(void) {
        printf("--------SCANNING--------------------------------------------\r\n");
        for(int i = 0; i < scan_list.size; i++)  {
            printf("DEVICE #%d\r\n", i);
            for(int j = 0; j < BUF_SIZE; j++) {
                printf("%02x ", scan_list.data[i][j]);
            }
            printf("\r\n");
        }
        printf("----------END------------------------------------------------\r\n\n");
}

bool validAdType(int type) {
    return type <= 0x06;
}

bool listFull(void) {
    return scan_list.size >= MAX_DEVICES;
}