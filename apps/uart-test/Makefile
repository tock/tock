PROJECT_NAME = $(shell basename "$(realpath ./)")

APPLICATION_SRCS = $(notdir $(wildcard ./*.c))
APPLICATION_SRCS += softdevice_handler.c
APPLICATION_SRCS += ble_advdata.c
APPLICATION_SRCS += ble_conn_params.c
APPLICATION_SRCS += app_timer.c
APPLICATION_SRCS += app_error.c
APPLICATION_SRCS += ble_srv_common.c
APPLICATION_SRCS += app_util_platform.c
APPLICATION_SRCS += nrf_drv_common.c
APPLICATION_SRCS += nrf_delay.c
APPLICATION_SRCS += nrf_drv_uart.c
# APPLICATION_SRCS += nrf_log.c
# APPLICATION_SRCS += app_uart.c
APPLICATION_SRCS += app_fifo.c
APPLICATION_SRCS += app_uart_fifo.c
APPLICATION_SRCS += retarget.c
APPLICATION_SRCS += led.c

APPLICATION_SRCS += simple_ble.c
# APPLICATION_SRCS += simple_adv.c

# RAM_KB = 32

SOFTDEVICE_MODEL = s130
SDK_VERSION = 11

LIBRARY_PATHS += . ../../include
SOURCE_PATHS += ../../src

CFLAGS = -Os
# CFLAGS += -DNRF_LOG_USES_RTT=1

# LDFLAGS += --specs=nano.specs

NRF_BASE_PATH ?= ../..
include $(NRF_BASE_PATH)/make/Makefile

