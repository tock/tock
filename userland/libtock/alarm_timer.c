#include "alarm.h"
#include "internal/alarm.h"
#include "timer.h"
#include <limits.h>
#include <stdlib.h>

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
  alarm_t* next;
  alarm_t* prev;
};

static alarm_t* root = NULL;

static int root_insert(alarm_t* alarm) {
  if (root == NULL) {
    root = alarm;
    root->next = NULL;
    root->prev = NULL;
  }
  alarm_t **cur = &root;
  alarm_t *prev = NULL;
  while (*cur != NULL) {
    if (cmp_exp(alarm->t0, alarm->expiration, (*cur)->expiration) < 0) {
      // insert before
      alarm_t *tmp = *cur;
      *cur = alarm;
      alarm->next = tmp;
      alarm->prev = prev;
      tmp->prev = alarm;
    }
    prev = *cur;
    cur = &(*cur)->next;
  }
  return 0;
}

static alarm_t* root_pop(uint32_t now) {
  if (root == NULL) {
    return NULL;
  } else {
    alarm_t *res = root;
    root = root->next;
    if (root != NULL) {
      root->prev = NULL;
    }
    res->next = NULL;
    return res;
  }
}

static alarm_t* root_peek(void) {
  return root;
}

static void callback( __attribute__ ((unused)) int unused0,
                      __attribute__ ((unused)) int unused1,
                      __attribute__ ((unused)) int unused2,
                      __attribute__ ((unused)) void* ud) {
  for (alarm_t* alarm = root_peek(); alarm != NULL; alarm = root_peek()) {
    uint32_t now = alarm_read();
    // has the alarm not expired yet? (distance from `now` has to be larger or
    // equal to distance from current clock value.
    if (alarm->expiration - alarm->t0 > now - alarm->t0) {
      alarm_internal_absolute(alarm->expiration);
      break;
    } else {
      root_pop(now);

      if (alarm->callback) {
        tock_enqueue(alarm->callback, now, alarm->expiration, 0, alarm->ud);
      }
      free(alarm);
    }
  }
}

alarm_t *alarm_at(uint32_t expiration, subscribe_cb cb, void* ud) {
  alarm_t *alarm = (alarm_t*)malloc(sizeof(alarm_t));
  if (alarm == NULL) {
    return NULL;
  }
  alarm->t0         = alarm_read();
  alarm->expiration = expiration;
  alarm->callback   = cb;
  alarm->ud         = ud;
  alarm->prev       = NULL;
  alarm->next       = NULL;

  if (root_insert(alarm) != 0) {
    return NULL;
  }

  if (root_peek() == alarm) {
    alarm_internal_subscribe((subscribe_cb*)callback, NULL);
    alarm_internal_absolute(alarm->expiration);
  }

  return alarm;
}

void alarm_cancel(alarm_t* alarm) {
  if (alarm->prev != NULL) {
    alarm->prev->next = alarm->next;
  }
  if (alarm->next != NULL) {
    alarm->next->prev = alarm->prev;
  }

  if (root == alarm) {
    root = alarm->next;
    if (root != NULL) {
      alarm_internal_absolute(root->expiration);
    }
  }

  alarm->prev = NULL;
  alarm->next = NULL;

}

uint32_t alarm_read(void) {
  return (uint32_t) command(3, 4, 0);
}

// Timer implementation

struct timer_repeating {
  uint32_t interval;
  subscribe_cb* cb;
  void* ud;
  alarm_t* alarm;
};


alarm_t* timer_in(uint32_t ms, subscribe_cb cb, void* ud) {
  uint32_t interval   = ms * alarm_internal_frequency() / 1000;
  uint32_t now        = alarm_read();
  uint32_t expiration = now + interval;
  return alarm_at(expiration, cb, ud);
}

static void repeating_cb( uint32_t now,
                          __attribute__ ((unused)) int unused1,
                          __attribute__ ((unused)) int unused2,
                          void* ud) {
  timer_repeating_t* repeating = (timer_repeating_t*)ud;
  uint32_t interval   = repeating->interval;
  uint32_t expiration = now + interval;
  uint32_t cur_exp    = repeating->alarm->expiration;
  repeating->alarm = alarm_at(expiration, (subscribe_cb*)repeating_cb,
                              (void*)repeating);
  repeating->cb(now, cur_exp, 0, repeating->ud);
}

timer_repeating_t* timer_every(uint32_t ms, subscribe_cb cb, void* ud) {
  uint32_t interval = ms * alarm_internal_frequency() / 1000;

  timer_repeating_t* repeating =
    (timer_repeating_t*)malloc(sizeof(timer_repeating_t));
  if (repeating == NULL) {
    return NULL;
  }
  repeating->interval = interval;
  repeating->cb       = cb;
  repeating->ud       = ud;

  uint32_t now        = alarm_read();
  uint32_t expiration = now + interval;

  repeating->alarm = alarm_at(expiration, (subscribe_cb*)repeating_cb,
                              (void*)repeating);
  return (void*)repeating;
}

void timer_cancel(timer_repeating_t* timer) {
  alarm_cancel(timer->alarm);
  free(timer->alarm);
}

void delay_ms(uint32_t ms) {
  void delay_cb(__attribute__ ((unused)) int unused0,
                __attribute__ ((unused)) int unused1,
                __attribute__ ((unused)) int unused2,
                void* ud) {
    *((bool*)ud) = true;
  }

  bool cond = false;
  timer_in(ms, delay_cb, &cond);
  yield_for(&cond);
}

int yield_for_with_timeout(bool* cond, uint32_t ms) {
  void yield_for_timeout_cb(__attribute__ ((unused)) int unused0,
                            __attribute__ ((unused)) int unused1,
                            __attribute__ ((unused)) int unused2,
                            void* ud) {
    *((bool*)ud) = true;
  }

  bool timeout = false;
  alarm_t* a   = timer_in(ms, yield_for_timeout_cb, &timeout);

  while (!*cond) {
    if (timeout) {
      return TOCK_FAIL;
    }

    yield();
  }

  alarm_cancel(a);
  free(a);
  return TOCK_SUCCESS;
}
