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
  uint32_t expiration;
  subscribe_cb *callback;
  void* ud;
};

typedef struct {
  alarm_t** data;
  int capacity;
  int size;
} heap_t;

static heap_t timer_heap = {
  .data = NULL,
  .capacity = 0,
  .size = 0,
};

static void heap_insert(uint32_t now, alarm_t* timer) {
  if (timer_heap.capacity - timer_heap.size <= 0) {
    // Heap too small! Make it bigger
    int new_capacity = (timer_heap.capacity + 1) * 2;
    timer_heap.data = (alarm_t**)realloc(timer_heap.data,
        new_capacity * sizeof(alarm_t*));
    timer_heap.capacity = new_capacity;
  }

  // insert it at the end...
  int idx = timer_heap.size;
  timer_heap.data[idx] = timer;
  timer_heap.size++;

  // then up-heap...
  while(idx != 0) {
    int parent_idx = (idx - 1) / 2;
    alarm_t *parent = timer_heap.data[parent_idx];
    if (cmp_exp(now, timer->expiration, parent->expiration) < 0) {
      timer_heap.data[idx] = parent;
      timer_heap.data[parent_idx] = timer;
      idx = parent_idx;
    } else {
      break;
    }
  }
}

static alarm_t* heap_pop(uint32_t now) {
  if (timer_heap.size == 0) {
    return NULL;
  }

  alarm_t* ret = timer_heap.data[0];

  // swap leaft element to root
  timer_heap.size--;
  if (timer_heap.size == 0) {
    return ret;
  }
  alarm_t *timer = timer_heap.data[timer_heap.size];
  timer_heap.data[0] = timer;

  // sift-down
  int idx = 0;
  while (idx < timer_heap.size) {
    int childl_idx = (idx + 1) * 2;
    int childr_idx = (idx + 2) * 2;

    if (childl_idx >= timer_heap.size) {
      childl_idx = idx;
    }
    if (childr_idx >= timer_heap.size) {
      childr_idx = idx;
    }

    alarm_t* childl = timer_heap.data[childl_idx];
    alarm_t* childr = timer_heap.data[childr_idx];

    if (cmp_exp(now, timer->expiration, childl->expiration) <= 0 &&
        cmp_exp(now, timer->expiration, childr->expiration) <= 0) {
      break;
    } else if (cmp_exp(now, childl->expiration, childr->expiration) < 0) {
      timer_heap.data[idx] = childl;
      timer_heap.data[childl_idx] = timer;
    } else {
      timer_heap.data[idx] = childr;
      timer_heap.data[childr_idx] = timer;
    }
  }

  return ret;
}

static alarm_t* heap_peek(void) {
  if (timer_heap.size > 0) {
    return timer_heap.data[0];
  } else {
    return NULL;
  }
}

static void callback( uint32_t now,
                      __attribute__ ((unused)) int unused1,
                      __attribute__ ((unused)) int unused2,
                      __attribute__ ((unused)) void* ud) {
  alarm_t* timer = heap_pop(now);
  if (timer == NULL) {
    return;
  }

  alarm_t *next;
  for (next = heap_peek(); next != NULL && next->callback == NULL;
        next = heap_peek()) {
    free(heap_pop(now));
  }
  if (next != NULL) {
    alarm_internal_absolute(next->expiration);
  }

  if (timer->callback) {
    timer->callback(now, timer->expiration, 0, timer->ud);
  }
  free(timer);
}

alarm_t *alarm_start(uint32_t expiration, subscribe_cb cb, void* ud) {
  alarm_t *timer = (alarm_t*)malloc(sizeof(alarm_t));
  timer->expiration = expiration;
  timer->callback = cb;
  timer->ud = ud;

  uint32_t now = alarm_internal_read();

  heap_insert(now, timer);

  if (heap_peek() == timer) {
    alarm_internal_subscribe((subscribe_cb*)callback, NULL);
    alarm_internal_absolute(timer->expiration);
  }

  return timer;
}

alarm_t* alarm_in(uint32_t ms, subscribe_cb cb, void* ud) {
  uint32_t now = alarm_internal_read();
  uint32_t expiration = INT_MAX - now >= ms ? now + ms : ms - (INT_MAX - now);
  return alarm_start(expiration, cb, ud);
}

struct alarm_repeating {
  uint32_t interval;
  subscribe_cb* cb;
  void* ud;
  alarm_t* timer;
};

static void repeating_cb( uint32_t now,
                      __attribute__ ((unused)) int unused1,
                      __attribute__ ((unused)) int unused2,
                      void* uud) {
  alarm_repeating_t* udwrapper = (alarm_repeating_t*)uud;
  uint32_t ms = udwrapper->interval;
  uint32_t expiration = now + ms;
  uint32_t cur_exp = udwrapper->timer->expiration;
  udwrapper->timer = alarm_start(expiration, (subscribe_cb*)repeating_cb, (void*)udwrapper);
  udwrapper->cb(now, cur_exp, 0, udwrapper->ud);
}

alarm_repeating_t* alarm_every(uint32_t ms, subscribe_cb cb, void* ud) {
  uint32_t now = alarm_internal_read();
  uint32_t expiration = now + ms;
  alarm_repeating_t* uud = (alarm_repeating_t*)malloc(sizeof(alarm_repeating_t));
  uud->interval = ms;
  uud->cb = cb;
  uud->ud = ud;
  uud->timer = alarm_start(expiration, (subscribe_cb*)repeating_cb, (void*)uud);
  return (void*)uud;
}

void alarm_cancel(alarm_t* timer) {
  // Removing from a heap is tricky, so just remove the callback and let it get
  // lazily removed.
  timer->callback = NULL;
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

