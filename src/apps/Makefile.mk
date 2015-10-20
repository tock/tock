CFLAGS_APPS := -fPIC -msingle-pic-base -mno-pic-data-is-text-relative 

include src/apps/*/Makefile.mk
