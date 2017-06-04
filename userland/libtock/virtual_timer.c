#include <stdlib.h>
#include <timer.h>
#include "virtual_timer.h"

struct virtual_timer {
  int expiration;
  subscribe_cb *callback;
  void* ud;
};

typedef struct {
  virtual_timer_t** data;
  int capacity;
  int size;
} heap_t;

static heap_t timer_heap = {
  .data = NULL,
  .capacity = 0,
  .size = 0,
};

static void heap_insert(virtual_timer_t* timer) {
  if (timer_heap.capacity - timer_heap.size <= 0) {
    // Heap too small! Make it bigger
    int new_capacity = (timer_heap.capacity + 1) * 2;
    timer_heap.data = (virtual_timer_t**)realloc(timer_heap.data,
        new_capacity * sizeof(virtual_timer_t*));
    timer_heap.capacity = new_capacity;
  }

  // insert it at the end...
  int idx = timer_heap.size;
  timer_heap.data[idx] = timer;
  timer_heap.size++;

  // then up-heap...
  while(idx != 0) {
    int parent_idx = (idx - 1) / 2;
    virtual_timer_t *parent = timer_heap.data[parent_idx];
    if (timer->expiration < parent->expiration) {
      timer_heap.data[idx] = parent;
      timer_heap.data[parent_idx] = timer;
      idx = parent_idx;
    } else {
      break;
    }
  }
}

static virtual_timer_t* heap_pop(void) {
  if (timer_heap.size == 0) {
    return NULL;
  }

  virtual_timer_t* ret = timer_heap.data[0];

  // swap leaft element to root
  timer_heap.size--;
  if (timer_heap.size == 0) {
    return ret;
  }
  virtual_timer_t *timer = timer_heap.data[timer_heap.size];
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

    virtual_timer_t* childl = timer_heap.data[childl_idx];
    virtual_timer_t* childr = timer_heap.data[childr_idx];

    if (timer->expiration <= childl->expiration &&
        timer->expiration <= childr->expiration) {
      break;
    } else if (childl->expiration < childr->expiration) {
      timer_heap.data[idx] = childl;
      timer_heap.data[childl_idx] = timer;
    } else {
      timer_heap.data[idx] = childr;
      timer_heap.data[childr_idx] = timer;
    }
  }

  return ret;
}

static virtual_timer_t* heap_peek(void) {
  if (timer_heap.size > 0) {
    return timer_heap.data[0];
  } else {
    return NULL;
  }
}

static void callback( int now,
                      __attribute__ ((unused)) int unused1,
                      __attribute__ ((unused)) int unused2,
                      __attribute__ ((unused)) void* ud) {
  virtual_timer_t* timer = heap_pop();
  if (timer == NULL) {
    return;
  }

  virtual_timer_t *next;
  for (next = heap_peek(); next != NULL && next->callback == NULL;
        next = heap_peek()) {
    free(heap_pop());
  }
  if (next != NULL) {
    timer_absolute(next->expiration);
  }

  if (timer->callback) {
    timer->callback(now, timer->expiration, 0, timer->ud);
  }
  free(timer);
}

virtual_timer_t *virtual_timer_start(int ms, subscribe_cb cb, void* ud) {
  virtual_timer_t *timer = (virtual_timer_t*)malloc(sizeof(virtual_timer_t));
  timer->expiration = ms;
  timer->callback = cb;
  timer->ud = ud;

  heap_insert(timer);

  if (heap_peek() == timer) {
    timer_subscribe(callback, NULL);
    timer_absolute(timer->expiration);
  }

  return timer;
}

void virtual_timer_cancel(virtual_timer_t* timer) {
  // Removing from a heap is tricky, so just remove the callback and let it get
  // lazily removed.
  timer->callback = NULL;
}

