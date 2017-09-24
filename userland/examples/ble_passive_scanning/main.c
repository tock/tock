#include <simple_ble.h>
#include <stdio.h>
#include <string.h>

/*
 * BLE Demo Application
 * Active scanner for BLE advertisements
 */

bool isDetected(void);
bool listFull(void);
bool validAdType(unsigned char);
void printList(void);

#define BUF_SIZE 39
#define MAX_DEVICES 50

struct Vector {
  unsigned char (*data)[BUF_SIZE];
  int size;
};

// global variables
unsigned char scan[BUF_SIZE]     = { 0 };
unsigned char* data[MAX_DEVICES] = { 0 };
int last = 0;
struct Vector scan_list = {.data = (unsigned char (*)[BUF_SIZE])data, .size = 0 };

static void callback(__attribute__((unused)) int unused0,
                     __attribute__((unused)) int unused1, __attribute__((unused)) int unused2,
                     __attribute__((unused)) void* ud)
{

  if (!listFull() && validAdType(scan[8]) && !isDetected()) {
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
  printf("\rBLE Passive Scanner\r\n\n");

  // using the pre-configured adv interval
  int err = ble_start_passive_scan(scan, BUF_SIZE, callback);
  if (err < TOCK_SUCCESS) {
    printf("ble_start_passive_scan_wip, error: %s\r\n", tock_strerror(err));
  }

  return 0;
}

bool isDetected(void)
{
  for (int i = 0; i < scan_list.size; i++) {
    // only compare address
    if (memcmp(scan_list.data[i], scan, 8) == 0) {
      return true;
    }
  }
  return false;
}

void printList(void)
{
  printf("--------DETECTED DEVICES--------------------------------------------\r\n");
  for (int i = 0; i < scan_list.size; i++) {
    printf("DEVICE ADDRESS: %02x %02x %02x %02x %02x %02x %02x %02x\r\n",
           scan_list.data[i][7], scan_list.data[i][6], scan_list.data[i][5],
           scan_list.data[i][4], scan_list.data[i][3], scan_list.data[i][2],
           scan_list.data[i][1], scan_list.data[i][0]);
    printf("DATA: ");
    for (int j = 8; j < BUF_SIZE; j++) {
      printf("%02x ", scan_list.data[i][j]);
    }
    printf("\r\n\n");
  }
  printf("---------------------------------------------------------------------\r\n\n");
}

bool validAdType(unsigned char type) {
  return type <= 0x06;
}

bool listFull(void) {
  return scan_list.size >= MAX_DEVICES;
}
