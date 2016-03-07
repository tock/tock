#include <inttypes.h>
#include <stdlib.h>
#include <unistd.h>
#include "tock.h"

extern int __wait();
extern int __command(uint32_t, uint32_t, int);
extern int __allow();
extern int __subscribe();

struct callback_link {
  CB_TYPE result;
  struct callback_link *next;
};

typedef struct callback_link callback_link_t;

callback_link_t *wait_queue_head = NULL;
callback_link_t *wait_queue_tail = NULL;

int wait() {
  if (wait_queue_head != NULL) {
    callback_link_t *cur = wait_queue_head;
    wait_queue_head = wait_queue_head->next;
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
  while(1) {
    CB_TYPE type = __wait();
    if (type == cb_type) {
      return type;
    }

    // Async callback. Store for later
    callback_link_t *cur =
      (callback_link_t*)malloc(sizeof(callback_link_t));
    cur->result = type;
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
