#include "ipc.h"

int ipc_discover(const char* pkg_name) {
  int len = strlen(pkg_name);
  char* pkg_name_buf = (char*)malloc(len * sizeof(char));
  memcpy(pkg_name_buf, pkg_name, len);
  int res = allow(IPC_DRIVER_NUM, 0, pkg_name_buf, len);
  free(pkg_name_buf);
  return res;
}

int ipc_register_svc(subscribe_cb callback, void *ud) {
  return subscribe(IPC_DRIVER_NUM, 0, callback, ud);
}

int ipc_register_client_cb(int svc_id, subscribe_cb callback, void *ud) {
  if (svc_id <= 0) {
    return -1;
  }
  return subscribe(IPC_DRIVER_NUM, svc_id, callback, ud);
}

int ipc_notify_svc(int pid) {
  return command(IPC_DRIVER_NUM, pid, 0, 0);
}

int ipc_notify_client(int pid) {
  return command(IPC_DRIVER_NUM, pid, 1, 0);
}

int ipc_share(int pid, void* base, int len) {
  if (pid <= 0) {
    return -1;
  }
  return allow(IPC_DRIVER_NUM, pid, base, len);
}

