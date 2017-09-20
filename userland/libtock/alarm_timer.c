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

static alarm_t* root = NULL;

static void root_insert(alarm_t* alarm) {
  if (root == NULL) {
    root       = alarm;
    root->next = NULL;
    root->prev = NULL;
    return;
  }

  alarm_t **cur = &root;
  alarm_t *prev = NULL;
  while (*cur != NULL) {
    if (cmp_exp(alarm->t0, alarm->expiration, (*cur)->expiration) < 0) {
      // insert before
      alarm_t *tmp = *cur;
      *cur        = alarm;
      alarm->next = tmp;
      alarm->prev = prev;
      tmp->prev   = alarm;
      return;
    }
    prev = *cur;
    cur  = &prev->next;
  }
  // didn't return, so prev points to the last in the list
  prev->next  = alarm;
  alarm->prev = prev;
  alarm->next = NULL;

}

static alarm_t* root_pop(void) {
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
      alarm_internal_set(alarm->expiration);
      break;
    } else {
      root_pop();

      if (alarm->callback) {
        tock_enqueue(alarm->callback, now, alarm->expiration, 0, alarm->ud);
      }
    }
  }
}

void alarm_at(uint32_t expiration, subscribe_cb cb, void* ud, alarm_t* alarm) {
  alarm->t0         = alarm_read();
  alarm->expiration = expiration;
  alarm->callback   = cb;
  alarm->ud         = ud;
  alarm->prev       = NULL;
  alarm->next       = NULL;

  root_insert(alarm);
  int i = 0;
  for (alarm_t* cur = root_peek(); cur != NULL; cur = cur->next) {
    i++;
  }

  if (root_peek() == alarm) {
    alarm_internal_subscribe((subscribe_cb*)callback, NULL);
    alarm_internal_set(alarm->expiration);
  }
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
      alarm_internal_set(root->expiration);
    }
  }

  alarm->prev = NULL;
  alarm->next = NULL;

}

uint32_t alarm_read(void) {
  return (uint32_t) command(DRIVER_NUM_ALARM, 2, 0);
}

// Timer implementation

void timer_in(uint32_t ms, subscribe_cb cb, void* ud, tock_timer_t *timer) {
  uint32_t frequency  = alarm_internal_frequency();
  uint32_t interval   = (ms / 1000) * frequency + (ms % 1000) * (frequency / 1000);
  uint32_t now        = alarm_read();
  uint32_t expiration = now + interval;
  alarm_at(expiration, cb, ud, &timer->alarm);
}

static void repeating_cb( uint32_t now,
                          __attribute__ ((unused)) int unused1,
                          __attribute__ ((unused)) int unused2,
                          void* ud) {
  tock_timer_t* repeating = (tock_timer_t*)ud;
  uint32_t interval       = repeating->interval;
  uint32_t expiration     = now + interval;
  uint32_t cur_exp        = repeating->alarm.expiration;
  alarm_at(expiration, (subscribe_cb*)repeating_cb,
           (void*)repeating, &repeating->alarm);
  repeating->cb(now, cur_exp, 0, repeating->ud);
}

void timer_every(uint32_t ms, subscribe_cb cb, void* ud, tock_timer_t* repeating) {
  uint32_t frequency = alarm_internal_frequency();
  uint32_t interval  = (ms / 1000) * frequency + (ms % 1000) * (frequency / 1000);

  repeating->interval = interval;
  repeating->cb       = cb;
  repeating->ud       = ud;

  uint32_t now        = alarm_read();
  uint32_t expiration = now + interval;

  alarm_at(expiration, (subscribe_cb*)repeating_cb,
           (void*)repeating, &repeating->alarm);
}

void timer_cancel(tock_timer_t* timer) {
  alarm_cancel(&timer->alarm);
}

void delay_ms(uint32_t ms) {
  void delay_cb(__attribute__ ((unused)) int unused0,
                __attribute__ ((unused)) int unused1,
                __attribute__ ((unused)) int unused2,
                void* ud) {
    *((bool*)ud) = true;
  }

  bool cond = false;
  tock_timer_t timer;
  timer_in(ms, delay_cb, &cond, &timer);
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
  tock_timer_t timer;
  timer_in(ms, yield_for_timeout_cb, &timeout, &timer);

  while (!*cond) {
    if (timeout) {
      return TOCK_FAIL;
    }

    yield();
  }

  timer_cancel(&timer);
  return TOCK_SUCCESS;
}
