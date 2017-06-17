#include <limits.h>
#include <stdlib.h>
#include "internal/alarm.h"
#include "alarm.h"

// Returns < 0 if exp0 is earlier, > 0 if exp1 is earlier, and 0
// if they are equal.
static int cmp_exp(uint32_t now, uint32_t exp0, uint32_t exp1) {
  return (exp0 - now) - (exp1 - now);
}

struct alarm {
  uint32_t t0;
  uint32_t expiration;
  subscribe_cb *callback;
  void* ud;
};

typedef struct {
  alarm_t** data;
  int capacity;
  int size;
} heap_t;

static heap_t alarm_heap = {
  .data = NULL,
  .capacity = 0,
  .size = 0,
};

static void heap_insert(alarm_t* alarm) {
  if (alarm_heap.capacity - alarm_heap.size <= 0) {
    // Heap too small! Make it bigger
    int new_capacity = (alarm_heap.capacity + 1) * 2;
    alarm_heap.data = (alarm_t**)realloc(alarm_heap.data,
        new_capacity * sizeof(alarm_t*));
    alarm_heap.capacity = new_capacity;
  }

  // insert it at the end...
  int idx = alarm_heap.size;
  alarm_heap.data[idx] = alarm;
  alarm_heap.size++;

  // then up-heap...
  while(idx != 0) {
    int parent_idx = (idx - 1) / 2;
    alarm_t *parent = alarm_heap.data[parent_idx];
    if (cmp_exp(alarm->t0, alarm->expiration, parent->expiration) < 0) {
      alarm_heap.data[idx] = parent;
      alarm_heap.data[parent_idx] = alarm;
      idx = parent_idx;
    } else {
      break;
    }
  }
}

static alarm_t* heap_pop(uint32_t now) {
  if (alarm_heap.size == 0) {
    return NULL;
  }

  alarm_t* ret = alarm_heap.data[0];

  // swap leaf element to root
  alarm_heap.size--;
  if (alarm_heap.size == 0) {
    return ret;
  }
  alarm_t *alarm = alarm_heap.data[alarm_heap.size];
  alarm_heap.data[0] = alarm;

  // sift-down
  int idx = 0;
  while (idx < alarm_heap.size) {
    int childl_idx = (idx + 1) * 2;
    int childr_idx = (idx + 2) * 2;

    if (childl_idx >= alarm_heap.size) {
      childl_idx = idx;
    }
    if (childr_idx >= alarm_heap.size) {
      childr_idx = idx;
    }

    alarm_t* childl = alarm_heap.data[childl_idx];
    alarm_t* childr = alarm_heap.data[childr_idx];

    if (cmp_exp(now, alarm->expiration, childl->expiration) <= 0 &&
        cmp_exp(now, alarm->expiration, childr->expiration) <= 0) {
      break;
    } else if (cmp_exp(now, childl->expiration, childr->expiration) < 0) {
      alarm_heap.data[idx] = childl;
      alarm_heap.data[childl_idx] = alarm;
    } else {
      alarm_heap.data[idx] = childr;
      alarm_heap.data[childr_idx] = alarm;
    }
  }

  return ret;
}

static alarm_t* heap_peek(void) {
  if (alarm_heap.size > 0) {
    return alarm_heap.data[0];
  } else {
    return NULL;
  }
}

static void callback( __attribute__ ((unused)) int unused0,
                      __attribute__ ((unused)) int unused1,
                      __attribute__ ((unused)) int unused2,
                      __attribute__ ((unused)) void* ud) {
  int i = 0;
  for (alarm_t* alarm = heap_peek(); alarm != NULL; alarm = heap_peek()) {
    i++;
    uint32_t now = alarm_read();
    // has the alarm not expired yet? (distance from `now` has to be larger or
    // equal to distance from current clock value.
    if (alarm->expiration - alarm->t0 > now - alarm->t0) {
      alarm_internal_absolute(alarm->expiration);
      break;
    } else {
      heap_pop(now);

      if (alarm->callback) {
        // alarm->callback callback _could_ yield, which might result in
        // reentering this function.
        tock_enqueue(alarm->callback, now, alarm->expiration, 0, alarm->ud);
        //alarm->callback(now, alarm->expiration, 0, alarm->ud);
      }
      free(alarm);
    }
  }
}

alarm_t *alarm_at(uint32_t expiration, subscribe_cb cb, void* ud) {
  alarm_t *alarm = (alarm_t*)malloc(sizeof(alarm_t));
  alarm->t0 = alarm_read();
  alarm->expiration = expiration;
  alarm->callback = cb;
  alarm->ud = ud;

  heap_insert(alarm);

  if (heap_peek() == alarm) {
    alarm_internal_subscribe((subscribe_cb*)callback, NULL);
    alarm_internal_absolute(alarm->expiration);
  }

  return alarm;
}

alarm_t* alarm_in(uint32_t ms, subscribe_cb cb, void* ud) {
  uint32_t interval = ms * alarm_internal_frequency() / 1000;
  uint32_t now = alarm_read();
  uint32_t expiration = now + interval;
  return alarm_at(expiration, cb, ud);
}

struct alarm_repeating {
  uint32_t interval;
  subscribe_cb* cb;
  void* ud;
  alarm_t* alarm;
};

static void repeating_cb( uint32_t now,
                      __attribute__ ((unused)) int unused1,
                      __attribute__ ((unused)) int unused2,
                      void* uud) {
  alarm_repeating_t* udwrapper = (alarm_repeating_t*)uud;
  uint32_t interval = udwrapper->interval;
  uint32_t expiration = now + interval;
  uint32_t cur_exp = udwrapper->alarm->expiration;
  udwrapper->alarm = alarm_at(expiration, (subscribe_cb*)repeating_cb, (void*)udwrapper);
  udwrapper->cb(now, cur_exp, 0, udwrapper->ud);
}

alarm_repeating_t* alarm_every(uint32_t ms, subscribe_cb cb, void* ud) {
  uint32_t interval = ms * alarm_internal_frequency() / 1000;

  alarm_repeating_t* uud = (alarm_repeating_t*)malloc(sizeof(alarm_repeating_t));
  uud->interval = interval;
  uud->cb = cb;
  uud->ud = ud;

  uint32_t now = alarm_read();
  uint32_t expiration = now + interval;

  uud->alarm = alarm_at(expiration, (subscribe_cb*)repeating_cb, (void*)uud);
  return (void*)uud;
}

void alarm_cancel(alarm_t* alarm) {
  // Removing from a heap is tricky, so just remove the callback and let it get
  // lazily removed.
  alarm->callback = NULL;
}

uint32_t alarm_read(void) {
  return (uint32_t) command(3, 4, 0);
}

void delay_ms(uint32_t ms) {
  void delay_cb(__attribute__ ((unused)) int unused0,
                __attribute__ ((unused)) int unused1,
                __attribute__ ((unused)) int unused2,
                void* ud) {
    *((bool*)ud) = true;
  }

  bool cond = false;
  alarm_in(ms, delay_cb, &cond);
  yield_for(&cond);
}

