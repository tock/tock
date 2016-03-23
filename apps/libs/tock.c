#include <inttypes.h>
#include <stdlib.h>
#include <unistd.h>
#include "tock.h"
#include "firestorm.h"

extern int __wait();
extern int __command(uint32_t, uint32_t, int);
extern int __allow();
extern int __subscribe();

struct callback_link {
  CB_TYPE result;
  struct callback_link *next;
};

typedef struct callback_link callback_link_t;

static callback_link_t *wait_queue_head = NULL;
static callback_link_t *wait_queue_tail = NULL;

int wait() {
  callback_link_t *cur = wait_queue_head;
  if (cur != NULL) {
    wait_queue_head = cur->next;
    if (wait_queue_head == NULL) {
      wait_queue_tail = NULL;
    }
    int result = cur->result;
    free(cur);
    return result;
  }
  return __wait();
}

int wait_for(CB_TYPE cb_type) {
  callback_link_t *prev = NULL;
  callback_link_t *cur = wait_queue_head;
  while (cur != NULL) {
    int result = cur->result;
    if (result == cb_type) {
      if (prev == NULL) {
        wait_queue_head = cur->next;
      } else {
        prev = cur->next;
      }
      if (cur->next == NULL) {
        wait_queue_tail = wait_queue_head;
      }
      free(cur);
      return result;
    } else {
      prev = cur;
      cur = cur->next;
    }
  }

  while(1) {
    CB_TYPE res_type = __wait();
    if (res_type == cb_type) {
      return res_type;
    }

    // Async callback. Store for later
    callback_link_t *cur =
      (callback_link_t*)malloc(sizeof(callback_link_t));
    cur->result = res_type;
    cur->next = NULL;
    if (wait_queue_tail == NULL) {
      wait_queue_tail = cur;
      wait_queue_head = cur;
    } else {
      wait_queue_tail->next = cur;
      wait_queue_tail = cur;
    }
  }
}

int subscribe(uint32_t driver, uint32_t subscribe,
              subscribe_cb cb, void* userdata) {
  return __subscribe(driver, subscribe, cb, userdata);
}


int command(uint32_t driver, uint32_t command, int data) {
  return __command(driver, command, data);
}

int allow(uint32_t driver, uint32_t allow, void* ptr, size_t size) {
  return __allow(driver, allow, ptr, size);
}
