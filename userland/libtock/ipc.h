#pragma once

#include <stdlib.h>
#include <string.h>

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

#define IPC_DRIVER_NUM 0x10000

// Performs service discovery
//
// Returns the process identifier of the process with the given package name,
// or a negative value on error.
int ipc_discover(const char* pkg_name);

// Registers a service callback for this process.
//
// Service callbacks are called in response to `notify`s from clients and take
// the following arguments in order:
//
//   int pid   - the notifying client's process id
//   int len   - the length of the shared buffer or zero if no buffer is shared
//               from the client.
//   char* buf - the base address of the shared buffer, or NULL if no buffer is
//               shared from the client.
//   void* ud  - `userdata`. same as the argument to this function.
int ipc_register_svc(subscribe_cb callback, void *ud);

// Registers a client callback for a particular service.
//
// `svc_id` is the (non-zero) process id of the service to subscribe to.
//
// Client callbacks are called in response to `notify`s from a particular
// service and take the following arguments in order:
//
//   int pid   - the notifying service's process id
//   int len   - the length of the shared buffer or zero if no buffer is shared
//               from the service.
//   char* buf - the base address of the shared buffer, or NULL if no buffer is
//               shared from the service.
//   void* ud  - `userdata`. same as the argument to this function.
int ipc_register_client_cb(int svc_id, subscribe_cb callback, void *ud);

// Send a notify to the client at the given process id
int ipc_notify_client(int pid);

// Send a notify to the service at the given process id
int ipc_notify_svc(int pid);

// Share a buffer with the given process (either service or client)
//
// `pid` is the non-zero process id of the recipient.
// `base` must be aligned to the value of `len`.
// `len` must be a power-of-two larger than 16.
int ipc_share(int pid, void* base, int len);

#ifdef __cplusplus
}
#endif
