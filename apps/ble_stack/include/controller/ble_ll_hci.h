/**
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *  http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

#ifndef H_BLE_LL_HCI_
#define H_BLE_LL_HCI_

/* For supported commands */
#define BLE_LL_SUPP_CMD_LEN (36)
extern const uint8_t g_ble_ll_supp_cmds[BLE_LL_SUPP_CMD_LEN];

/*
 * This determines the number of outstanding commands allowed from the
 * host to the controller.
 */
#define BLE_LL_CFG_NUM_HCI_CMD_PKTS     (1)

/* Initialize LL HCI */
void ble_ll_hci_init(void);

/* HCI command processing function */
void ble_ll_hci_cmd_proc(struct os_event *ev);

/* Used to determine if the LE event is enabled/disabled */
uint8_t ble_ll_hci_is_le_event_enabled(int subev);

/* Used to determine if event is enabled/disabled */
uint8_t ble_ll_hci_is_event_enabled(int evcode);

/* Send event from controller to host */
int ble_ll_hci_event_send(uint8_t *evbuf);

/* Sends a command complete with a no-op opcode to host */
int ble_ll_hci_send_noop(void);


#endif /* H_BLE_LL_HCI_ */
